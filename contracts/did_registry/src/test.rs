#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Helper: set up a fresh DidRegistry contract and return the env, admin, and client.
fn setup() -> (Env, Address, DidRegistryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(DidRegistry, (admin.clone(),));
    let client = DidRegistryClient::new(&env, &contract_id);

    (env, admin, client)
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
fn test_create_did() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    // create_did should:
    // 1. Require auth from `user`
    // 2. Generate a new DID address
    // 3. Store a DidRecord with the user as owner
    // 4. Link the user's wallet to the DID
    // 5. Return the DID address
    let did = client.create_did(&user);

    // After implementation, verify:
    // - The user's wallet resolves to this DID
    let resolved = client.get_did_for_wallet(&user);
    assert_eq!(resolved, Some(did.clone()));

    // - The DID has one linked wallet
    let wallets = client.get_linked_wallets(&did);
    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets.get_unchecked(0), user);

    // - New DIDs are not verified by default
    assert!(!client.is_verified(&did));
}

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_link_wallet() {
    let (env, _admin, client) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // First, create a DID for user1
    let did = client.create_did(&user1);

    // link_wallet should:
    // 1. Require auth from the DID owner (user1)
    // 2. Add user2's wallet to the DID's linked wallets
    // 3. Update the WalletToDid mapping for user2
    client.link_wallet(&did, &user2);

    // After implementation, verify:
    // - Both wallets resolve to the same DID
    assert_eq!(client.get_did_for_wallet(&user1), Some(did.clone()));
    assert_eq!(client.get_did_for_wallet(&user2), Some(did.clone()));

    // - The DID now has two linked wallets
    let wallets = client.get_linked_wallets(&did);
    assert_eq!(wallets.len(), 2);
}
