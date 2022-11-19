//! Types of this pallet
use crate::*;
use frame_support::PalletId;
use sp_runtime::KeyTypeId;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When an offchain worker is signing transactions it's going to request keys from type
/// `KeyTypeId` via the keystore to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ramp");

/// Pallet ID
/// Account id will be derived from this pallet id.
pub const PALLET_ID: PalletId = PalletId(*b"FiatRamp");

/// Hardcoded inital test api endpoint
const API_URL: &[u8; 33] = b"http://w.e36.io:8093/ebics/api-v1";

/// Account id of
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

/// Balance type
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

/// String vector bytes
pub type StringOf<T> = BoundedVec<u8, <T as Config>::MaxStringLength>;

/// IBAN account
pub type IbanOf<T> = BoundedVec<u8, <T as Config>::MaxIbanLength>;

/// Transaction type
pub type TransactionOf<T> = Transaction<T>;

/// Types of activities that can be performed by the OCW
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum OcwActivity {
	ProcessStatements,
	ProcessBurnRequests,
	None,
}

/// Type that represents a burn request
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct BurnRequest<T: Config, Balance: MaxEncodedLen + Default> {
	pub id: u64,
	pub burner: IbanOf<T>,
	pub dest_iban: Option<IbanOf<T>>,
	pub amount: Balance,
}

/// Trait for deseralizing a value from a JsonValue type
pub trait Deserialize<T> {
    fn deserialize(value: &JsonValue) -> Option<T>;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub(crate) struct Payload<Public> {
	pub number: u64,
	pub public: Public,
}

/// Type of transaction
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum TransactionType {
	Incoming,
	Outgoing,
	None
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo)]
pub struct Transaction<T: Config> {
	// from
	pub iban: IbanOf<T>,
	pub name: StringOf<T>,
	pub currency: StringOf<T>,
	pub amount: u128,
	// to
	pub reference: StringOf<T>,
	pub tx_type: TransactionType
}

/// IbanAccount Type contains the basic information of an account
/// 
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo)]
pub struct IbanAccount<T: Config> {
	/// IBAN number of the account
	pub iban: IbanOf<T>,
	/// Closing balance of the account
	pub balance: u128,
	/// Last time the statement was updated
	pub last_updated: u64
}
