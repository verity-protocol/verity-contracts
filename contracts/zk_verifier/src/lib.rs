#![no_std]

mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contracterror, contractimpl, Bytes, Env, Vec};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// The submitted proof failed verification.
    InvalidProof = 1,
    /// The proof uses an unsupported elliptic curve.
    UnsupportedCurve = 2,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// ZK Verifier contract — verifies zero-knowledge proofs on-chain.
///
/// This contract verifies Noir-generated ZK proofs using BN254 elliptic
/// curve host functions introduced in Stellar Protocol 25 (X-Ray) and
/// Protocol 26 (Yardstick).
///
/// The verification key is stored at deployment and corresponds to a
/// specific Noir circuit. Each circuit (e.g., "over 18", "passed KYC")
/// has its own verification key.
#[contract]
pub struct ZkVerifier;

#[contractimpl]
impl ZkVerifier {
    /// Verify a ZK proof against the stored verification key.
    ///
    /// # Arguments
    /// * `proof` — The serialized ZK proof bytes (BN254 curve points)
    /// * `public_inputs` — The public inputs to the circuit (e.g., commitment hash,
    ///   nullifier, merkle root). These are verified as part of the proof.
    ///
    /// # Returns
    /// `true` if the proof is valid, `false` otherwise.
    pub fn verify_proof(
        _env: Env,
        _proof: Bytes,
        _public_inputs: Vec<Bytes>,
    ) -> Result<bool, Error> {
        // TODO: Load verification key from instance storage
        // TODO: Deserialize the proof into BN254 curve points
        // TODO: Hash public inputs using Poseidon
        // TODO: Perform pairing check using BN254 host functions
        // TODO: Emit ProofVerified event
        // TODO: Return verification result
        todo!("verify_proof: implement BN254 proof verification")
    }
}
