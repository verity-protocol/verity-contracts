#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Symbol,
};

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
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// A credential of this type already exists for this DID.
    CredentialAlreadyExists = 1,
    /// No credential of this type exists for this DID.
    CredentialNotFound = 2,
    /// The issuer is not registered in the Issuer Registry contract.
    NotApprovedIssuer = 3,
    /// This credential has already been revoked.
    AlreadyRevoked = 4,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// The Credential contract — stores verified credential hashes on-chain.
///
/// When a KYC provider verifies a user's identity off-chain, they issue
/// a credential by writing its hash to this contract. The hash proves
/// verification happened without revealing any document details.
///
/// Only approved issuers (registered in the Issuer Registry) can write
/// credentials. The Verity backend validates issuer authorization before
/// calling this contract.
#[contract]
pub struct CredentialContract;

#[contractimpl]
impl CredentialContract {
    /// Initialize the contract.
    ///
    /// Called once at deployment. Currently no admin is needed —
    /// issuer authorization is checked against the Issuer Registry contract.
    pub fn __constructor(_env: Env) {}

    /// Issue a verified credential to a DID.
    ///
    /// Called by the Verity backend after a KYC provider confirms identity
    /// and the document has been permanently deleted.
    ///
    /// Flow:
    /// 1. Validate the issuer is registered in the Issuer Registry
    /// 2. Check no credential of this type already exists for this DID
    /// 3. Store the CredentialRecord
    /// 4. Update DidCredentials index
    /// 5. Update IssuerCredentials index
    /// 6. Emit CredentialIssued event
    ///
    /// # Arguments
    /// * `issuer` - The KYC provider's address (must be in Issuer Registry)
    /// * `did` - The DID receiving the credential
    /// * `credential_type` - Type of credential (e.g., "kyc_basic")
    /// * `credential_hash` - SHA-256 hash of the verified credential data
    pub fn issue_credential(
        _env: Env,
        _issuer: Address,
        _did: Address,
        _credential_type: Symbol,
        _credential_hash: BytesN<32>,
    ) -> Result<(), Error> {
        // TODO: Verify issuer is approved (cross-call to issuer_registry contract)
        // TODO: Check credential doesn't already exist for this DID + type
        // TODO: Build CredentialRecord with current ledger timestamp
        // TODO: Store in persistent storage
        // TODO: Update DidCredentials and IssuerCredentials indexes
        // TODO: Emit CredentialIssued event
        todo!("issue_credential: implement credential issuance")
    }

    /// Revoke a previously issued credential.
    ///
    /// Only the original issuer can revoke their own credential.
    /// Revocation is immutable — once revoked, a credential cannot be re-enabled.
    ///
    /// # Arguments
    /// * `issuer` - Must be the original issuer of the credential
    /// * `did` - The DID whose credential is being revoked
    /// * `credential_type` - The type of credential to revoke
    pub fn revoke_credential(
        _env: Env,
        _issuer: Address,
        _did: Address,
        _credential_type: Symbol,
    ) -> Result<(), Error> {
        // TODO: Load credential record
        // TODO: Verify caller is the original issuer
        // TODO: Check credential isn't already revoked
        // TODO: Set is_revoked = true
        // TODO: Emit CredentialRevoked event
        todo!("revoke_credential: implement credential revocation")
    }

    /// Read a credential record for a DID and credential type.
    ///
    /// Returns None if no credential of this type exists.
    pub fn get_credential(
        _env: Env,
        _did: Address,
        _credential_type: Symbol,
    ) -> Option<CredentialRecord> {
        // TODO: Look up Credential(Did, type) in persistent storage
        todo!("get_credential: implement credential lookup")
    }

    /// Check whether a credential is valid (exists and is not revoked).
    ///
    /// This is the method third-party apps ultimately rely on — the
    /// backend resolves a DID and checks its credential validity to
    /// return the yes/no verified signal.
    pub fn is_credential_valid(_env: Env, _did: Address, _credential_type: Symbol) -> bool {
        // TODO: Load credential, return true if exists and is_revoked == false
        todo!("is_credential_valid: implement validity check")
    }
}
