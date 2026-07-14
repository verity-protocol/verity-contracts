#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

// ---------------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------------

/// The on-chain identity record for a single DID.
///
/// Each DID is a permanent entry on the Stellar ledger that exists independently
/// of any wallet address. One DID can have multiple wallets linked to it.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DidRecord {
    /// The address that identifies this DID on-chain.
    pub did_address: Address,
    /// The wallet address that originally created this DID.
    pub owner: Address,
    /// Ledger timestamp when the DID was created.
    pub created_at: u64,
    /// Whether a trusted KYC provider has verified this DID.
    /// Set to true after a credential is issued by an approved issuer.
    pub is_verified: bool,
}

/// Typed storage keys for the DID Registry contract.
///
/// Soroban uses typed enums for storage keys to prevent key collisions.
/// Each variant maps to a specific piece of on-chain state.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    /// Maps a wallet address to the DID it is linked to.
    /// This is the primary lookup — a wallet owner calls this to find their DID.
    WalletToDid(Address),

    /// Maps a DID address to its full record.
    DidToRecord(Address),

    /// Maps a DID address to the list of wallets linked to it.
    /// Supports wallet rotation: users can add/remove wallets without losing identity.
    LinkedWallets(Address),

    /// Global counter of total DIDs created. Used for indexing and analytics.
    DidCount,

    /// The admin address that pays base reserve fees for DID creation.
    /// This is the Verity backend — users never pay for DID creation.
    Admin,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// The wallet address already has a DID linked to it.
    DidAlreadyExists = 1,
    /// The requested DID does not exist on-chain.
    DidNotFound = 2,
    /// The caller is not the owner of this DID.
    NotOwner = 3,
    /// The wallet is already linked to this DID.
    WalletAlreadyLinked = 4,
    /// The wallet is not linked to this DID.
    WalletNotLinked = 5,
    /// Cannot remove the last wallet from a DID. A DID must have at least one wallet.
    CannotRemoveLastWallet = 6,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// The DID Registry contract — foundation of the Verity identity protocol.
///
/// Creates and manages Decentralized Identifiers (DIDs) on Stellar.
/// A DID is a permanent identity record that can have multiple wallet addresses
/// linked to it. If a wallet is lost, the user links a new wallet to their
/// existing DID — identity is never lost.
#[contract]
pub struct DidRegistry;

#[contractimpl]
impl DidRegistry {
    /// Initialize the contract with an admin address.
    ///
    /// The admin pays base reserve fees when creating DIDs on behalf of users.
    /// This is typically the Verity backend server keypair.
    ///
    /// Called once at contract deployment.
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::DidCount, &0u32);
    }

    /// Create a new DID and link the caller's wallet to it.
    ///
    /// Flow:
    /// 1. Check that the caller's wallet doesn't already have a DID
    /// 2. Generate a new DID address (the contract instance address or a derived key)
    /// 3. Store the DidRecord with the caller as owner
    /// 4. Link the caller's wallet to the DID in WalletToDid mapping
    /// 5. Add the wallet to the DID's LinkedWallets list
    /// 6. Increment DidCount
    /// 7. Emit a DidCreated event
    ///
    /// Returns the DID address that was created.
    ///
    /// # Arguments
    /// * `owner` - The wallet address creating the DID. Must authorize this call.
    pub fn create_did(_env: Env, owner: Address) -> Result<Address, Error> {
        owner.require_auth();

        // TODO: Check owner doesn't already have a DID
        // TODO: Generate DID address
        // TODO: Store DidRecord
        // TODO: Map wallet → DID
        // TODO: Initialize linked wallets list
        // TODO: Increment DidCount
        // TODO: Emit DidCreated event
        todo!("create_did: implement DID creation flow")
    }

    /// Link a new wallet address to an existing DID.
    ///
    /// This is how users recover from wallet loss — they link a new wallet
    /// to their existing DID without re-verification.
    ///
    /// Requires authorization from the DID owner.
    ///
    /// # Arguments
    /// * `did` - The DID to link a wallet to
    /// * `new_wallet` - The new wallet address to link
    pub fn link_wallet(_env: Env, _did: Address, _new_wallet: Address) -> Result<(), Error> {
        // TODO: Verify caller is the DID owner
        // TODO: Check new_wallet isn't already linked
        // TODO: Add new_wallet to LinkedWallets
        // TODO: Update WalletToDid mapping for new_wallet
        // TODO: Emit WalletLinked event
        todo!("link_wallet: implement wallet linking")
    }

    /// Remove a wallet address from a DID.
    ///
    /// Cannot remove the last wallet — a DID must always have at least one
    /// linked wallet to remain usable.
    ///
    /// Requires authorization from the DID owner.
    ///
    /// # Arguments
    /// * `did` - The DID to unlink a wallet from
    /// * `wallet` - The wallet address to remove
    pub fn unlink_wallet(_env: Env, _did: Address, _wallet: Address) -> Result<(), Error> {
        // TODO: Verify caller is the DID owner
        // TODO: Check wallet is linked
        // TODO: Check this isn't the last wallet
        // TODO: Remove wallet from LinkedWallets
        // TODO: Remove WalletToDid mapping
        // TODO: Emit WalletUnlinked event
        todo!("unlink_wallet: implement wallet removal")
    }

    /// Resolve a wallet address to its DID.
    ///
    /// This is the primary lookup used by the Verity backend and third-party
    /// apps to find a user's identity from their wallet.
    ///
    /// Returns None if the wallet has no DID.
    pub fn get_did_for_wallet(env: Env, wallet: Address) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::WalletToDid(wallet))
    }

    /// Get all wallet addresses linked to a DID.
    ///
    /// Used by the frontend dashboard to show the user all their linked wallets.
    pub fn get_linked_wallets(env: Env, did: Address) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::LinkedWallets(did))
            .unwrap_or_else(|| soroban_sdk::Vec::new(&env))
    }

    /// Check whether a DID has been verified by a trusted KYC provider.
    ///
    /// Third-party apps call this (via the backend DID Resolution API)
    /// to check a user's verification status. They never see wallet addresses
    /// or personal details — only this boolean.
    pub fn is_verified(env: Env, did: Address) -> bool {
        let record: Option<DidRecord> = env.storage().persistent().get(&DataKey::DidToRecord(did));

        match record {
            Some(r) => r.is_verified,
            None => false,
        }
    }

    /// Set the verification status of a DID.
    ///
    /// Called by the Verity backend after a KYC provider confirms identity.
    /// Requires admin authorization — only the Verity backend can verify DIDs.
    ///
    /// # Arguments
    /// * `did` - The DID to update
    /// * `verified` - New verification status
    pub fn set_verified(_env: Env, _did: Address, _verified: bool) -> Result<(), Error> {
        // TODO: Load admin from instance storage and require_auth
        // TODO: Load DidRecord, update is_verified, save
        // TODO: Emit VerificationStatusChanged event
        todo!("set_verified: implement verification status update")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

mod test;
