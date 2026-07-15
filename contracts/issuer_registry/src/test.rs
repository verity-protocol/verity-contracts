#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Helper: set up a fresh IssuerRegistry contract and return the env and client.
fn setup() -> (Env, Address, IssuerRegistryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(IssuerRegistry, (admin.clone(),));
    let client = IssuerRegistryClient::new(&env, &contract_id);

    (env, admin, client)
}

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_add_issuer() {
    let (env, _admin, client) = setup();
    let issuer = Address::generate(&env);

    client.add_issuer(&issuer);
}

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_remove_issuer() {
    let (env, _admin, client) = setup();
    let issuer = Address::generate(&env);

    client.remove_issuer(&issuer);
}
