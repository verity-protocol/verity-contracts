#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol};

// ---------------------------------------------------------------------------
// Event Types
// ---------------------------------------------------------------------------

/// Emitted when a new DID is created.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DidCreatedEvent {
    /// The newly created DID identifier.
    pub did_id: BytesN<32>,
    /// The wallet that created the DID.
    pub owner: Address,
    /// Ledger timestamp of creation.
    pub created_at: u64,
}

/// Emitted when a wallet is linked to a DID.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct WalletLinkedEvent {
    /// The DID the wallet was linked to.
    pub did_id: BytesN<32>,
    /// The wallet address that was linked.
    pub wallet_address: Address,
}

/// Emitted when a wallet is unlinked from a DID.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct WalletUnlinkedEvent {
    /// The DID the wallet was unlinked from.
    pub did_id: BytesN<32>,
    /// The wallet address that was unlinked.
    pub wallet_address: Address,
}

/// Emitted when a DID's verification status changes.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VerificationStatusChangedEvent {
    /// The DID whose status changed.
    pub did_id: BytesN<32>,
    /// The new verification status.
    pub is_verified: bool,
}

// ---------------------------------------------------------------------------
// Event Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn publish_did_created(env: &Env, did: &BytesN<32>, owner: &Address, created_at: u64) {
    let event = DidCreatedEvent {
        did_id: did.clone(),
        owner: owner.clone(),
        created_at,
    };
    env.events()
        .publish((Symbol::new(env, "did_created"),), event);
}

/// Publish a WalletLinked event.
pub fn publish_wallet_linked(env: &Env, did: &BytesN<32>, wallet: &Address) {
    let event = WalletLinkedEvent {
        did_id: did.clone(),
        wallet_address: wallet.clone(),
    };
    env.events()
        .publish((Symbol::new(env, "wallet_linked"),), event);
}

/// Publish a WalletUnlinked event.
pub fn publish_wallet_unlinked(env: &Env, did: &BytesN<32>, wallet: &Address) {
    let event = WalletUnlinkedEvent {
        did_id: did.clone(),
        wallet_address: wallet.clone(),
    };
    env.events()
        .publish((Symbol::new(env, "wallet_unlinked"),), event);
}

/// Publish a VerificationStatusChanged event.
pub fn publish_verification_status_changed(env: &Env, did: &BytesN<32>, is_verified: bool) {
    let event = VerificationStatusChangedEvent {
        did_id: did.clone(),
        is_verified,
    };
    env.events()
        .publish((Symbol::new(env, "verification_status_changed"),), event);
}
