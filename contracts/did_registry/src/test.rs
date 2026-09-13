#![cfg(test)]

use super::{events, *};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Events, MockAuth, MockAuthInvoke},
    xdr, Address, BytesN, ConversionError, Env, IntoVal, InvokeError, Symbol, TryFromVal, Val,
};

/// Helper: set up a DidRegistry contract with all auths mocked.
///
/// Used for behavior tests. The scoped-auth security tests use
/// `setup_scoped` instead so missing signatures actually fail.
fn setup() -> (Env, Address, DidRegistryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(DidRegistry, (admin.clone(),));
    let client = DidRegistryClient::new(&env, &contract_id);

    (env, admin, client)
}

/// Helper: set up a DidRegistry contract WITHOUT `mock_all_auths`.
///
/// Authorization must be provided explicitly via `client.mock_auths(...)`,
/// so tests can prove exactly which signatures a call requires.
fn setup_scoped() -> (Env, Address, Address, DidRegistryClient<'static>) {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(DidRegistry, (admin.clone(),));
    let client = DidRegistryClient::new(&env, &contract_id);

    (env, admin, contract_id, client)
}

/// Create a DID in a scoped-auth env, authorizing only `user`.
fn scoped_create(
    env: &Env,
    contract_id: &Address,
    client: &DidRegistryClient<'static>,
    user: &Address,
) -> BytesN<32> {
    client
        .mock_auths(&[MockAuth {
            address: user,
            invoke: &MockAuthInvoke {
                contract: contract_id,
                fn_name: "create_did",
                args: (user.clone(),).into_val(env),
                sub_invokes: &[],
            },
        }])
        .create_did(user)
}

/// Authorize both `user1` and `user2` for a `link_wallet` call and invoke it.
///
/// Both signatures are present, so the call reaches contract logic and returns
/// the contract's own `Result`.
fn scoped_link(
    env: &Env,
    client: &DidRegistryClient<'static>,
    contract_id: &Address,
    user1: &Address,
    user2: &Address,
    did: &BytesN<32>,
) -> Result<Result<(), ConversionError>, Result<Error, InvokeError>> {
    client
        .mock_auths(&[
            MockAuth {
                address: user1,
                invoke: &MockAuthInvoke {
                    contract: contract_id,
                    fn_name: "link_wallet",
                    args: (did.clone(), user2.clone()).into_val(env),
                    sub_invokes: &[],
                },
            },
            MockAuth {
                address: user2,
                invoke: &MockAuthInvoke {
                    contract: contract_id,
                    fn_name: "link_wallet",
                    args: (did.clone(), user2.clone()).into_val(env),
                    sub_invokes: &[],
                },
            },
        ])
        .try_link_wallet(did, user2)
}

/// Get the first topic (a `Symbol` Val) of the most recent event.
fn last_event_first_topic(env: &Env) -> Symbol {
    let events = env.events().all();
    let last = events.events().last().expect("expected an event");
    match &last.body {
        xdr::ContractEventBody::V0(v0) => {
            let topic = v0.topics.first().expect("expected a topic");
            let val = Val::try_from_val(env, topic).unwrap();
            Symbol::try_from_val(env, &val).unwrap()
        }
    }
}

/// Get the data payload of the most recent event.
fn last_event_data(env: &Env) -> Val {
    let events = env.events().all();
    let last = events.events().last().expect("expected an event");
    match &last.body {
        xdr::ContractEventBody::V0(v0) => Val::try_from_val(env, &v0.data).unwrap(),
    }
}

/// A random (unregistered) DID for "does not exist" tests.
fn nonexistent_did(env: &Env) -> BytesN<32> {
    BytesN::<32>::random(env)
}

// ---------------------------------------------------------------------------
// create_did
// ---------------------------------------------------------------------------

#[test]
fn test_create_did() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    let did = client.create_did(&user);

    // A DidCreated event was published
    assert_eq!(
        last_event_first_topic(&env),
        Symbol::new(&env, "did_created")
    );
    let created = events::DidCreatedEvent::try_from_val(&env, &last_event_data(&env)).unwrap();
    assert_eq!(created.did_id, did.clone());
    assert_eq!(created.owner, user.clone());

    // Owner resolves to the new DID
    assert_eq!(client.get_did_for_wallet(&user), Some(did.clone()));

    // DID has exactly one linked wallet: the creator
    let wallets = client.get_linked_wallets(&did);
    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets.get_unchecked(0), user);

    // New DIDs are not verified
    assert!(!client.is_verified(&did));
}

#[test]
fn test_get_did() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    let did = client.create_did(&user);
    let record = client
        .get_did(&did)
        .expect("registered DID resolves to a record");

    assert_eq!(record.did_id, did);
    assert_eq!(record.owner, user);
    assert_eq!(record.is_verified, false);
}

#[test]
fn test_get_did_unknown() {
    let (env, _admin, client) = setup();
    let bogus = BytesN::<32>::random(&env);

    assert!(client.get_did(&bogus).is_none());
}

#[test]
fn test_create_did_distinct_owner_distinct_did() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(DidRegistry, (admin,));
    let client = DidRegistryClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let did1 = client.create_did(&user1);
    let did2 = client.create_did(&user2);

    assert_ne!(did1, did2);
}

#[test]
fn test_create_did_duplicate_rejected() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    let did = client.create_did(&user);

    let result = client.try_create_did(&user);
    assert!(matches!(result, Err(Ok(Error::DidAlreadyExists))));
    assert_eq!(did, client.get_did_for_wallet(&user).unwrap());
}

// ---------------------------------------------------------------------------
// link_wallet
// ---------------------------------------------------------------------------

#[test]
fn test_link_wallet() {
    let (env, _admin, client) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let did = client.create_did(&user1);
    client.link_wallet(&did, &user2);

    // Owner AND new wallet were both authorized (auth tree of last call)
    let auths = env.auths();
    assert!(auths.iter().any(|(addr, _)| addr == &user1));
    assert!(auths.iter().any(|(addr, _)| addr == &user2));

    // A WalletLinked event was published
    assert_eq!(
        last_event_first_topic(&env),
        Symbol::new(&env, "wallet_linked")
    );

    // Both wallets resolve to the same DID
    assert_eq!(client.get_did_for_wallet(&user1), Some(did.clone()));
    assert_eq!(client.get_did_for_wallet(&user2), Some(did.clone()));

    // The DID now has two linked wallets
    let wallets = client.get_linked_wallets(&did);
    assert_eq!(wallets.len(), 2);
    assert_eq!(wallets.get_unchecked(0), user1);
    assert_eq!(wallets.get_unchecked(1), user2);
}

#[test]
fn test_link_wallet_did_not_found() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    let did = nonexistent_did(&env);
    let result = client.try_link_wallet(&did, &user);
    assert!(matches!(result, Err(Ok(Error::DidNotFound))));
}

#[test]
fn test_link_wallet_wallet_already_linked() {
    let (env, _admin, client) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let _did1 = client.create_did(&user1);
    let did2 = client.create_did(&user2);

    // user1 already has their own DID — cannot also be linked to user2's
    let result = client.try_link_wallet(&did2, &user1);
    assert!(matches!(result, Err(Ok(Error::WalletAlreadyLinked))));
}

// ---------------------------------------------------------------------------
// link_wallet — mutual consent (scoped auth)
// ---------------------------------------------------------------------------

#[test]
fn test_link_wallet_requires_mutual_consent() {
    let (env, _admin, contract_id, client) = setup_scoped();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let did = scoped_create(&env, &contract_id, &client, &user1);

    // Case 1: owner signs, but the wallet being linked does not → must fail
    let owner_only = client
        .mock_auths(&[MockAuth {
            address: &user1,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "link_wallet",
                args: (did.clone(), user2.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_link_wallet(&did, &user2);
    assert!(owner_only.is_err());

    // Case 2: the wallet signs, but the DID owner does not → must fail
    let wallet_only = client
        .mock_auths(&[MockAuth {
            address: &user2,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "link_wallet",
                args: (did.clone(), user2.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_link_wallet(&did, &user2);
    assert!(wallet_only.is_err());

    // In no failed attempt was the wallet actually linked
    assert_eq!(client.get_did_for_wallet(&user2), None);

    // Case 3: both signatures present → succeeds
    let result = scoped_link(&env, &client, &contract_id, &user1, &user2, &did);
    assert!(matches!(result, Ok(Ok(()))));

    // And the link is now registered
    assert_eq!(client.get_did_for_wallet(&user2), Some(did.clone()));
}

// ---------------------------------------------------------------------------
// unlink_wallet
// ---------------------------------------------------------------------------

#[test]
fn test_unlink_wallet_by_owner() {
    let (env, _admin, client) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let did = client.create_did(&user1);
    client.link_wallet(&did, &user2);

    client.unlink_wallet(&did, &user2, &user1); // caller = owner

    // A WalletUnlinked event was published
    assert_eq!(
        last_event_first_topic(&env),
        Symbol::new(&env, "wallet_unlinked")
    );

    // user2 no longer resolves to the DID; user1 still does
    assert_eq!(client.get_did_for_wallet(&user2), None);
    assert_eq!(client.get_did_for_wallet(&user1), Some(did.clone()));

    // Linked wallets list is back to one
    let wallets = client.get_linked_wallets(&did);
    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets.get_unchecked(0), user1);
}

#[test]
fn test_unlink_wallet_by_wallet_itself() {
    let (env, _admin, contract_id, client) = setup_scoped();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let did = scoped_create(&env, &contract_id, &client, &user1);
    let link = scoped_link(&env, &client, &contract_id, &user1, &user2, &did);
    assert!(matches!(link, Ok(Ok(()))));

    // The wallet itself severs the link — no owner signature present
    let result = client
        .mock_auths(&[MockAuth {
            address: &user2,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "unlink_wallet",
                args: (did.clone(), user2.clone(), user2.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_unlink_wallet(&did, &user2, &user2);
    assert!(result.is_ok());
    assert_eq!(client.get_did_for_wallet(&user2), None);
    assert_eq!(client.get_did_for_wallet(&user1), Some(did.clone()));
}

#[test]
fn test_unlink_wallet_unauthorized_caller() {
    let (env, _admin, contract_id, client) = setup_scoped();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let stranger = Address::generate(&env);

    let did = scoped_create(&env, &contract_id, &client, &user1);
    let link = scoped_link(&env, &client, &contract_id, &user1, &user2, &did);
    assert!(matches!(link, Ok(Ok(()))));

    // A stranger (neither owner nor wallet) signs the removal → Unauthorized
    let result = client
        .mock_auths(&[MockAuth {
            address: &stranger,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "unlink_wallet",
                args: (did.clone(), user2.clone(), stranger.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_unlink_wallet(&did, &user2, &stranger);
    assert!(matches!(result, Err(Ok(Error::Unauthorized))));

    // Nothing changed
    assert_eq!(client.get_did_for_wallet(&user2), Some(did.clone()));
}

#[test]
fn test_unlink_wallet_did_not_found() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    let did = nonexistent_did(&env);
    let result = client.try_unlink_wallet(&did, &user, &user);
    assert!(matches!(result, Err(Ok(Error::DidNotFound))));
}

#[test]
fn test_unlink_wallet_not_linked() {
    let (env, _admin, client) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let did = client.create_did(&user1);

    let result = client.try_unlink_wallet(&did, &user2, &user1);
    assert!(matches!(result, Err(Ok(Error::WalletNotLinked))));
}

#[test]
fn test_unlink_wallet_cannot_remove_last() {
    let (env, _admin, client) = setup();
    let user1 = Address::generate(&env);

    let did = client.create_did(&user1);

    let result = client.try_unlink_wallet(&did, &user1, &user1);
    assert!(matches!(result, Err(Ok(Error::CannotRemoveLastWallet))));
}

// ---------------------------------------------------------------------------
// set_verified
// ---------------------------------------------------------------------------

#[test]
fn test_set_verified() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    let did = client.create_did(&user);
    assert!(!client.is_verified(&did));

    client.set_verified(&did, &true);

    // VerificationStatusChanged event published on the mutating call
    assert_eq!(
        last_event_first_topic(&env),
        Symbol::new(&env, "verification_status_changed")
    );

    assert!(client.is_verified(&did));

    // Can be reset back to false
    client.set_verified(&did, &false);
    assert!(!client.is_verified(&did));
}

#[test]
fn test_set_verified_requires_admin() {
    let (env, _admin, contract_id, client) = setup_scoped();
    let user = Address::generate(&env);

    let did = scoped_create(&env, &contract_id, &client, &user);

    // A non-admin signs set_verified → admin auth is required and missing
    let result = client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_verified",
                args: (did.clone(), true).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_set_verified(&did, &true);
    assert!(result.is_err());
    assert!(!client.is_verified(&did));
}

#[test]
fn test_set_verified_admin_succeeds() {
    let (env, admin, contract_id, client) = setup_scoped();
    let user = Address::generate(&env);

    let did = scoped_create(&env, &contract_id, &client, &user);

    // The admin signs → verification is set
    let result = client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_verified",
                args: (did.clone(), true).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_set_verified(&did, &true);
    assert!(matches!(result, Ok(Ok(()))));
    assert!(client.is_verified(&did));
}

#[test]
fn test_set_verified_did_not_found() {
    let (env, _admin, client) = setup();
    let did = nonexistent_did(&env);

    let result = client.try_set_verified(&did, &true);
    assert!(matches!(result, Err(Ok(Error::DidNotFound))));
}
