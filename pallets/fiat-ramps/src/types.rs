use sp_std::convert::TryInto;

/// Contains Pallet types and methods
/// Contains Transaction and IbanAccount types
use crate::*;

/// String vector bytes
pub type StringOf<T> = BoundedVec<u8, T::MaxStringLength>;

/// IBAN account
pub type IbanOf<T> = BoundedVec<u8, T::MaxIbanLength>;

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
	number: u64,
	public: Public,
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
	pub iban: Iban,
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
