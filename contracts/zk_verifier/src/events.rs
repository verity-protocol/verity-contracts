#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Env, Symbol};

// ---------------------------------------------------------------------------
// Event Types
// ---------------------------------------------------------------------------

/// Emitted when a ZK proof is verified.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ProofVerifiedEvent {
    /// Whether the proof was valid.
    pub result: bool,
}

// ---------------------------------------------------------------------------
// Event Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn publish_proof_verified(env: &Env, result: bool) {
    let event = ProofVerifiedEvent { result };
    env.events()
        .publish((Symbol::new(env, "proof_verified"),), event);
}
