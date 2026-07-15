#![no_std]

mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contracterror, contractimpl, Address, BytesN, Env, Symbol};

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
        env: Env,
        did: Address,
        credential_type: Symbol,
    ) -> Option<storage::CredentialRecord> {
        storage::get_credential(&env, &did, &credential_type)
    }

    /// Check whether a credential is valid (exists and is not revoked).
    ///
    /// This is the method third-party apps ultimately rely on — the
    /// backend resolves a DID and checks its credential validity to
    /// return the yes/no verified signal.
    pub fn is_credential_valid(env: Env, did: Address, credential_type: Symbol) -> bool {
        storage::is_credential_valid(&env, &did, &credential_type)
    }
}
