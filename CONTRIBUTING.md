# Contributing to Verity Contracts

Thanks for your interest in contributing to Verity! This guide covers how to pick up work, the standards we follow, and how to submit changes.

## Picking Up an Issue

1. Browse open issues — look for [`good-first-issue`](https://github.com/verity-stellar/verity-contracts/labels/good-first-issue) or [`contract-task`](https://github.com/verity-stellar/verity-contracts/labels/contract-task) labels
2. Comment on the issue to claim it — mention which contract you'll be working on
3. Fork the repo and create a branch from `main`
4. Check the issue's acceptance criteria before you start

**Definition of "done":**
- The function compiles and passes `cargo check`
- At least one test covers the new function
- `cargo fmt -- --check` passes
- `cargo clippy -- -D warnings` passes
- The PR is reviewed and merged

## Development Setup

```bash
# Clone your fork
git clone https://github.com/<your-username>/verity-contracts.git
cd verity-contracts

# Verify your environment
cargo check
cargo test

# Create a branch
git checkout -b feat/did-registry-create-did
```

### Prerequisites

- Rust v1.84.0+ (`rustup update`)
- No external services required — tests run entirely in-memory using Soroban's `Env`

## Code Style

### Formatting

All code must pass `rustfmt` with default settings:

```bash
cargo fmt -- --check
```

### Linting

All code must pass Clippy with no warnings:

```bash
cargo clippy -- -D warnings
```

### Soroban Conventions

- Use `#[contracttype]` enums for storage keys (not raw `Symbol` constants)
- Use `#[contracterror]` enums with `#[repr(u32)]` for errors, starting at 1
- Prefix unused parameters with `_` (e.g., `_env` in stub functions)
- Use `todo!("function_name: description")` for unimplemented functions
- One contract per crate, one crate per directory under `contracts/`

### Doc Comments

- Every public function must have a doc comment explaining what it does
- Document `# Arguments` for non-obvious parameters
- Document `# Panics` if the function can panic
- Keep comments focused on **intent**, not implementation details

### File Structure

Each contract follows this layout:

```
contracts/<name>/
├── Cargo.toml
└── src/
    ├── lib.rs       # Contract definition, data structures, function signatures
    └── test.rs      # Tests (included via `mod test;` in lib.rs)
```

## PR Guidelines

### Branch Naming

```
feat/<contract>-<description>     # New functionality
fix/<contract>-<description>      # Bug fix
docs/<description>                # Documentation only
test/<contract>-<description>     # Test coverage improvements
```

Examples:
- `feat/did-registry-create-did`
- `fix/credential-revocation-auth`
- `test/did-registry-wallet-rotation`

### What to Include

1. **Focused changes** — one contract per PR when possible
2. **Tests** — every new function needs at least one test
3. **No secrets** — never commit API keys, private keys, or credentials
4. **Clean history** — rebase on `main` before submitting if your branch is behind

### PR Description Template

Use the pull request template (`.github/pull_request_template.md`). Include:

- Which contract(s) you changed
- What the change does
- How you tested it
- Related issue number

### Review Process

1. CI must pass (compilation, formatting, tests)
2. At least one maintainer review required
3. Address review feedback by pushing additional commits
4. Squash and merge once approved

## Testing Requirements

### Every Contract Function Needs Coverage

At minimum, each public function should have one test covering the happy path. Error paths are encouraged.

### Test Naming Convention

```
test_<function_name>_<scenario>
```

Examples:
- `test_create_did_success`
- `test_create_did_already_exists`
- `test_link_wallet`
- `test_unlink_wallet_last_wallet_fails`

### Test Pattern

Use this pattern as a starting point:

```rust
#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Address, ContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths(); // Auto-approve all auth for simplicity

    let admin = Address::generate(&env);
    let contract_id = env.register(Contract, (admin.clone(),));
    let client = ContractClient::new(&env, &contract_id);

    (admin, client)
}

#[test]
fn test_example() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(Contract, (admin.clone(),));
    let client = ContractClient::new(&env, &contract_id);

    // Test your function
    // assert_eq!(client.some_function(), expected);
}
```

### When to Use `mock_all_auths()`

- **Default**: Use `env.mock_all_auths()` to auto-approve all authorization. This keeps tests simple and focused on business logic.
- **Auth testing**: When testing that a function correctly requires authorization, use `env.mock_auths(...)` or check `env.auths()` to assert the right addresses were asked to authorize.

## Contract-Specific Notes

### Storage Keys

Always use the `DataKey` enum pattern:

```rust
#[contracttype]
pub enum DataKey {
    SomeGlobal,
    PerUser(Address),
    CompoundKey(Address, Symbol),
}
```

Never use raw strings as storage keys.

### Error Handling

Return `Result<T, Error>` from functions that can fail. Use `?` operator with the error enum:

```rust
pub fn do_something(env: Env) -> Result<(), Error> {
    let value: Option<Foo> = env.storage().persistent().get(&DataKey::SomeGlobal);
    let value = value.ok_or(Error::NotFound)?;
    // ...
    Ok(())
}
```

### Authorization

Call `address.require_auth()` at the start of functions that need authorization. This should be the first line after parameter validation.

## Questions?

Open a [GitHub Discussion](https://github.com/verity-stellar/verity-contracts/discussions) or ask in the issue you're working on.
