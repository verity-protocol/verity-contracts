#![allow(dead_code)]

use soroban_sdk::{contracttype, Address, Env, Vec};

// ---------------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------------

/// The on-chain identity record for a single DID.
///
/// Each DID is a permanent entry on the Stellar ledger that exists independently
/// of any wallet address. One DID can have multiple wallets linked to it.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DidRecord {
    /// The address that identifies this DID on-chain.
    pub did_address: Address,
    /// The wallet address that originally created this DID.
    pub owner: Address,
    /// Ledger timestamp when the DID was created.
    pub created_at: u64,
    /// Whether a trusted KYC provider has verified this DID.
    /// Set to true after a credential is issued by an approved issuer.
    pub is_verified: bool,
}

/// Typed storage keys for the DID Registry contract.
///
/// Soroban uses typed enums for storage keys to prevent key collisions.
/// Each variant maps to a specific piece of on-chain state.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    /// Maps a wallet address to the DID it is linked to.
    /// This is the primary lookup — a wallet owner calls this to find their DID.
    WalletToDid(Address),

    /// Maps a DID address to its full record.
    DidToRecord(Address),

    /// Maps a DID address to the list of wallets linked to it.
    /// Supports wallet rotation: users can add/remove wallets without losing identity.
    LinkedWallets(Address),

    /// Global counter of total DIDs created. Used for indexing and analytics.
    DidCount,

    /// The admin address that pays base reserve fees for DID creation.
    /// This is the Verity backend — users never pay for DID creation.
    Admin,
}

// ---------------------------------------------------------------------------
// Storage Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
///
/// Returns None if the wallet has no DID.
pub fn get_did_for_wallet(env: &Env, wallet: &Address) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::WalletToDid(wallet.clone()))
}

/// Get all wallet addresses linked to a DID.
///
/// Used by the frontend dashboard to show the user all their linked wallets.
pub fn get_linked_wallets(env: &Env, did: &Address) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::LinkedWallets(did.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

/// Check whether a DID has been verified by a trusted KYC provider.
///
/// Third-party apps call this (via the backend DID Resolution API)
/// to check a user's verification status. They never see wallet addresses
/// or personal details — only this boolean.
pub fn is_verified(env: &Env, did: &Address) -> bool {
    let record: Option<DidRecord> = env
        .storage()
        .persistent()
        .get(&DataKey::DidToRecord(did.clone()));

    match record {
        Some(r) => r.is_verified,
        None => false,
    }
}

/// Load the full DidRecord for a DID.
///
/// Returns None if the DID does not exist.
pub fn get_did_record(env: &Env, did: &Address) -> Option<DidRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::DidToRecord(did.clone()))
}

/// Get the current total count of DIDs created.
pub fn get_did_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::DidCount)
        .unwrap_or(0)
}

/// Store a DidRecord in persistent storage.
pub fn set_did_record(env: &Env, did: &Address, record: &DidRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::DidToRecord(did.clone()), record);
}

/// Map a wallet address to a DID in storage.
pub fn set_wallet_to_did(env: &Env, wallet: &Address, did: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::WalletToDid(wallet.clone()), did);
}

/// Store the linked wallets list for a DID.
pub fn set_linked_wallets(env: &Env, did: &Address, wallets: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DataKey::LinkedWallets(did.clone()), wallets);
}

/// Increment the global DID count and return the new value.
pub fn increment_did_count(env: &Env) -> u32 {
    let current = get_did_count(env);
    let new_count = current + 1;
    env.storage().instance().set(&DataKey::DidCount, &new_count);
    new_count
}
