#![no_std]

mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contracterror, contractimpl, Address, BytesN, Env, Symbol, Vec};
use verity_did_registry::DidRegistryClient;
use verity_issuer_registry::IssuerRegistryClient;

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
    /// The caller is not the original issuer of this credential.
    Unauthorized = 5,
    /// The DID being issued to does not exist in the DID Registry contract.
    DidNotFound = 6,
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
/// Issuance is permissioned on-chain: the issuing KYC provider must be an
/// approved issuer in the Issuer Registry contract. The approval check is a
/// cross-contract call that cannot be skipped by callers.
#[contract]
pub struct CredentialContract;

#[contractimpl]
impl CredentialContract {
    /// Initialize the contract with the Issuer Registry it trusts and the
    /// DID Registry it validates DIDs against.
    ///
    /// Both references are pinned at deployment and are immutable for the
    /// life of this contract — no admin or later call can redirect which
    /// registries are trusted. Called once at contract deployment.
    ///
    /// # Arguments
    /// * `issuer_registry` — Address of the Issuer Registry contract (permissions issuance)
    /// * `did_registry` — Address of the DID Registry contract (validates DIDs exist)
    pub fn __constructor(env: Env, issuer_registry: Address, did_registry: Address) {
        storage::set_issuer_registry(&env, &issuer_registry);
        storage::set_did_registry(&env, &did_registry);
    }

    /// Issue a verified credential to a DID.
    ///
    /// Called by a KYC provider after it confirms identity and the document
    /// has been permanently deleted.
    ///
    /// Flow:
    /// 1. Require the issuer to authorize the call
    /// 2. Cross-call the Issuer Registry: reject unapproved issuers
    /// 3. Cross-call the DID Registry: reject DIDs that do not exist
    /// 4. Reject if a credential of this type already exists for the DID
    /// 5. Store the CredentialRecord with the current ledger timestamp
    /// 6. Update DidCredentials and IssuerCredentials indexes
    /// 7. Emit CredentialIssued event
    ///
    /// # Arguments
    /// * `issuer` - The KYC provider's address (must be in Issuer Registry)
    /// * `did` - The DID identifier receiving the credential
    /// * `credential_type` - Type of credential (e.g., "kyc_basic")
    /// * `credential_hash` - SHA-256 hash of the verified credential data
    pub fn issue_credential(
        env: Env,
        issuer: Address,
        did: BytesN<32>,
        credential_type: Symbol,
        credential_hash: BytesN<32>,
    ) -> Result<(), Error> {
        issuer.require_auth();

        let registry = IssuerRegistryClient::new(&env, &storage::get_issuer_registry(&env));
        if !registry.is_issuer_approved(&issuer) {
            return Err(Error::NotApprovedIssuer);
        }

        // The subject must be a real DID. Check the primary record in the
        // DID Registry directly — none is returned for DIDs that were never
        // created, so no credentials can be minted against invented IDs.
        let did_registry = DidRegistryClient::new(&env, &storage::get_did_registry(&env));
        if did_registry.get_did(&did).is_none() {
            return Err(Error::DidNotFound);
        }

        if storage::get_credential(&env, &did, &credential_type).is_some() {
            return Err(Error::CredentialAlreadyExists);
        }

        let issued_at = env.ledger().timestamp();
        let record = storage::CredentialRecord {
            did: did.clone(),
            issuer: issuer.clone(),
            credential_type: credential_type.clone(),
            credential_hash: credential_hash.clone(),
            issued_at,
            is_revoked: false,
        };
        storage::set_credential(&env, &did, &credential_type, &record);
        storage::push_did_credential(&env, &did, &credential_type);
        storage::push_issuer_credential(&env, &issuer, &credential_type);
        events::publish_credential_issued(&env, &did, &issuer, &credential_type, issued_at);

        Ok(())
    }

    /// Revoke a previously issued credential.
    ///
    /// Only the original issuer can revoke their own credential.
    /// Revocation is immutable — once revoked, a credential cannot be
    /// re-enabled.
    ///
    /// # Arguments
    /// * `issuer` - Must be the original issuer of the credential
    /// * `did` - The DID whose credential is being revoked
    /// * `credential_type` - The type of credential to revoke
    pub fn revoke_credential(
        env: Env,
        issuer: Address,
        did: BytesN<32>,
        credential_type: Symbol,
    ) -> Result<(), Error> {
        issuer.require_auth();

        let mut record = storage::get_credential(&env, &did, &credential_type)
            .ok_or(Error::CredentialNotFound)?;

        if record.issuer != issuer {
            return Err(Error::Unauthorized);
        }
        if record.is_revoked {
            return Err(Error::AlreadyRevoked);
        }

        record.is_revoked = true;
        storage::set_credential(&env, &did, &credential_type, &record);
        events::publish_credential_revoked(&env, &did, &issuer, &credential_type);

        Ok(())
    }

    /// Read a credential record for a DID and credential type.
    ///
    /// Returns None if no credential of this type exists.
    pub fn get_credential(
        env: Env,
        did: BytesN<32>,
        credential_type: Symbol,
    ) -> Option<storage::CredentialRecord> {
        storage::get_credential(&env, &did, &credential_type)
    }

    /// Check whether a credential is valid (exists and is not revoked).
    ///
    /// This is the method third-party apps ultimately rely on — the
    /// backend resolves a DID and checks its credential validity to
    /// return the yes/no verified signal.
    pub fn is_credential_valid(env: Env, did: BytesN<32>, credential_type: Symbol) -> bool {
        storage::is_credential_valid(&env, &did, &credential_type)
    }

    /// Get all credential types a DID holds.
    ///
    /// Used by the frontend dashboard to render a user's credential list.
    pub fn get_credentials_for_did(env: Env, did: BytesN<32>) -> Vec<Symbol> {
        storage::get_credentials_for_did(&env, &did)
    }

    /// Get all credential types an issuer has issued.
    ///
    /// Used for issuer analytics and audit trails.
    pub fn get_credentials_for_issuer(env: Env, issuer: Address) -> Vec<Symbol> {
        storage::get_credentials_for_issuer(&env, &issuer)
    }
}
