#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol};

/// Helper: set up a fresh CredentialContract and return the env and client.
fn setup() -> (Env, CredentialContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CredentialContract, ());
    let client = CredentialContractClient::new(&env, &contract_id);

    (env, client)
}

// ---------------------------------------------------------------------------
// Tests
//
// These tests demonstrate the testing pattern contributors should follow.
// They call into functions that are currently stubbed with todo!(), so they
// are annotated with #[should_panic]. Once the functions are implemented,
// remove the #[should_panic] annotations and these tests will pass as-is.
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_issue_credential() {
    let (env, client) = setup();
    let issuer = Address::generate(&env);
    let did = Address::generate(&env);
    let credential_type = Symbol::new(&env, "kyc_basic");
    let credential_hash = BytesN::from_array(&env, &[0u8; 32]);

    client.issue_credential(&issuer, &did, &credential_type, &credential_hash);
}

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_revoke_credential() {
    let (env, client) = setup();
    let issuer = Address::generate(&env);
    let did = Address::generate(&env);
    let credential_type = Symbol::new(&env, "kyc_basic");

    client.revoke_credential(&issuer, &did, &credential_type);
}
