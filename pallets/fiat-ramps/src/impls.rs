//! Implementations of traits and types for the pallet
use core::convert::TryInto;

use crate::*;
use crate::types::*;
use sp_std::default::Default;

use self::utils::{extract_value, parse_object};

impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

impl Default for OcwActivity {
	fn default() -> Self {
		OcwActivity::None
	}
}

impl From<u32> for OcwActivity {
	fn from(activity: u32) -> Self {
		match activity {
			0 => OcwActivity::ProcessStatements,
			1 => OcwActivity::ProcessBurnRequests,
			_ => OcwActivity::None,
		}
	}
}

impl Default for TransactionType {
	fn default() -> Self {
		Self::None
	}
}

/// Implement `Deserialize` for common types
impl Deserialize<Vec<u8>> for Vec<u8> {
    fn deserialize(json: &JsonValue) -> Option<Vec<u8>> {
        json.clone()
            .to_string()
            .map(|v| v.iter().map(|c| *c as u8).collect::<Vec<_>>())
    }
}

impl Deserialize<u128> for u128 {
    fn deserialize(json: &JsonValue) -> Option<u128> {
        json.clone()
            .to_number()
            .map(|num| {
                let value_1 = num.integer as u128 * 10_u128.pow(num.exponent as u32 + 10);
				let value_2 = num.fraction as u128 * 10_u128.pow(
					num.exponent as u32 + 10 - num.fraction_length
				);
				value_1 + value_2
            })
    }
}

/// Functions of `Transaction<T>` type
impl<MaxIbanLength: Get<u32>, MaxStringLength: Get<u32>> Transaction<MaxIbanLength, MaxStringLength> {
    pub fn new(
        iban: Iban<MaxIbanLength>,
        name: BoundedVec<u8, MaxStringLength>,
        currency: BoundedVec<u8, MaxStringLength>,
        amount: u128,
        reference: BoundedVec<u8, MaxStringLength>,
        tx_type: TransactionType
    ) -> Self {
        Self {
            iban,
            name,
            currency,
            amount,
            reference,
            tx_type
        }
    }

	// Get single transaction instance from json
	pub fn from_json_statement(json: &JsonValue, tx_type: &TransactionType) -> Option<Self> {
		let raw_object = json.as_object();
        
        match raw_object {
            Some(obj) => {
                let iban = extract_value::<Vec<u8>>("iban", obj).try_into().expect("Invalid IBAN");
                let name = extract_value::<Vec<u8>>("name", obj).try_into().expect("Invalid name");
                let currency = extract_value::<Vec<u8>>("currency", obj).try_into().expect("Invalid currency");
                let amount = extract_value::<u128>("amount", obj);
                let reference = extract_value::<Vec<u8>>("reference", obj).try_into().expect("Invalid reference");

                Some(Self::new(
                    iban,
                    name,
                    currency,
                    amount,
                    reference,
                    *tx_type
                ))
            },
            _ => None,
        }
	}
	
	/// Parse multiple transactions from `JsonValue`
	pub fn parse_transactions(json: &JsonValue, transaction_type: TransactionType) -> Option<Vec<Self>> {
        // Get the key string for the transaction type
        let key_string = match transaction_type {
            TransactionType::Incoming => "incomingTransactions",
            TransactionType::Outgoing => "outgoingTransactions",
            _ => return None,
        };

        let mut transactions = Vec::new();
        
		match json {
			JsonValue::Object(obj) => {
                match parse_object(key_string, &obj) {
                    JsonValue::Array(txs) => {
                        for json_tx in txs {
                            if let Some(tx) = Self::from_json_statement(&json_tx, &transaction_type) {
                                transactions.push(tx);
                            }
                        }
                    },
                    _ => {}
                }
            },
			_ => {}
		};

        Some(transactions)
	}
}

/// Functions of `BankAccount` type
impl<MaxLength: Get<u32>> BankAccount<MaxLength> {
    pub fn new (
        iban: Iban<MaxLength>,
        balance: u128,
        last_updated: u64,
        behaviour: AccountBehaviour<MaxLength>,
    ) -> Self {
        Self {
            iban,
            balance,
            last_updated,
            behaviour,
        }
    }
	
	pub fn from_json_value(json: &JsonValue) -> Option<Self> {
        let raw_object = json.as_object();
		let iban_account = match raw_object {
			Some(obj) => {
				let iban = extract_value::<Vec<u8>>("iban", obj).try_into().unwrap_or_default();
				let balance = extract_value::<u128>("balanceCL", obj);

				Self::new(
					iban,
					balance,
					0,
                    AccountBehaviour::default()
                )
			},
			_ => return None,
		};
		Some(iban_account)
	}
}

/// Utility functions
pub mod utils {
    use lite_json::{JsonValue, NumberValue};
    use crate::Config;
    use crate::types::{Deserialize, IbanOf};

    /// Utility function for parsing value from json object
    pub fn parse_object(key: &str, obj: &[(Vec<char>, lite_json::JsonValue)]) -> JsonValue {
        let raw_object = obj.into_iter().find(|(k, _)| k.iter().copied().eq(key.chars()));
        if let Some((_, v)) = raw_object {
            return v.clone();
        }
        JsonValue::Null
    }

    /// Utility function for extracting value from json object
    ///
    /// parse value of a given key from json object
    pub fn extract_value<T: Deserialize<T> + Default>(key: &str, obj: &[(Vec<char>, lite_json::JsonValue)]) -> T {
        let (_, v) = obj.into_iter().find(|(k, _)| k.iter().copied().eq(key.chars())).unwrap();
        if let Some(value) = T::deserialize(v) {
            return value;
        }
        Default::default()
    }

    /// Unpeq request template
    /// 
    /// # Arguments
    /// 
    /// `account_id` - Sender of the unpeq request
    /// `amount` - Amount of the unpeq request
    /// `iban` - IBAN of the receiver
    /// `reference` - Reference of the unpeq request, we save request id in this field
    pub fn unpeg_request<T: Config>(
        dest: &str, 
        amount: u128, 
        iban: &IbanOf<T>,
        reference: &str,
    ) -> JsonValue {

        // First step is to convert amount to NumberValue type
        let integer = amount / 1_000_000_0000;
        let fraction = amount % 1_000_000_0000;

        // Mutable copy of `fraction` that will be used to calculate length of the fraction
        let mut fraction_copy = fraction.clone();

        let fraction_length = {
            let mut len = 0;
            
            while fraction_copy > 0 {
                fraction_copy /= 10;
                len += 1;
            }
            len
        };

        let amount_json = JsonValue::Number(NumberValue {
            integer: integer as u64,
            fraction: fraction as u64,
            fraction_length,
            exponent: 0,
            negative: false,
        });

        let iban_json = JsonValue::String(
            iban[..].iter().map(|b| *b as char).collect::<Vec<char>>()
        );

        JsonValue::Object(
            vec![
                (
                    "amount".chars().into_iter().collect(), 
                    amount_json
                ),
                (
                    "clearingSystemMemberId".chars().into_iter().collect(),
                    JsonValue::String(vec!['H', 'Y', 'P', 'L', 'C', 'H', '2', '2'])
                ),
                (
                    "currency".chars().into_iter().collect(), 
                    JsonValue::String(vec!['E', 'U', 'R'])
                ),
                (
                    "nationalPayment".chars().into_iter().collect(),
                    JsonValue::Boolean(true)
                ),
                (
                    "ourReference".chars().into_iter().collect(), 
                    JsonValue::String(reference.chars().into_iter().collect())
                ),
                (
                    "purpose".chars().into_iter().collect(), 
                    JsonValue::String(dest.chars().into_iter().collect())
                ),
                (
                    "receipientBankName".chars().into_iter().collect(),
                    JsonValue::String(vec!['H', 'y', 'p'])
                ),
                (
                    "receipientCity".chars().into_iter().collect(),
                    JsonValue::String(vec!['e'])
                ),
                (
                    "receipientCountry".chars().into_iter().collect(),
                    JsonValue::String(vec!['C', 'H'])
                ),
                (
                    "receipientName".chars().into_iter().collect(),
                    JsonValue::String(vec!['e'])
                ),
                (
                    "receipientIban".chars().into_iter().collect(),
                    iban_json
                ),
                (
                    "receipientStreet".chars().into_iter().collect(),
                    JsonValue::String(vec!['e'])
                ),
                (
                    "receipientStreetNr".chars().into_iter().collect(),
                    JsonValue::String(vec!['2', '5'])
                ),
                (
                    "receipientZip".chars().into_iter().collect(),
                    JsonValue::String(vec!['6', '3', '4', '0'])
                )
            ]
        )
    }
}
