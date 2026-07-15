#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol};

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
    pub did: Address,
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
    /// Maps (DID, credential_type) to the CredentialRecord.
    /// This is the primary storage — one credential per type per DID.
    Credential(Address, Symbol),

    /// Maps an issuer address to the list of credential types it has issued.
    /// Used for issuer analytics and audit trails.
    IssuerCredentials(Address),

    /// Maps a DID to the list of credential types it holds.
    /// Used by the frontend dashboard to show all credentials for a user.
    DidCredentials(Address),
}

// ---------------------------------------------------------------------------
// Storage Helpers
// ---------------------------------------------------------------------------

/// Load a credential record for a DID and credential type.
pub fn get_credential(
    env: &Env,
    did: &Address,
    credential_type: &Symbol,
) -> Option<CredentialRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Credential(did.clone(), credential_type.clone()))
}

/// Check whether a credential exists and is not revoked.
pub fn is_credential_valid(env: &Env, did: &Address, credential_type: &Symbol) -> bool {
    match get_credential(env, did, credential_type) {
        Some(cred) => !cred.is_revoked,
        None => false,
    }
}
