#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, BytesN, Env, Symbol};

// ---------------------------------------------------------------------------
// Event Types
// ---------------------------------------------------------------------------

/// Emitted when a new nullifier hash is stored.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct NullifierStoredEvent {
    /// The nullifier hash that was stored.
    pub nullifier_hash: BytesN<32>,
}

// ---------------------------------------------------------------------------
// Event Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn publish_nullifier_stored(env: &Env, nullifier_hash: &BytesN<32>) {
    let event = NullifierStoredEvent {
        nullifier_hash: nullifier_hash.clone(),
    };
    env.events()
        .publish((Symbol::new(env, "nullifier_stored"),), event);
}
