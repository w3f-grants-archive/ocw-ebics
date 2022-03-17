use codec::Decode;
use frame_support::{
	assert_ok, assert_err, 
};
use std::sync::Arc;
use sp_core::{
    offchain::{testing, OffchainWorkerExt, TransactionPoolExt}, Public as CorePublic, sr25519::Public, ByteArray,
};
use sp_keystore::{SyncCryptoStore, KeystoreExt};
use sp_runtime::{ 
	RuntimeAppPublic, DispatchError, 
};
use lite_json::Serialize;

use crate::{types::{
	Transaction, IbanAccount, unpeg_request,
	TransactionType, StrVecBytes, Iban,
}, BurnRequestStatus};
use crate::helpers::{
	ResponseTypes, StatementTypes,
	get_mock_response,
};
use sp_std::convert::TryInto;

use crate::mock::*;

fn test_processing(
    statement_type: StatementTypes,
    response_type: ResponseTypes,
) {
    let (offchain, state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();
    let keystore = sp_keystore::testing::KeyStore::new();

    SyncCryptoStore::sr25519_generate_new(
        &keystore,
        crate::crypto::Public::ID,
        Some(&format!("{}/alice", "cup swing hill dinner pioneer mom stick steel sad raven oak practice")),
    ).unwrap();

    let mut t = new_test_ext(); 

	t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));

	let (response_bytes, parsed_response) = get_mock_response(
		response_type.clone(), 
		statement_type.clone()
	);

	let statements_endpoint = "http://127.0.0.1:8081/ebics/api-v1/bankstatements".to_string();

	ebics_server_response(&mut state.write(),
		testing::PendingRequest {
			method: "GET".to_string(),
			uri: statements_endpoint,
			response: Some(response_bytes),
			sent: true,
			..Default::default()
		}
	);

	t.execute_with(|| {
		let _res = FiatRampsExample::fetch_transactions_and_send_signed();

        match response_type {
            ResponseTypes::Empty => {
                // No transactions should be sent for empty statement
                assert!(pool_state.read().transactions.is_empty());
            },
            ResponseTypes::SingleStatement | ResponseTypes::MultipleStatements => {
                let tx = pool_state.write().transactions.pop().unwrap();

                assert!(pool_state.read().transactions.is_empty());

                let tx = Extrinsic::decode(&mut &*tx).unwrap();
                assert_eq!(tx.signature.unwrap().0, 0);

                assert_eq!(tx.call, Call::FiatRampsExample(crate::Call::process_statements {
                    statements: parsed_response.clone(),
                }));
            }
        }
	})
}

#[test]
fn should_fail_to_update_api_url_non_sudo() {
	let mut t = new_test_ext(); 
	let test_accounts = get_test_accounts();
	
	// Alice is a sudo account
	let bob = test_accounts[1].clone();
	let charlie = test_accounts[2].clone();

	t.execute_with(|| {
		assert_err!(
			FiatRampsExample::set_api_url(Some(bob).into(), "http://w36.com/api/v1/".as_bytes().to_vec()),
			DispatchError::BadOrigin
		);

		assert_err!(
			FiatRampsExample::set_api_url(Some(charlie).into(), "http://w36.com/api/v1/".as_bytes().to_vec()),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn should_make_http_call_and_parse() {
	let (offchain, state) = testing::TestOffchainExt::new();
	let mut t = new_test_ext(); 

	t.register_extension(OffchainWorkerExt::new(offchain));

	let (response_bytes, parsed_response) = get_mock_response(
		ResponseTypes::SingleStatement, 
		StatementTypes::IncomingTransactions
	);

	let statements_endpoint = "http://127.0.0.1:8081/ebics/api-v1/bankstatements".to_string();

	ebics_server_response(&mut state.write(),
		testing::PendingRequest {
			method: "GET".to_string(),
			uri: statements_endpoint.clone(),
			response: Some(response_bytes.clone()),
			sent: true,
			..Default::default()
		}
	);

	t.execute_with(|| {
		let response = FiatRampsExample::fetch_json("http://127.0.0.1:8081/ebics/api-v1".as_bytes()).unwrap();
		let raw_array = response.as_array();
		
		let statements = match raw_array {
			Some(v) => {
				let mut balances: Vec<(IbanAccount, Vec<Transaction>)> = Vec::with_capacity(v.len());
				for val in v.iter() {
					// extract iban account
					let iban_account = match IbanAccount::from_json_value(&val) {
						Some(account) => account,
						None => Default::default(),
					};

					// extract transactions
					let mut transactions = Transaction::parse_transactions(&val, TransactionType::Outgoing).unwrap_or_default();
					let mut incoming_transactions = Transaction::parse_transactions(&val, TransactionType::Incoming).unwrap_or_default();
					
					transactions.append(&mut incoming_transactions);
					
					balances.push((iban_account, transactions));
				}
				balances
			},
			None => Default::default(),
		};

		assert_eq!(statements.len(), 1);
		assert_eq!(statements[0].0, parsed_response[0].0);
		assert_eq!(statements[0].1, parsed_response[0].1);
	})
}

#[test]
fn test_process_empty_statement() {
    test_processing(
        StatementTypes::IncomingTransactions,
        ResponseTypes::Empty,
    )
}

#[test]
fn test_process_incoming_transactions() {
    test_processing(
        StatementTypes::IncomingTransactions,
        ResponseTypes::SingleStatement,
    )
}

#[test]
fn test_process_outgoing_transactions() {
    test_processing(
        StatementTypes::OutgoingTransactions,
        ResponseTypes::SingleStatement,
    )
}

#[test]
fn test_process_multiple_statements() {
    test_processing(
        StatementTypes::IncomingTransactions,
        ResponseTypes::MultipleStatements,
    )
}

#[test]
fn test_process_multiple_statements_outgoing() {
    test_processing(
        StatementTypes::OutgoingTransactions,
        ResponseTypes::MultipleStatements,
    )
}

#[test]
fn test_iban_mapping() {
	let mut t = new_test_ext();

	let test_accounts = get_test_accounts();
	
	let alice = test_accounts[0].clone();
	let bob = test_accounts[1].clone();
	let charlie = test_accounts[2].clone();

	let alice_iban: Iban = "CH2108307000289537320".as_bytes().try_into().expect("Failed to convert string to bytes");
	let bob_iban: Iban = "CH1230116000289537312".as_bytes().try_into().expect("Failed to convert string to bytes");
	let charlie_iban: Iban = "CH1230116000289537313".as_bytes().try_into().expect("Failed to convert string to bytes");

	t.execute_with(|| {
		assert_ok!(FiatRampsExample::map_iban_account(
			Some(alice.clone()).into(),
			IbanAccount {
				iban: alice_iban.clone(),
				balance: 100,
				last_updated: 0,
			}
		));
		assert_ok!(FiatRampsExample::map_iban_account(
			Some(bob.clone()).into(),
			IbanAccount {
				iban: bob_iban.clone(),
				balance: 100,
				last_updated: 0,
			}
		));

		assert_ok!(FiatRampsExample::map_iban_account(
			Some(charlie.clone()).into(),
			IbanAccount {
				iban: charlie_iban.clone(),
				balance: 100,
				last_updated: 0,
			}
		));

		assert_eq!(FiatRampsExample::iban_to_account(&alice_iban).unwrap(), alice.clone());
		assert_eq!(FiatRampsExample::iban_to_account(bob_iban).unwrap(), bob.clone());
		assert_eq!(FiatRampsExample::iban_to_account(charlie_iban).unwrap(), charlie.clone());

		// Unmapping should work
		assert_ok!(FiatRampsExample::unmap_iban_account(
			Some(alice.clone()).into(),
			alice_iban.clone()
		));
		// Should be mapped to Null account (0x0000000000000000000000000000000000000000)
		assert_eq!(
			FiatRampsExample::iban_to_account(alice_iban).unwrap(), 
			Public::from_slice(&[0u8; 32]).unwrap()
		);
	})
}

#[test]
fn test_burn_request() {
    let (offchain, state) = testing::TestOffchainExt::new();
    let (pool, _pool_state) = testing::TestTransactionPoolExt::new();
    let keystore = sp_keystore::testing::KeyStore::new();

	SyncCryptoStore::sr25519_generate_new(
        &keystore,
        crate::crypto::Public::ID,
        Some(&format!("{}/alice", "cup swing hill dinner pioneer mom stick steel sad raven oak practice")),
    ).unwrap();

    let mut t = new_test_ext(); 

	t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt(Arc::new(keystore)));

	let test_accounts = get_test_accounts();

	let alice = test_accounts[0].clone();
	let bob = test_accounts[1].clone();
	let charlie = test_accounts[2].clone();

	let alice_iban: Iban = "CH2108307000289537320".as_bytes().try_into().expect("Failed to convert string to bytes");
	let bob_iban: Iban = "CH1230116000289537312".as_bytes().try_into().expect("Failed to convert string to bytes");
	let charlie_iban: Iban = "CH1230116000289537313".as_bytes().try_into().expect("Failed to convert string to bytes");

	{

		let mock_unpeg_request = unpeg_request(
		&format!("{:?}", bob),
			10000,
			&bob_iban,
			&"0".to_string(),
		)
		.serialize();

		let mock_unpeg_request_1 = unpeg_request(
		&format!("{:?}", charlie),
			100,
			&charlie_iban,
			&"1".to_string(),
		)
		.serialize();

		let mock_unpeg_request_2 = unpeg_request(
		&format!("{:?}", charlie),
			1000,
			&charlie_iban,
			&"2".to_string(),
		)
		.serialize();
	
		let unpeg_endpoint = "http://127.0.0.1:8081/ebics/api-v1/unpeg";

		ebics_server_response(&mut state.write(),
			testing::PendingRequest {
				uri: unpeg_endpoint.to_string(),
				method: "POST".to_string(),
				body: mock_unpeg_request.clone(),
				response: Some(mock_unpeg_request),
				headers: [
					("Content-Type".to_string(), "application/json".to_string()), 
					("accept".to_string(), "*/*".to_string())
				].to_vec(),
				sent: true,
				..Default::default()
			}
		);

		ebics_server_response(&mut state.write(),
			testing::PendingRequest {
				uri: unpeg_endpoint.to_string(),
				method: "POST".to_string(),
				body: mock_unpeg_request_1.clone(),
				response: Some(mock_unpeg_request_1),
				headers: [
					("Content-Type".to_string(), "application/json".to_string()), 
					("accept".to_string(), "*/*".to_string())
				].to_vec(),
				sent: true,
				..Default::default()
			}
		);
		ebics_server_response(&mut state.write(),
			testing::PendingRequest {
				uri: unpeg_endpoint.to_string(),
				method: "POST".to_string(),
				body: mock_unpeg_request_2.clone(),
				response: Some(mock_unpeg_request_2),
				headers: [
					("Content-Type".to_string(), "application/json".to_string()), 
					("accept".to_string(), "*/*".to_string())
				].to_vec(),
				sent: true,
				..Default::default()
			}
		);
	}

	t.execute_with(|| {
		// Local counter to keep track of the number of burn requests
		fn check_burn_request(
			previous_pallet_balance: u128,
			request_counter: u64,
			amount: u128,
			burner: &AccountId,
			_dest_account: Option<&AccountId>,
			dest_iban: Option<&Iban>,
		) {
			// Check if burn request has been added to the queue
			let burn_request = FiatRampsExample::burn_request(request_counter).unwrap();
			assert_eq!(burn_request.amount, amount);
			assert_eq!(burn_request.burner, burner.clone());
			assert_eq!(burn_request.dest_iban, Some(dest_iban.unwrap().clone()));

			// Burn amount should be transfered to Pallet's account
			// Pallet's accounts serves as the treasury of unpegged funds
			// Once the burn request is confirmed as an outgoing transaction in the bank statement,
			// We can tag the burn request as confirmed and send funds to the destination account
			assert_eq!(
				Balances::free_balance(FiatRampsExample::account_id()), 
				previous_pallet_balance + amount
			);
			
			// Trigger processing of burn requests	
			assert_ok!(FiatRampsExample::process_burn_requests());

			// Check if burn request's status has been updated
			let burn_request = FiatRampsExample::burn_request(request_counter).unwrap();
			assert_eq!(burn_request.status, BurnRequestStatus::Sent);
		}

		// map Alice iban
		assert_ok!(FiatRampsExample::map_iban_account(
			Some(alice.clone()).into(),
			IbanAccount {
				iban: alice_iban.clone(),
				balance: 100,
				last_updated: 0,
			}
		));
		// map Bob iban
		assert_ok!(FiatRampsExample::map_iban_account(
			Some(bob.clone()).into(),
			IbanAccount {
				iban: bob_iban.clone(),
				balance: 100,
				last_updated: 0,
			}
		));

		assert_ok!(FiatRampsExample::map_iban_account(
			Some(charlie.clone()).into(),
			IbanAccount {
				iban: charlie_iban.clone(),
				balance: 0,
				last_updated: 0
			}
		));

		// Pallet's balance before unpeg request
		let initial_pallet_balance = Balances::free_balance(FiatRampsExample::account_id());
		// call `burn_to_iban` to transfer 10000 from Alice to Bob
		assert_ok!(FiatRampsExample::burn_to_iban(
			Some(alice.clone()).into(),
			10000,
			bob_iban.clone(),
		));

		check_burn_request(
			initial_pallet_balance,
			0,
			10000,
			&alice,
			Some(&bob),
			Some(&bob_iban),
		);

		// Pallet's balance before unpeg request
		let initial_pallet_balance = Balances::free_balance(FiatRampsExample::account_id());
		// make burn to address
		assert_ok!(FiatRampsExample::burn_to_address(
			Some(bob.clone()).into(),
			100,
			charlie.clone()
		));

		check_burn_request(
			initial_pallet_balance,
			1,
			100,
			&bob,
			Some(&charlie),
			Some(&charlie_iban),
		);

		// Pallet's balance before unpeg request
		let initial_pallet_balance = Balances::free_balance(FiatRampsExample::account_id());

		// Make a generic burn, similar to withdrawin money from the bank
		assert_ok!(FiatRampsExample::burn(
			Some(charlie.clone()).into(),
			1000,
		));

		check_burn_request(
			initial_pallet_balance,
			2,
			1000,
			&charlie,
			Some(&charlie),
			Some(&charlie_iban),
		);
	})
}

/// Mock server response
fn ebics_server_response(
	state: &mut testing::OffchainState,
	pending_request: testing::PendingRequest
) {
	state.expect_request(pending_request);
}
