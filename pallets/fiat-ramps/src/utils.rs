use crate::{
	types::{Deserialize, IbanOf},
	Config,
};
/// Utility functions
use lite_json::{JsonValue, NumberValue};
use sp_std::{vec, vec::Vec};

/// Utility function for parsing value from json object
pub fn parse_object(key: &str, obj: &[(Vec<char>, lite_json::JsonValue)]) -> JsonValue {
	let raw_object = obj.iter().find(|(k, _)| k.iter().copied().eq(key.chars()));
	if let Some((_, v)) = raw_object {
		return v.clone()
	}
	JsonValue::Null
}

/// Utility function for extracting value from json object
///
/// parse value of a given key from json object
pub fn extract_value<T: Deserialize<T> + Default>(
	key: &str,
	obj: &[(Vec<char>, lite_json::JsonValue)],
) -> T {
	let (_, v) = obj.iter().find(|(k, _)| k.iter().copied().eq(key.chars())).unwrap();
	if let Some(value) = T::deserialize(v) {
		return value
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
	let integer = amount / 10_000_000_000;
	let fraction = amount % 10_000_000_000;

	// Mutable copy of `fraction` that will be used to calculate length of the fraction
	let mut fraction_copy = fraction;

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

	let iban_json = JsonValue::String(iban[..].iter().map(|b| *b as char).collect::<Vec<char>>());

	JsonValue::Object(vec![
		("amount".chars().into_iter().collect(), amount_json),
		(
			"clearingSystemMemberId".chars().into_iter().collect(),
			JsonValue::String(vec!['H', 'Y', 'P', 'L', 'C', 'H', '2', '2']),
		),
		("currency".chars().into_iter().collect(), JsonValue::String(vec!['E', 'U', 'R'])),
		("nationalPayment".chars().into_iter().collect(), JsonValue::Boolean(true)),
		(
			"ourReference".chars().into_iter().collect(),
			JsonValue::String(reference.chars().into_iter().collect()),
		),
		(
			"purpose".chars().into_iter().collect(),
			JsonValue::String(dest.chars().into_iter().collect()),
		),
		(
			"receipientBankName".chars().into_iter().collect(),
			JsonValue::String(vec!['H', 'y', 'p']),
		),
		("receipientCity".chars().into_iter().collect(), JsonValue::String(vec!['e'])),
		("receipientCountry".chars().into_iter().collect(), JsonValue::String(vec!['C', 'H'])),
		("receipientName".chars().into_iter().collect(), JsonValue::String(vec!['e'])),
		("receipientIban".chars().into_iter().collect(), iban_json),
		("receipientStreet".chars().into_iter().collect(), JsonValue::String(vec!['e'])),
		("receipientStreetNr".chars().into_iter().collect(), JsonValue::String(vec!['2', '5'])),
		(
			"receipientZip".chars().into_iter().collect(),
			JsonValue::String(vec!['6', '3', '4', '0']),
		),
	])
}
