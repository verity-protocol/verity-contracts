#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Address, Env};

// ---------------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------------

/// Typed storage keys for the Issuer Registry contract.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    /// The admin address that manages issuer approvals.
    Admin,
    /// Maps an issuer address to a boolean indicating approval status.
    IsApproved(Address),
    /// List of all approved issuer addresses.
    Issuers,
}

// ---------------------------------------------------------------------------
// Storage Helpers
// ---------------------------------------------------------------------------

/// Check whether an issuer is approved.
#[allow(dead_code)]
pub fn is_issuer_approved(env: &Env, issuer: &Address) -> bool {
    env.storage()
        .persistent()
        .get::<_, bool>(&DataKey::IsApproved(issuer.clone()))
        .unwrap_or(false)
}
