#![no_std]

mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contracterror, contractimpl, BytesN, Env};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// This nullifier has already been used — a DID already exists for this identity.
    NullifierAlreadyExists = 1,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// Nullifier contract — prevents Sybil attacks by ensuring one identity = one DID.
///
/// When a user creates a DID, they must provide a nullifier hash that is
/// derived from their real-world identity (e.g., a Poseidon hash of their
/// document commitment). This hash is stored on-chain to prevent the same
/// person from creating multiple DIDs.
#[contract]
pub struct NullifierContract;

#[contractimpl]
impl NullifierContract {
    /// Store a new nullifier hash.
    ///
    /// Called by the DID Registry after a new DID is created successfully.
    /// The nullifier must not already exist — this is checked by `check_nullifier`
    /// before DID creation proceeds.
    ///
    /// # Arguments
    /// * `nullifier_hash` — Poseidon hash of the user's identity commitment
    pub fn store_nullifier(_env: Env, _nullifier_hash: BytesN<32>) -> Result<(), Error> {
        // TODO: Check nullifier doesn't already exist in persistent storage
        // TODO: Store nullifier_hash → true in persistent storage
        // TODO: Emit NullifierStored event
        todo!("store_nullifier: implement nullifier storage")
    }

    /// Check whether a nullifier hash has already been used.
    ///
    /// Returns `true` if the nullifier exists (identity already has a DID),
    /// `false` if it's new (identity can create a DID).
    ///
    /// # Arguments
    /// * `nullifier_hash` — Poseidon hash of the user's identity commitment
    pub fn check_nullifier(_env: Env, _nullifier_hash: BytesN<32>) -> bool {
        // TODO: Look up nullifier_hash in persistent storage
        // TODO: Return true if exists, false otherwise
        todo!("check_nullifier: implement nullifier lookup")
    }
}
