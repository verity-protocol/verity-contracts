#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Address, Env, Vec};

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
    /// List of all approved issuer addresses, in registration order.
    Issuers,
}

// ---------------------------------------------------------------------------
// Storage Helpers
// ---------------------------------------------------------------------------

/// The admin address that manages issuer approvals.
pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

/// Check whether an issuer is approved.
pub fn is_issuer_approved(env: &Env, issuer: &Address) -> bool {
    env.storage()
        .persistent()
        .get::<_, bool>(&DataKey::IsApproved(issuer.clone()))
        .unwrap_or(false)
}

/// Set the approval status of an issuer.
pub fn set_issuer_approved(env: &Env, issuer: &Address, approved: bool) {
    env.storage()
        .persistent()
        .set(&DataKey::IsApproved(issuer.clone()), &approved);
}

/// Append an issuer to the ordered list of approved issuers.
///
/// The list keeps registration order for the admin panel's issuer listing.
pub fn add_to_issuers(env: &Env, issuer: &Address) {
    let mut issuers: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Issuers)
        .unwrap_or_else(|| Vec::new(env));
    issuers.push_back(issuer.clone());
    env.storage().persistent().set(&DataKey::Issuers, &issuers);
}
