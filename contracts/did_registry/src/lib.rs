#![no_std]

mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, xdr::ToXdr, Address, Bytes, BytesN, Env, Vec,
};

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
    /// The caller is not authorized to perform this action.
    Unauthorized = 3,
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
/// A DID is a permanent identity identifier that can have multiple wallet
/// addresses linked to it. If a wallet is lost, the user links a new wallet to
/// their existing DID — identity is never lost.
///
/// ## The DID identifier
///
/// A DID is a deterministic `BytesN<32>` derived from the registry contract,
/// the owner wallet, and a salt. It is an identifier used as a storage key and
/// DID-URI component — it is NOT an `Address` and cannot authenticate anything.
/// All mutations to the DID document are authorized by the controlling wallet.
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
        env.storage()
            .instance()
            .set(&storage::DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&storage::DataKey::DidCount, &0u32);
    }

    /// Derive a deterministic DID identifier for an owner.
    ///
    /// The identifier is `sha256(domain_separator || contract_id || owner || salt)`.
    /// Because a wallet can only ever own one DID (registration is blocked by
    /// `DidAlreadyExists`), the owner alone makes the identifier unique — the
    /// salt and domain separator are included for hygiene and future use.
    fn derive_did_id(env: &Env, owner: &Address) -> BytesN<32> {
        let domain_separator = Bytes::from_slice(env, b"verity-did/v1");
        let mut input = domain_separator;
        input.append(&env.current_contract_address().to_xdr(env));
        input.append(&owner.clone().to_xdr(env));
        input.append(&storage::get_did_count(env).to_xdr(env));
        env.crypto().sha256(&input).into()
    }

    /// Create a new DID and link the caller's wallet to it.
    ///
    /// Flow:
    /// 1. Verify the caller's wallet doesn't already have a DID
    /// 2. Derive a deterministic DID identifier for the owner
    /// 3. Store the DidRecord with the caller as owner
    /// 4. Link the caller's wallet to the DID in WalletToDid mapping
    /// 5. Add the wallet to the DID's LinkedWallets list
    /// 6. Increment DidCount
    /// 7. Emit a DidCreated event
    ///
    /// Returns the DID identifier that was created.
    ///
    /// # Arguments
    /// * `owner` - The wallet address creating the DID. Must authorize this call.
    pub fn create_did(env: Env, owner: Address) -> Result<BytesN<32>, Error> {
        owner.require_auth();

        if storage::get_did_for_wallet(&env, &owner).is_some() {
            return Err(Error::DidAlreadyExists);
        }

        let did = Self::derive_did_id(&env, &owner);
        let created_at = env.ledger().timestamp();

        let record = storage::DidRecord {
            did_id: did.clone(),
            owner: owner.clone(),
            created_at,
            is_verified: false,
        };
        storage::set_did_record(&env, &did, &record);
        storage::set_wallet_to_did(&env, &owner, &did);

        let mut wallets = Vec::new(&env);
        wallets.push_back(owner.clone());
        storage::set_linked_wallets(&env, &did, &wallets);

        storage::increment_did_count(&env);
        events::publish_did_created(&env, &did, &owner, created_at);

        Ok(did)
    }

    /// Link a new wallet address to an existing DID.
    ///
    /// This is how users recover from wallet loss — they link a new wallet
    /// to their existing DID without re-verification.
    ///
    /// Requires mutual consent: the DID's owner must authorize, and the wallet
    /// being linked must also authorize. This prevents both hijacking a DID
    /// you don't control and tagging an address to a DID without its holder's
    /// agreement.
    ///
    /// # Arguments
    /// * `did` - The DID identifier to link a wallet to
    /// * `new_wallet` - The new wallet address to link
    pub fn link_wallet(env: Env, did: BytesN<32>, new_wallet: Address) -> Result<(), Error> {
        let record = storage::get_did_record(&env, &did).ok_or(Error::DidNotFound)?;

        record.owner.require_auth();

        if storage::get_did_for_wallet(&env, &new_wallet).is_some() {
            return Err(Error::WalletAlreadyLinked);
        }

        new_wallet.require_auth();

        let mut wallets = storage::get_linked_wallets(&env, &did);
        wallets.push_back(new_wallet.clone());
        storage::set_linked_wallets(&env, &did, &wallets);
        storage::set_wallet_to_did(&env, &new_wallet, &did);
        events::publish_wallet_linked(&env, &did, &new_wallet);

        Ok(())
    }

    /// Remove a wallet address from a DID.
    ///
    /// Cannot remove the last wallet — a DID must always have at least one
    /// linked wallet to remain usable.
    ///
    /// Either the DID's owner or the wallet being removed may authorize the
    /// removal: the owner revoking a compromised wallet, or a wallet holder
    /// detaching themselves from an identity they no longer want associated
    /// with.
    ///
    /// # Arguments
    /// * `did` - The DID identifier to unlink a wallet from
    /// * `wallet` - The wallet address to remove
    /// * `caller` - The address authorizing this call (owner or `wallet`)
    pub fn unlink_wallet(
        env: Env,
        did: BytesN<32>,
        wallet: Address,
        caller: Address,
    ) -> Result<(), Error> {
        caller.require_auth();

        let record = storage::get_did_record(&env, &did).ok_or(Error::DidNotFound)?;

        if caller != record.owner && caller != wallet {
            return Err(Error::Unauthorized);
        }

        if storage::get_did_for_wallet(&env, &wallet).is_none() {
            return Err(Error::WalletNotLinked);
        }

        let mut wallets = storage::get_linked_wallets(&env, &did);
        if wallets.len() <= 1 {
            return Err(Error::CannotRemoveLastWallet);
        }

        let mut index: Option<u32> = None;
        for (i, w) in wallets.iter().enumerate() {
            if w == wallet {
                index = Some(i as u32);
                break;
            }
        }
        let index = index.ok_or(Error::WalletNotLinked)?;
        wallets.remove(index);
        storage::set_linked_wallets(&env, &did, &wallets);
        storage::remove_wallet_to_did(&env, &wallet);
        events::publish_wallet_unlinked(&env, &did, &wallet);

        Ok(())
    }

    /// Resolve a wallet address to its DID.
    ///
    /// This is the primary lookup used by the Verity backend and third-party
    /// apps to find a user's identity from their wallet.
    ///
    /// Returns None if the wallet has no DID.
    pub fn get_did_for_wallet(env: Env, wallet: Address) -> Option<BytesN<32>> {
        storage::get_did_for_wallet(&env, &wallet)
    }

    /// Get all wallet addresses linked to a DID.
    ///
    /// Used by the frontend dashboard to show the user all their linked wallets.
    pub fn get_linked_wallets(env: Env, did: BytesN<32>) -> Vec<Address> {
        storage::get_linked_wallets(&env, &did)
    }

    /// Check whether a DID has been verified by a trusted KYC provider.
    ///
    /// Third-party apps call this (via the backend DID Resolution API)
    /// to check a user's verification status. They never see wallet addresses
    /// or personal details — only this boolean.
    pub fn is_verified(env: Env, did: BytesN<32>) -> bool {
        storage::is_verified(&env, &did)
    }

    /// Set the verification status of a DID.
    ///
    /// Called by the Verity backend after a KYC provider confirms identity.
    /// Requires admin authorization — only the Verity backend can verify DIDs.
    ///
    /// # Arguments
    /// * `did` - The DID identifier to update
    /// * `verified` - New verification status
    pub fn set_verified(env: Env, did: BytesN<32>, verified: bool) -> Result<(), Error> {
        storage::get_admin(&env).require_auth();

        let mut record = storage::get_did_record(&env, &did).ok_or(Error::DidNotFound)?;
        record.is_verified = verified;
        storage::set_did_record(&env, &did, &record);
        events::publish_verification_status_changed(&env, &did, verified);

        Ok(())
    }
}
