#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, BytesN, Env};

// ---------------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------------

/// Typed storage keys for the Nullifier contract.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    /// Maps a nullifier hash to a boolean indicating it has been used.
    Nullifier(BytesN<32>),
}

// ---------------------------------------------------------------------------
// Storage Helpers
// ---------------------------------------------------------------------------

/// Check whether a nullifier hash has already been stored.
#[allow(dead_code)]
pub fn nullifier_exists(env: &Env, hash: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .get::<_, bool>(&DataKey::Nullifier(hash.clone()))
        .unwrap_or(false)
}

/// Store a nullifier hash.
#[allow(dead_code)]
pub fn store_nullifier(env: &Env, hash: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::Nullifier(hash.clone()), &true);
}
