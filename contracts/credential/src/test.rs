#![cfg(test)]

use super::{events, *};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Events, Ledger as _, MockAuth, MockAuthInvoke},
    xdr, Address, BytesN, ConversionError, Env, IntoVal, InvokeError, Symbol, TryFromVal, Val,
};
use verity_did_registry::{DidRegistry, DidRegistryClient};
use verity_issuer_registry::{IssuerRegistry, IssuerRegistryClient};

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

/// Helper: set up a Credential contract backed by a real Issuer Registry and
/// a real DID Registry, with all auths mocked. The returned DID is a real,
/// registered DID.
///
/// Used for behavior tests. The scoped-auth security tests use
/// `setup_scoped` instead so missing signatures actually fail.
fn setup() -> (
    Env,
    BytesN<32>,
    Address,
    Address,
    CredentialContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let owner = Address::generate(&env);

    let registry_id = env.register(IssuerRegistry, (admin.clone(),));
    IssuerRegistryClient::new(&env, &registry_id).add_issuer(&issuer);

    let did_registry_id = env.register(DidRegistry, (admin.clone(),));
    let did = DidRegistryClient::new(&env, &did_registry_id).create_did(&owner);

    let contract_id = env.register(
        CredentialContract,
        (registry_id.clone(), did_registry_id.clone()),
    );
    let client = CredentialContractClient::new(&env, &contract_id);

    (env, did, issuer, registry_id, client)
}

/// Helper: set up a Credential contract WITHOUT `mock_all_auths`.
///
/// The issuer is approved under exactly the admin's signature in the Issuer
/// Registry and the DID is created under exactly the owner's signature in the
/// DID Registry, so tests can prove which signatures `issue_credential` and
/// `revoke_credential` really require — including the on-chain registry
/// sub-calls.
fn setup_scoped() -> (
    Env,
    BytesN<32>,
    Address,
    Address,
    Address,
    Address,
    Address,
    CredentialContractClient<'static>,
) {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let stranger = Address::generate(&env);
    let owner = Address::generate(&env);

    let registry_id = env.register(IssuerRegistry, (admin.clone(),));
    IssuerRegistryClient::new(&env, &registry_id)
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &registry_id,
                fn_name: "add_issuer",
                args: (issuer.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .add_issuer(&issuer);

    let did_registry_id = env.register(DidRegistry, (admin.clone(),));
    let did = DidRegistryClient::new(&env, &did_registry_id)
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &did_registry_id,
                fn_name: "create_did",
                args: (owner.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .create_did(&owner);

    let contract_id = env.register(
        CredentialContract,
        (registry_id.clone(), did_registry_id.clone()),
    );
    let client = CredentialContractClient::new(&env, &contract_id);

    (
        env,
        did,
        issuer,
        stranger,
        contract_id,
        registry_id,
        admin,
        client,
    )
}

/// Authorize `issuer` for `issue_credential`, modeling the Issuer Registry
/// `is_issuer_approved` sub-invocation in the auth tree, and invoke it.
///
/// Both registry sub-calls are read-only (no `require_auth` in the callee),
/// so the DID Registry `get_did` check needs no auth-tree entry.
fn scoped_issue(
    env: &Env,
    client: &CredentialContractClient<'static>,
    contract_id: &Address,
    registry_id: &Address,
    issuer: &Address,
    did: &BytesN<32>,
    credential_type: &Symbol,
    credential_hash: &BytesN<32>,
) -> Result<Result<(), ConversionError>, Result<Error, InvokeError>> {
    client
        .mock_auths(&[MockAuth {
            address: issuer,
            invoke: &MockAuthInvoke {
                contract: contract_id,
                fn_name: "issue_credential",
                args: (
                    issuer.clone(),
                    did.clone(),
                    credential_type.clone(),
                    credential_hash.clone(),
                )
                    .into_val(env),
                sub_invokes: &[MockAuthInvoke {
                    contract: registry_id,
                    fn_name: "is_issuer_approved",
                    args: (issuer.clone(),).into_val(env),
                    sub_invokes: &[],
                }],
            },
        }])
        .try_issue_credential(issuer, did, credential_type, credential_hash)
}

/// Authorize `issuer` for `revoke_credential` and invoke it.
fn scoped_revoke(
    env: &Env,
    client: &CredentialContractClient<'static>,
    contract_id: &Address,
    issuer: &Address,
    did: &BytesN<32>,
    credential_type: &Symbol,
) -> Result<Result<(), ConversionError>, Result<Error, InvokeError>> {
    client
        .mock_auths(&[MockAuth {
            address: issuer,
            invoke: &MockAuthInvoke {
                contract: contract_id,
                fn_name: "revoke_credential",
                args: (issuer.clone(), did.clone(), credential_type.clone()).into_val(env),
                sub_invokes: &[],
            },
        }])
        .try_revoke_credential(issuer, did, credential_type)
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
// Behavior tests (mock_all_auths)
// ---------------------------------------------------------------------------

#[test]
fn test_issue_credential() {
    let (env, did, issuer, _registry_id, client) = setup();
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    client.issue_credential(&issuer, &did, &credential_type, &hash);

    // A CredentialIssued event was published (read before any query calls)
    assert_eq!(
        last_event_first_topic(&env),
        Symbol::new(&env, "credential_issued")
    );
    let issued = events::CredentialIssuedEvent::try_from_val(&env, &last_event_data(&env)).unwrap();
    assert_eq!(issued.did, did);
    assert_eq!(issued.issuer, issuer);
    assert_eq!(issued.credential_type, credential_type);

    // The record is stored with the full issuance details
    let record = client.get_credential(&did, &credential_type).unwrap();
    assert_eq!(record.did, did);
    assert_eq!(record.issuer, issuer);
    assert_eq!(record.credential_type, credential_type);
    assert_eq!(record.credential_hash, hash);
    assert_eq!(record.is_revoked, false);
    assert!(record.issued_at > 0);
    assert_eq!(issued.issued_at, record.issued_at);

    // Third-party validity check answers yes
    assert!(client.is_credential_valid(&did, &credential_type));

    // Indexes are updated for dashboard + audit use
    let did_creds = client.get_credentials_for_did(&did);
    assert_eq!(did_creds.len(), 1);
    assert_eq!(did_creds.get_unchecked(0), credential_type);
    let issuer_creds = client.get_credentials_for_issuer(&issuer);
    assert_eq!(issuer_creds.len(), 1);
    assert_eq!(issuer_creds.get_unchecked(0), credential_type);
}

#[test]
fn test_issue_credential_duplicate_rejected() {
    let (env, did, issuer, _registry_id, client) = setup();
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    client.issue_credential(&issuer, &did, &credential_type, &hash);

    // Same DID + type is refused even with a different hash
    let other_hash = BytesN::<32>::random(&env);
    let result = client.try_issue_credential(&issuer, &did, &credential_type, &other_hash);
    assert!(matches!(result, Err(Ok(Error::CredentialAlreadyExists))));
}

#[test]
fn test_issue_credential_unapproved_issuer() {
    let (env, did, _issuer, _registry_id, client) = setup();
    let bogus = Address::generate(&env); // never added to the registry
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    let result = client.try_issue_credential(&bogus, &did, &credential_type, &hash);
    assert!(matches!(result, Err(Ok(Error::NotApprovedIssuer))));
    assert!(!client.is_credential_valid(&did, &credential_type));
}

#[test]
fn test_issue_credential_unknown_did() {
    let (env, _did, issuer, _registry_id, client) = setup();
    let bogus_did = BytesN::<32>::random(&env); // never registered in the DID Registry
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    // An approved issuer cannot mint a credential against a DID that does not
    // exist on-chain — no orphaned credentials on invented identifiers.
    let result = client.try_issue_credential(&issuer, &bogus_did, &credential_type, &hash);
    assert!(matches!(result, Err(Ok(Error::DidNotFound))));
    assert!(!client.is_credential_valid(&bogus_did, &credential_type));
}

#[test]
fn test_revoke_credential() {
    let (env, did, issuer, _registry_id, client) = setup();
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    client.issue_credential(&issuer, &did, &credential_type, &hash);
    client.revoke_credential(&issuer, &did, &credential_type);

    // A CredentialRevoked event was published (read before any query calls)
    assert_eq!(
        last_event_first_topic(&env),
        Symbol::new(&env, "credential_revoked")
    );
    let revoked =
        events::CredentialRevokedEvent::try_from_val(&env, &last_event_data(&env)).unwrap();
    assert_eq!(revoked.did, did);
    assert_eq!(revoked.issuer, issuer);
    assert_eq!(revoked.credential_type, credential_type);

    // Immutable: the record stays, but the validity signal flips to no
    assert!(!client.is_credential_valid(&did, &credential_type));
    let record = client.get_credential(&did, &credential_type).unwrap();
    assert_eq!(record.is_revoked, true);
}

#[test]
fn test_revoke_credential_not_found() {
    let (env, did, issuer, _registry_id, client) = setup();
    let credential_type = Symbol::new(&env, "kyc_basic");

    let result = client.try_revoke_credential(&issuer, &did, &credential_type);
    assert!(matches!(result, Err(Ok(Error::CredentialNotFound))));
}

#[test]
fn test_revoke_credential_wrong_issuer() {
    let (env, did, issuer, registry_id, client) = setup();

    // A second approved issuer exists
    let other_issuer = Address::generate(&env);
    IssuerRegistryClient::new(&env, &registry_id).add_issuer(&other_issuer);

    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    client.issue_credential(&issuer, &did, &credential_type, &hash);

    // Approved but NOT the original issuer cannot revoke
    let result = client.try_revoke_credential(&other_issuer, &did, &credential_type);
    assert!(matches!(result, Err(Ok(Error::Unauthorized))));
    assert!(client.is_credential_valid(&did, &credential_type));
}

#[test]
fn test_revoke_credential_already_revoked() {
    let (env, did, issuer, _registry_id, client) = setup();
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    client.issue_credential(&issuer, &did, &credential_type, &hash);
    client.revoke_credential(&issuer, &did, &credential_type);

    let result = client.try_revoke_credential(&issuer, &did, &credential_type);
    assert!(matches!(result, Err(Ok(Error::AlreadyRevoked))));
}

#[test]
fn test_queries_for_unknown_credentials() {
    let (env, did, _issuer, _registry_id, client) = setup();
    let credential_type = Symbol::new(&env, "kyc_basic");

    assert!(client.get_credential(&did, &credential_type).is_none());
    assert!(!client.is_credential_valid(&did, &credential_type));
    assert_eq!(client.get_credentials_for_did(&did).len(), 0);
}

// ---------------------------------------------------------------------------
// Security tests (scoped auth, real registry)
// ---------------------------------------------------------------------------

#[test]
fn test_issue_credential_requires_issuer_signature() {
    let (env, did, issuer, _stranger, contract_id, registry_id, _admin, client) = setup_scoped();
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    // No signature at all → host rejects before any contract logic runs
    let result = client.try_issue_credential(&issuer, &did, &credential_type, &hash);
    assert!(result.is_err());

    // Issuer signature + registry sub-invocation present → succeeds
    let result = scoped_issue(
        &env,
        &client,
        &contract_id,
        &registry_id,
        &issuer,
        &did,
        &credential_type,
        &hash,
    );
    assert!(matches!(result, Ok(Ok(()))));
    assert!(client.is_credential_valid(&did, &credential_type));
}

#[test]
fn test_issue_credential_requires_approved_issuer() {
    let (env, did, _issuer, stranger, contract_id, registry_id, _admin, client) = setup_scoped();
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    // Structurally correct call from a non-registered issuer is refused by the
    // contract itself — valid signatures are not enough.
    let result = scoped_issue(
        &env,
        &client,
        &contract_id,
        &registry_id,
        &stranger,
        &did,
        &credential_type,
        &hash,
    );
    assert!(matches!(result, Err(Ok(Error::NotApprovedIssuer))));
    assert!(!client.is_credential_valid(&did, &credential_type));
}

#[test]
fn test_issue_credential_unknown_did_scoped() {
    let (env, _did, issuer, _stranger, contract_id, registry_id, _admin, client) = setup_scoped();
    let bogus_did = BytesN::<32>::random(&env);
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    // Correctly signed call from an approved issuer is still refused when the
    // subject DID does not exist in the DID Registry.
    let result = scoped_issue(
        &env,
        &client,
        &contract_id,
        &registry_id,
        &issuer,
        &bogus_did,
        &credential_type,
        &hash,
    );
    assert!(matches!(result, Err(Ok(Error::DidNotFound))));
    assert!(!client.is_credential_valid(&bogus_did, &credential_type));
}

#[test]
fn test_revoke_credential_requires_issuer_signature() {
    let (env, did, issuer, _stranger, contract_id, registry_id, _admin, client) = setup_scoped();
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    scoped_issue(
        &env,
        &client,
        &contract_id,
        &registry_id,
        &issuer,
        &did,
        &credential_type,
        &hash,
    )
    .unwrap()
    .unwrap();

    // Unauthorized revoke attempt → host rejects
    let result = client.try_revoke_credential(&issuer, &did, &credential_type);
    assert!(result.is_err());
    assert!(client.is_credential_valid(&did, &credential_type));

    // Authorized revoke → succeeds
    let result = scoped_revoke(&env, &client, &contract_id, &issuer, &did, &credential_type);
    assert!(matches!(result, Ok(Ok(()))));
    assert!(!client.is_credential_valid(&did, &credential_type));
}

#[test]
fn test_revoke_credential_wrong_issuer_scoped() {
    let (env, did, issuer, stranger, contract_id, registry_id, _admin, client) = setup_scoped();
    let credential_type = Symbol::new(&env, "kyc_basic");
    let hash = BytesN::<32>::random(&env);

    scoped_issue(
        &env,
        &client,
        &contract_id,
        &registry_id,
        &issuer,
        &did,
        &credential_type,
        &hash,
    )
    .unwrap()
    .unwrap();

    // A stranger with their own (valid) signature reaches the contract but is
    // refused because they are not the original issuer.
    let result = scoped_revoke(
        &env,
        &client,
        &contract_id,
        &stranger,
        &did,
        &credential_type,
    );
    assert!(matches!(result, Err(Ok(Error::Unauthorized))));
    assert!(client.is_credential_valid(&did, &credential_type));
}
