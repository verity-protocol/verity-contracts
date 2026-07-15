#![allow(dead_code, deprecated)]

use soroban_sdk::{contracttype, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Event Types
// ---------------------------------------------------------------------------

/// Emitted when a new issuer is registered.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct IssuerAddedEvent {
    /// The issuer address that was registered.
    pub issuer: Address,
}

/// Emitted when an issuer is removed.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct IssuerRemovedEvent {
    /// The issuer address that was removed.
    pub issuer: Address,
}

// ---------------------------------------------------------------------------
// Event Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn publish_issuer_added(env: &Env, issuer: &Address) {
    let event = IssuerAddedEvent {
        issuer: issuer.clone(),
    };
    env.events()
        .publish((Symbol::new(env, "issuer_added"),), event);
}

#[allow(dead_code)]
pub fn publish_issuer_removed(env: &Env, issuer: &Address) {
    let event = IssuerRemovedEvent {
        issuer: issuer.clone(),
    };
    env.events()
        .publish((Symbol::new(env, "issuer_removed"),), event);
}
