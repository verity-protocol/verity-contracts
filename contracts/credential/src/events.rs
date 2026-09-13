#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol};

// ---------------------------------------------------------------------------
// Event Types
// ---------------------------------------------------------------------------

/// Emitted when a credential is issued to a DID.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CredentialIssuedEvent {
    /// The DID identifier receiving the credential.
    pub did: BytesN<32>,
    /// The issuer that issued the credential.
    pub issuer: Address,
    /// The type of credential issued.
    pub credential_type: Symbol,
    /// Ledger timestamp of issuance.
    pub issued_at: u64,
}

/// Emitted when a credential is revoked.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CredentialRevokedEvent {
    /// The DID identifier whose credential was revoked.
    pub did: BytesN<32>,
    /// The issuer that revoked the credential.
    pub issuer: Address,
    /// The type of credential revoked.
    pub credential_type: Symbol,
}

// ---------------------------------------------------------------------------
// Event Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn publish_credential_issued(
    env: &Env,
    did: &BytesN<32>,
    issuer: &Address,
    credential_type: &Symbol,
    issued_at: u64,
) {
    let event = CredentialIssuedEvent {
        did: did.clone(),
        issuer: issuer.clone(),
        credential_type: credential_type.clone(),
        issued_at,
    };
    env.events()
        .publish((Symbol::new(env, "credential_issued"),), event);
}

#[allow(dead_code)]
pub fn publish_credential_revoked(
    env: &Env,
    did: &BytesN<32>,
    issuer: &Address,
    credential_type: &Symbol,
) {
    let event = CredentialRevokedEvent {
        did: did.clone(),
        issuer: issuer.clone(),
        credential_type: credential_type.clone(),
    };
    env.events()
        .publish((Symbol::new(env, "credential_revoked"),), event);
}
