#![no_std]

mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contracterror, contractimpl, Address, Env, Vec};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// The issuer is already registered.
    AlreadyApproved = 1,
    /// The issuer is not registered.
    NotApproved = 2,
    /// Only the admin can manage issuers.
    Unauthorized = 3,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// Issuer Registry contract — manages approved KYC providers.
///
/// Only issuers registered in this contract can write credentials to the
/// Credential contract. The admin (Verity backend) controls which KYC
/// providers are approved.
///
/// ## Status
///
/// Implemented: admin registration, `add_issuer`, and the `is_issuer_approved`
/// lookup that the Credential contract cross-calls on-chain.
///
/// Open contributor work: `remove_issuer`, `get_all_issuers`, and a full
/// issuer-registry test suite (currently only the implemented functions are
/// tested).
#[contract]
pub struct IssuerRegistry;

#[contractimpl]
impl IssuerRegistry {
    /// Initialize the contract with an admin address.
    ///
    /// # Arguments
    /// * `admin` — The Verity backend address that manages issuer approvals
    pub fn __constructor(env: Env, admin: Address) {
        env.storage()
            .instance()
            .set(&storage::DataKey::Admin, &admin);
    }

    /// Register a new KYC provider as an approved issuer.
    ///
    /// Only callable by the admin. The issuer's Stellar address is stored
    /// and they can then write credentials to the Credential contract.
    ///
    /// # Arguments
    /// * `issuer` — The KYC provider's Stellar address
    pub fn add_issuer(env: Env, issuer: Address) -> Result<(), Error> {
        storage::get_admin(&env).require_auth();

        if storage::is_issuer_approved(&env, &issuer) {
            return Err(Error::AlreadyApproved);
        }

        storage::set_issuer_approved(&env, &issuer, true);
        storage::add_to_issuers(&env, &issuer);
        events::publish_issuer_added(&env, &issuer);

        Ok(())
    }

    /// Remove a KYC provider from the approved list.
    ///
    /// Only callable by the admin. Existing credentials issued by this
    /// provider remain valid — only future credential issuance is blocked.
    ///
    /// # Arguments
    /// * `issuer` — The KYC provider's Stellar address to remove
    pub fn remove_issuer(_env: Env, _issuer: Address) -> Result<(), Error> {
        // TODO(contributor): implement issuer removal
        // 1. Verify caller is admin
        // 2. Check issuer is currently approved
        // 3. Remove from persistent storage
        // 4. Update issuers list
        // 5. Emit IssuerRemoved event
        todo!("remove_issuer: implement issuer removal")
    }

    /// Check whether a KYC provider is currently approved.
    ///
    /// Called by the Credential contract to validate issuer authorization
    /// before accepting a credential write.
    pub fn is_issuer_approved(env: Env, issuer: Address) -> bool {
        storage::is_issuer_approved(&env, &issuer)
    }

    /// Get all approved KYC provider addresses.
    ///
    /// Used by the admin panel to display the list of registered issuers.
    pub fn get_all_issuers(_env: Env) -> Vec<Address> {
        // TODO(contributor): implement issuer listing
        // Return all approved issuers from the `Issuers` storage list.
        todo!("get_all_issuers: implement issuer listing")
    }
}
