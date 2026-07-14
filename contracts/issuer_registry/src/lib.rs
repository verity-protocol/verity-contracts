#![no_std]

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
/// ## How it works:
/// 1. Verity onboards a KYC provider (e.g., Smile ID, Persona)
/// 2. Admin calls `add_issuer` to register the provider's Stellar address
/// 3. When the KYC provider verifies a user, they call the Credential contract
/// 4. The Credential contract cross-checks this registry to confirm authorization
/// 5. Admin can revoke a provider's approval via `remove_issuer`
///
/// ## Admin model:
/// The admin is set at deployment (constructor argument). Only the admin
/// can add or remove issuers. This is a permissioned registry — not
/// decentralized — because KYC providers must be vetted and trusted.
#[contract]
pub struct IssuerRegistry;

#[contractimpl]
impl IssuerRegistry {
    /// Initialize the contract with an admin address.
    ///
    /// # Arguments
    /// * `admin` — The Verity backend address that manages issuer approvals
    pub fn __constructor(_env: Env, _admin: Address) {
        // TODO: Store admin address in instance storage
        todo!("__constructor: initialize admin")
    }

    /// Register a new KYC provider as an approved issuer.
    ///
    /// Only callable by the admin. The issuer's Stellar address is stored
    /// and they can then write credentials to the Credential contract.
    ///
    /// # Arguments
    /// * `issuer` — The KYC provider's Stellar address
    pub fn add_issuer(_env: Env, _issuer: Address) -> Result<(), Error> {
        // TODO: Verify caller is admin
        // TODO: Check issuer isn't already approved
        // TODO: Store issuer in persistent storage
        // TODO: Update issuers list
        // TODO: Emit IssuerAdded event
        todo!("add_issuer: implement issuer registration")
    }

    /// Remove a KYC provider from the approved list.
    ///
    /// Only callable by the admin. Existing credentials issued by this
    /// provider remain valid — only future credential issuance is blocked.
    ///
    /// # Arguments
    /// * `issuer` — The KYC provider's Stellar address to remove
    pub fn remove_issuer(_env: Env, _issuer: Address) -> Result<(), Error> {
        // TODO: Verify caller is admin
        // TODO: Check issuer is currently approved
        // TODO: Remove from persistent storage
        // TODO: Update issuers list
        // TODO: Emit IssuerRemoved event
        todo!("remove_issuer: implement issuer removal")
    }

    /// Check whether a KYC provider is currently approved.
    ///
    /// Called by the Credential contract to validate issuer authorization
    /// before accepting a credential write.
    pub fn is_issuer_approved(_env: Env, _issuer: Address) -> bool {
        // TODO: Look up issuer in persistent storage
        // TODO: Return true if approved, false otherwise
        todo!("is_issuer_approved: implement approval check")
    }

    /// Get all approved KYC provider addresses.
    ///
    /// Used by the admin panel to display the list of registered issuers.
    pub fn get_all_issuers(_env: Env) -> Vec<Address> {
        // TODO: Return all approved issuers from persistent storage
        todo!("get_all_issuers: implement issuer listing")
    }
}
