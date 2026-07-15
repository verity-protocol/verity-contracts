#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Event Types
// ---------------------------------------------------------------------------

/// Emitted when a new DID is created.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DidCreatedEvent {
    /// The newly created DID address.
    pub did_address: Address,
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
    pub did_address: Address,
    /// The wallet address that was linked.
    pub wallet_address: Address,
}

/// Emitted when a wallet is unlinked from a DID.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct WalletUnlinkedEvent {
    /// The DID the wallet was unlinked from.
    pub did_address: Address,
    /// The wallet address that was unlinked.
    pub wallet_address: Address,
}

/// Emitted when a DID's verification status changes.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VerificationStatusChangedEvent {
    /// The DID whose status changed.
    pub did_address: Address,
    /// The new verification status.
    pub is_verified: bool,
}

// ---------------------------------------------------------------------------
// Event Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn publish_did_created(env: &Env, did: &Address, owner: &Address, created_at: u64) {
    let event = DidCreatedEvent {
        did_address: did.clone(),
        owner: owner.clone(),
        created_at,
    };
    env.events()
        .publish((Symbol::new(env, "did_created"),), event);
}

/// Publish a WalletLinked event.
pub fn publish_wallet_linked(env: &Env, did: &Address, wallet: &Address) {
    let event = WalletLinkedEvent {
        did_address: did.clone(),
        wallet_address: wallet.clone(),
    };
    env.events()
        .publish((Symbol::new(env, "wallet_linked"),), event);
}

/// Publish a WalletUnlinked event.
pub fn publish_wallet_unlinked(env: &Env, did: &Address, wallet: &Address) {
    let event = WalletUnlinkedEvent {
        did_address: did.clone(),
        wallet_address: wallet.clone(),
    };
    env.events()
        .publish((Symbol::new(env, "wallet_unlinked"),), event);
}

/// Publish a VerificationStatusChanged event.
pub fn publish_verification_status_changed(env: &Env, did: &Address, is_verified: bool) {
    let event = VerificationStatusChangedEvent {
        did_address: did.clone(),
        is_verified,
    };
    env.events()
        .publish((Symbol::new(env, "verification_status_changed"),), event);
}
