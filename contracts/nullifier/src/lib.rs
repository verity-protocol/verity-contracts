#![no_std]

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
///
/// ## How it works:
/// 1. User generates a nullifier hash off-chain from their identity commitment
/// 2. On DID creation, the nullifier is checked against this contract
/// 3. If the nullifier already exists, DID creation is rejected
/// 4. If the nullifier is new, it is stored and DID creation proceeds
///
/// ## Privacy note:
/// The nullifier hash is a Poseidon hash — it cannot be reversed to reveal
/// the original document or identity data. It only proves "this identity
/// has already been used" without revealing which identity.
///
/// ## Cross-contract integration:
/// The DID Registry contract calls `check_nullifier` before creating a DID,
/// and `store_nullifier` after successful creation.
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
