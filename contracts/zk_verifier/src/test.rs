#![cfg(test)]

use super::*;
use soroban_sdk::{Bytes, Env, Vec};

/// Helper: set up a fresh ZkVerifier contract and return the env and client.
fn setup() -> (Env, ZkVerifierClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(ZkVerifier, ());
    let client = ZkVerifierClient::new(&env, &contract_id);

    (env, client)
}

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_verify_proof() {
    let (env, client) = setup();
    let proof = Bytes::from_array(&env, &[0u8; 32]);
    let public_inputs = Vec::new(&env);

    client.verify_proof(&proof, &public_inputs);
}
