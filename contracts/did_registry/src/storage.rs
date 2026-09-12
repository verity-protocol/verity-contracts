#![allow(dead_code)]

use soroban_sdk::{contracttype, Address, BytesN, Env, Vec};

// ---------------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------------

/// The on-chain identity record for a single DID.
///
/// Each DID is a permanent entry on the Stellar ledger that exists independently
/// of any wallet address. One DID can have multiple wallets linked to it.
///
/// The `did_id` is a deterministic hash derived from the registry contract,
/// the owner wallet, and a salt — it is an identifier, NOT an `Address`, and
/// cannot authenticate anything on its own. All mutations are authorized by the
/// controlling wallet (`owner`).
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DidRecord {
    /// The deterministic identifier for this DID (sha256 of contract + owner + salt).
    pub did_id: BytesN<32>,
    /// The wallet address that originally created this DID.
    /// This wallet is the controller that authorizes DID document mutations.
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

    /// Maps a DID identifier to its full record.
    DidToRecord(BytesN<32>),

    /// Maps a DID identifier to the list of wallets linked to it.
    /// Supports wallet rotation: users can add/remove wallets without losing identity.
    LinkedWallets(BytesN<32>),

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
pub fn get_did_for_wallet(env: &Env, wallet: &Address) -> Option<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::WalletToDid(wallet.clone()))
}

/// Get all wallet addresses linked to a DID.
///
/// Used by the frontend dashboard to show the user all their linked wallets.
pub fn get_linked_wallets(env: &Env, did: &BytesN<32>) -> Vec<Address> {
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
pub fn is_verified(env: &Env, did: &BytesN<32>) -> bool {
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
pub fn get_did_record(env: &Env, did: &BytesN<32>) -> Option<DidRecord> {
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
pub fn set_did_record(env: &Env, did: &BytesN<32>, record: &DidRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::DidToRecord(did.clone()), record);
}

/// Map a wallet address to a DID in storage.
pub fn set_wallet_to_did(env: &Env, wallet: &Address, did: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::WalletToDid(wallet.clone()), did);
}

/// Remove the WalletToDid mapping for a wallet address.
pub fn remove_wallet_to_did(env: &Env, wallet: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::WalletToDid(wallet.clone()));
}

/// Store the linked wallets list for a DID.
pub fn set_linked_wallets(env: &Env, did: &BytesN<32>, wallets: &Vec<Address>) {
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

/// Load the admin address from instance storage.
pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}
