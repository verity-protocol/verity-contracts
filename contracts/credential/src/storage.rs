#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol, Vec};

// ---------------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------------

/// A verified credential issued to a DID by a trusted KYC provider.
///
/// Only the credential hash is stored on-chain — never the raw document.
/// The hash is a SHA-256 digest of the credential data that the KYC provider
/// verified off-chain. Documents are permanently deleted immediately after
/// verification.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CredentialRecord {
    /// The DID this credential belongs to.
    pub did: BytesN<32>,
    /// The KYC provider (issuer) that verified and issued this credential.
    /// Must be registered in the Issuer Registry contract.
    pub issuer: Address,
    /// The type of credential (e.g., "kyc_basic", "accredited_investor").
    /// Used for selective disclosure in future versions.
    pub credential_type: Symbol,
    /// SHA-256 hash of the verified credential data.
    /// The raw document is never stored — only this hash persists on-chain.
    pub credential_hash: BytesN<32>,
    /// Ledger timestamp when the credential was issued.
    pub issued_at: u64,
    /// Whether this credential has been revoked by its issuer.
    pub is_revoked: bool,
}

/// Typed storage keys for the Credential contract.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    /// Address of the Issuer Registry contract this contract trusts.
    /// Stored in instance storage at deployment.
    IssuerRegistry,
    /// Address of the DID Registry contract this contract validates DIDs
    /// against. Stored in instance storage at deployment.
    DidRegistry,
    /// Maps (DID, credential_type) to the CredentialRecord.
    /// This is the primary storage — one credential per type per DID.
    Credential(BytesN<32>, Symbol),
    /// Maps an issuer address to the list of credential types it has issued.
    /// Used for issuer analytics and audit trails.
    IssuerCredentials(Address),
    /// Maps a DID to the list of credential types it holds.
    /// Used by the frontend dashboard to show all credentials for a user.
    DidCredentials(BytesN<32>),
}

// ---------------------------------------------------------------------------
// Storage Helpers
// ---------------------------------------------------------------------------

/// Configure the Issuer Registry contract this contract trusts.
///
/// Called once from `__constructor`.
pub fn set_issuer_registry(env: &Env, address: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::IssuerRegistry, address);
}

/// The Issuer Registry contract this contract cross-calls for approval checks.
pub fn get_issuer_registry(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::IssuerRegistry)
        .unwrap()
}

/// Configure the DID Registry contract this contract validates DIDs against.
///
/// Called once from `__constructor`.
pub fn set_did_registry(env: &Env, address: &Address) {
    env.storage().instance().set(&DataKey::DidRegistry, address);
}

/// The DID Registry contract this contract cross-calls for DID existence checks.
pub fn get_did_registry(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::DidRegistry).unwrap()
}

/// Load a credential record for a DID and credential type.
pub fn get_credential(
    env: &Env,
    did: &BytesN<32>,
    credential_type: &Symbol,
) -> Option<CredentialRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Credential(did.clone(), credential_type.clone()))
}

/// Store a credential record for a DID and credential type.
pub fn set_credential(
    env: &Env,
    did: &BytesN<32>,
    credential_type: &Symbol,
    record: &CredentialRecord,
) {
    env.storage().persistent().set(
        &DataKey::Credential(did.clone(), credential_type.clone()),
        record,
    );
}

/// Check whether a credential exists and is not revoked.
pub fn is_credential_valid(env: &Env, did: &BytesN<32>, credential_type: &Symbol) -> bool {
    match get_credential(env, did, credential_type) {
        Some(cred) => !cred.is_revoked,
        None => false,
    }
}

/// Append a credential type to a DID's credential index.
pub fn push_did_credential(env: &Env, did: &BytesN<32>, credential_type: &Symbol) {
    let mut types: Vec<Symbol> = env
        .storage()
        .persistent()
        .get(&DataKey::DidCredentials(did.clone()))
        .unwrap_or_else(|| Vec::new(env));
    types.push_back(credential_type.clone());
    env.storage()
        .persistent()
        .set(&DataKey::DidCredentials(did.clone()), &types);
}

/// Append a credential type to an issuer's credential index.
pub fn push_issuer_credential(env: &Env, issuer: &Address, credential_type: &Symbol) {
    let mut types: Vec<Symbol> = env
        .storage()
        .persistent()
        .get(&DataKey::IssuerCredentials(issuer.clone()))
        .unwrap_or_else(|| Vec::new(env));
    types.push_back(credential_type.clone());
    env.storage()
        .persistent()
        .set(&DataKey::IssuerCredentials(issuer.clone()), &types);
}

/// All credential types a DID currently holds (empty if none).
pub fn get_credentials_for_did(env: &Env, did: &BytesN<32>) -> Vec<Symbol> {
    env.storage()
        .persistent()
        .get(&DataKey::DidCredentials(did.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

/// All credential types an issuer has issued (empty if none).
pub fn get_credentials_for_issuer(env: &Env, issuer: &Address) -> Vec<Symbol> {
    env.storage()
        .persistent()
        .get(&DataKey::IssuerCredentials(issuer.clone()))
        .unwrap_or_else(|| Vec::new(env))
}
