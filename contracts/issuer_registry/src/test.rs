#![cfg(test)]

use super::{events, *};
use soroban_sdk::{
    testutils::{Address as _, Events, MockAuth, MockAuthInvoke},
    xdr, Address, ConversionError, Env, IntoVal, InvokeError, Symbol, TryFromVal, Val,
};

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

/// Helper: set up an IssuerRegistry with all auths mocked.
fn setup() -> (Env, Address, IssuerRegistryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(IssuerRegistry, (admin.clone(),));
    let client = IssuerRegistryClient::new(&env, &contract_id);

    (env, admin, client)
}

/// Helper: set up an IssuerRegistry WITHOUT `mock_all_auths`.
///
/// Authorization must be provided explicitly via `client.mock_auths(...)`,
/// so tests can prove that only the admin can approve issuers.
fn setup_scoped() -> (Env, Address, Address, IssuerRegistryClient<'static>) {
    let env = Env::default();

    let admin = Address::generate(&env);
    let contract_id = env.register(IssuerRegistry, (admin.clone(),));
    let client = IssuerRegistryClient::new(&env, &contract_id);

    (env, admin, contract_id, client)
}

/// Authorize `signer` for an `add_issuer` call and invoke it.
fn scoped_add(
    env: &Env,
    client: &IssuerRegistryClient<'static>,
    contract_id: &Address,
    signer: &Address,
    issuer: &Address,
) -> Result<Result<(), ConversionError>, Result<Error, InvokeError>> {
    client
        .mock_auths(&[MockAuth {
            address: signer,
            invoke: &MockAuthInvoke {
                contract: contract_id,
                fn_name: "add_issuer",
                args: (issuer.clone(),).into_val(env),
                sub_invokes: &[],
            },
        }])
        .try_add_issuer(issuer)
}

// ---------------------------------------------------------------------------
// Event helpers
// ---------------------------------------------------------------------------

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

/// Get the data (a `Val`) payload of the most recent event.
fn last_event_data(env: &Env) -> Val {
    let events = env.events().all();
    let last = events.events().last().expect("expected an event");
    match &last.body {
        xdr::ContractEventBody::V0(v0) => Val::try_from_val(env, &v0.data).unwrap(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_add_issuer() {
    let (env, _admin, client) = setup();
    let issuer = Address::generate(&env);

    client.add_issuer(&issuer);

    // An IssuerAdded event was published (read before any query calls)
    assert_eq!(
        last_event_first_topic(&env),
        Symbol::new(&env, "issuer_added")
    );
    let added = events::IssuerAddedEvent::try_from_val(&env, &last_event_data(&env)).unwrap();
    assert_eq!(added.issuer, issuer);

    // Approval is recorded and immediately visible to cross-callers
    assert!(client.is_issuer_approved(&issuer));

    // An unrelated address is not approved
    let other = Address::generate(&env);
    assert!(!client.is_issuer_approved(&other));
}

#[test]
fn test_add_issuer_duplicate_rejected() {
    let (env, _admin, client) = setup();
    let issuer = Address::generate(&env);

    client.add_issuer(&issuer);
    let result = client.try_add_issuer(&issuer);

    assert!(matches!(result, Err(Ok(Error::AlreadyApproved))));
}

#[test]
fn test_add_issuer_requires_admin() {
    let (env, admin, contract_id, client) = setup_scoped();
    let issuer = Address::generate(&env);

    // A stranger (not the admin) signing the call is rejected
    let stranger = Address::generate(&env);
    let result = scoped_add(&env, &client, &contract_id, &stranger, &issuer);
    assert!(result.is_err());
    assert!(!client.is_issuer_approved(&issuer));

    // The admin signing the identical call succeeds
    let result = scoped_add(&env, &client, &contract_id, &admin, &issuer);
    assert!(matches!(result, Ok(Ok(()))));
    assert!(client.is_issuer_approved(&issuer));
}

#[test]
fn test_is_issuer_approved_false_when_not_registered() {
    let (env, _admin, client) = setup();
    let issuer = Address::generate(&env);

    assert!(!client.is_issuer_approved(&issuer));
}
