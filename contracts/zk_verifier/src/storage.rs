#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Bytes, Env};

// ---------------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------------

/// Typed storage keys for the ZK Verifier contract.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    /// The verification key for the ZK circuit.
    /// Set at deployment and never changed.
    VerificationKey,
}

// ---------------------------------------------------------------------------
// Storage Helpers
// ---------------------------------------------------------------------------

/// Load the verification key from instance storage.
///
/// TODO: Implement — currently no verification key is stored.
pub fn get_verification_key(_env: &Env) -> Option<Bytes> {
    // TODO: Load from instance storage
    None
}
