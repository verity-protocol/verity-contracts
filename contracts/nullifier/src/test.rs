#![cfg(test)]

use super::*;
use soroban_sdk::{BytesN, Env};

/// Helper: set up a fresh NullifierContract and return the env and client.
fn setup() -> (Env, NullifierContractClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(NullifierContract, ());
    let client = NullifierContractClient::new(&env, &contract_id);

    (env, client)
}

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_store_nullifier() {
    let (env, client) = setup();
    let hash = BytesN::from_array(&env, &[1u8; 32]);

    client.store_nullifier(&hash);
}

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_check_nullifier() {
    let (env, client) = setup();
    let hash = BytesN::from_array(&env, &[1u8; 32]);

    client.check_nullifier(&hash);
}
