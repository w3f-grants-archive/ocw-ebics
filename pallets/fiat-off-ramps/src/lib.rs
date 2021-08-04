//! A demonstration of an offchain worker that sends onchain callbacks

#![cfg_attr(not(feature = "std"), no_std)]


use core::{convert::TryInto, fmt};
use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, 
	dispatch::DispatchResult, traits::{Get}
};
use codec::{Decode, Encode};
use serde_json::{Value};
use frame_system::{ensure_none, offchain::{AppCrypto, CreateSignedTransaction, SendTransactionTypes, SendUnsignedTransaction, SignedPayload, SigningTypes, SubmitTransaction}};
use sp_core::{crypto::KeyTypeId};
use sp_runtime::{RuntimeDebug, offchain as rt_offchain, offchain::{ storage::StorageValueRef, storage_lock::{BlockAndTime, StorageLock}}, transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
	}};

use serde::{Deserialize};

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When an offchain worker is signing transactions it's going to request keys from type
/// `KeyTypeId` via the keystore to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");
const NUM_VEC_LEN: usize = 10;
/// The type to sign and send transactions.
const UNSIGNED_TXS_PRIORITY: u64 = 100;

const FETCHED_CRYPTO: (&[u8], &[u8], &[u8]) = (
	b"BTC", b"coincap",
	b"https://api.coincap.io/v2/assets/bitcoin"
);

const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds
const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD + 1000; // in milli-seconds
const LOCK_BLOCK_EXPIRATION: u32 = 3; // in block number

const ONCHAIN_TX_KEY: &[u8] = b"fiat-off-ramps::storage::tx";

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrapper.
/// We can utilize the supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// them with the pallet-specific identifier.
pub mod crypto {
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	use sp_runtime::{traits::Verify, MultiSignature, MultiSigner};

	app_crypto!(sr25519, KEY_TYPE);
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Payload<Public> {
	number: u64,
	public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

#[derive(Debug, Deserialize, Encode, Decode, Default)]
struct IndexingData(Vec<u8>, u64);

type StrVecBytes = Vec<u8>;

/// This is the pallet's configuration trait
pub trait Config: pallet_timestamp::Config + frame_system::Config + CreateSignedTransaction<Call<Self>> {
	/// The identifier type for an offchain worker.
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	type SubmitTransaction: SendTransactionTypes<<Self as Config>::Call>;

	type BlockFetchDur: Get<Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Config> as FiatOffRamps {
		TokenSrcPPMap: map hasher(blake2_128_concat) StrVecBytes => Vec<(T::Moment, u64)>
	}
}

decl_event!(
	/// Events generated by the module.
	pub enum Event<T>
	where
		Moment = <T as pallet_timestamp::Config>::Moment,
	{
		/// Event generated when a new number is accepted to contribute to the average.
		FetchedPrice(StrVecBytes, StrVecBytes, Moment, u64),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		// Error returned when not sure which ocw function to executed
		UnknownOffchainMux,

		// Error returned when making signed transactions in off-chain worker
		NoLocalAcctForSigning,
		OffchainSignedTxError,

		// Error returned when making unsigned transactions in off-chain worker
		OffchainUnsignedTxError,

		// Error returned when making unsigned transactions with signed payloads in off-chain worker
		OffchainUnsignedTxSignedPayloadError,

		// Error returned when fetching github info
		HttpFetchingError,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 10000]
		pub fn record_price(
			origin,
			_block: T::BlockNumber,
			crypto_info: (StrVecBytes, StrVecBytes, StrVecBytes),
			price: u64
		) -> DispatchResult {
			let who = ensure_none(origin)?;
			let (sym, remote_src) = (crypto_info.0, crypto_info.1);
			let now = pallet_timestamp::Now::get();
			debug(format!("record_price: {:?}, {:?}, {:?}",
				*core::str::from_utf8(&sym).map_err(|_| "`sym` conversion error")?,
				*core::str::from_utf8(&remote_src).map_err(|_| "`remote_src` conversion error")?,
				price)
		  	);
			<TokenSrcPPMap<T>>::mutate(&sym, |pp_vec| pp_vec.push((now, price)));

			Self::deposit_event(RawEvent::FetchedPrice(sym, remote_src, now, price));
			Ok(())
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug("Entering off-chain worker");
			let duration = T::BlockFetchDur::get();
			// Here we are showcasing various techniques used when running off-chain workers (ocw)
			// 1. Sending signed transaction from ocw
			// 2. Sending unsigned transaction from ocw
			// 3. Sending unsigned transactions with signed payloads from ocw
			// 4. Fetching JSON via http requests in ocw
			// const TRANSACTION_TYPES: usize = 4;
			// let result = match block_number.try_into().unwrap_or(0) % TRANSACTION_TYPES	{
			// 	1 => Self::record_price(),
			// 	_ => Err(Error::<T>::UnknownOffchainMux),
			// };

			let (sym, remote_src, remote_url) = FETCHED_CRYPTO;
			if duration > 0.into() && block_number % duration == 0.into() {
				if let Err(e) = Self::fetch_price(block_number, *sym, *remote_src, *remote_url) {
					debug(format!("Error fetching: {:?}, {:?}: {:?}",
					core::str::from_utf8(sym).unwrap(),
					core::str::from_utf8(remote_src).unwrap(),
					e));
				}
			}


			// Reading back the off-chain indexing value. It is exactly the same as reading from
			// ocw local storage.
			let key = Self::derived_key(block_number);
			let oci_mem = StorageValueRef::persistent(&key);

			if let Some(Some(data)) = oci_mem.get::<IndexingData>() {
				debug(format!("off-chain indexing data: {:?}, {:?}",
					str::from_utf8(&data.0).unwrap_or("error"), data.1));
			} else {
				debug(format!("no off-chain indexing data retrieved."));
			}
		}
	}
}

impl<T: Config> Module<T> {
	fn fetch_json<'a>(remote_url: &'a [u8]) -> serde_json::Result<Value> {
		let remote_url_str = core::str::from_utf8(remote_url)
			.map_err(|_| "Error in converting remote_url to string")?;

		let pending = rt_offchain::http::Request::get(remote_url_str).send()
			.map_err(|_| "Error in sending http GET request")?;

		let response = pending.wait()
			.map_err(|_| "Error in waiting http response back")?;

		if response.code != 200 {
			debug(format!("Unexpected status code: {}", response.code));
			return Ok(Value::Null)
		}

		let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();
	
		let json_val = serde_json::from_str(&core::str::from_utf8(&json_result)?)?;
		debug( format!("json_val {:?}", json_val));
		Ok(json_val)
	}

	fn fetch_price<'a>(
		block: T::BlockNumber,
		sym: &'a [u8],
		remote_src: &'a [u8],
		remote_url: &'a [u8]
	) -> Result<(), <Error<T>>> {
		debug(format!("fetching price: {:?}:{:?}",
		  core::str::from_utf8(sym).unwrap(),
		  core::str::from_utf8(remote_src).unwrap())
		);

		let json = Self::fetch_json(remote_url)?;
		let price = Self::fetch_price_coincap(json);

		let call = Call::record_price(
			block,
			(sym.to_vec(), remote_src.to_vec(), remote_url.to_vec()),
			price
		);

		SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(|_| {
			debug("Failed in offchain_unsigned_tx");
			<Error<T>>::OffchainUnsignedTxError
		})
	}

	fn fetch_price_coincap(json: Value) -> Result<u64, serde_json::Error> {
		let val_f64: f64 = json.get("USD");
		Ok((val_f64 * 1000.).round() as u64)
	}
}

impl<T: Config> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		let valid_tx = |provide| {
			ValidTransaction::with_tag_prefix("fiat-off-ramps")
				.priority(UNSIGNED_TXS_PRIORITY)
				.and_provides([&provide])
				.longevity(3)
				.propagate(true)
				.build()
		};

		match call {
			Call::record_price(_number,  (_sym, _remote, _url)) => valid_tx(b"record_price"),
			_ => InvalidTransaction::Call.into(),
		}
	}
}

impl<T: Config> rt_offchain::storage_lock::BlockNumberProvider for Module<T> {
	type BlockNumber = T::BlockNumber;
	fn current_block_number() -> Self::BlockNumber {
		<frame_system::Pallet<T>>::block_number()
	}
}
