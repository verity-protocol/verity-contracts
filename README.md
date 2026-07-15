# Verity Contracts

Soroban smart contracts for **Verity** — self-sovereign identity on Stellar.

> Prove who you are, reveal nothing.

## What is Verity?

Verity is a decentralized identity protocol built on Stellar and Soroban. It gives users a permanent on-chain identity (DID) that exists independently of any wallet address. Users verify their identity once through a trusted KYC provider, their documents are immediately deleted, and from that point forward any Stellar app can confirm they are verified with a single click — without ever seeing their wallet address, documents, or transaction history.

Every Stellar application needs identity verification. Right now each one forces users to re-upload documents, stores sensitive files on centralized servers, and exposes wallet addresses to every app they visit. Verity eliminates all of this.

Stellar's Protocol 25 and 26 added ZK-friendly cryptographic primitives — BN254 curve operations, Poseidon hashing, multi-scalar multiplication — making on-chain proof verification affordable. Verity builds the identity layer on top of these primitives.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Stellar Blockchain                       │
│                                                                 │
│  ┌─────────────────┐  ┌──────────────────┐  ┌───────────────┐  │
│  │  did_registry    │  │  credential      │  │ zk_verifier   │  │
│  │  ─────────────── │  │  ──────────────  │  │ ──────────── │  │
│  │  Creates DIDs    │  │  Stores cred     │  │ Verifies ZK   │  │
│  │  Links wallets   │  │  hashes on-chain │  │ proofs using  │  │
│  │  Wallet rotation │  │  Revocation      │  │ BN254 host fns│  │
│  └────────┬─────────┘  └────────┬─────────┘  └───────┬───────┘  │
│           │                     │                     │          │
│  ┌────────┴─────────┐  ┌───────┴──────────┐                    │
│  │  nullifier       │  │ issuer_registry   │                    │
│  │  ──────────────  │  │ ────────────────  │                    │
│  │  Sybil resist.   │  │ Approved KYC      │                    │
│  │  one DID per     │  │ providers list    │                    │
│  │  real identity   │  │ Admin-controlled  │                    │
│  └──────────────────┘  └──────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
           ▲                     ▲                     ▲
           │                     │                     │
           └──────────┬──────────┘─────────────────────┘
                      │
           ┌──────────┴──────────┐
           │   verity-backend    │
           │   (NestJS/TS)       │
           │                     │
           │  KYC Verifier       │
           │  OAuth Service      │
           │  DID Resolution API │
           │  Indexer Service    │
           └──────────┬──────────┘
                      │
           ┌──────────┴──────────┐
           │   verity-frontend   │
           │   (Next.js/TS)     │
           └─────────────────────┘
```

## Contracts

| Contract | Purpose | Status |
|----------|---------|--------|
| **did_registry** | Creates DIDs, links/unlinks wallets, manages wallet rotation. The foundation of Verity identity. | Scaffolded — data structures and function signatures defined, implementation needed |
| **credential** | Stores credential hashes issued by approved KYC providers. Handles revocation. | Scaffolded — data structures and function signatures defined, implementation needed |
| **zk_verifier** | Verifies Noir-generated ZK proofs on-chain using BN254 host functions. | Stub — function signatures defined, implementation needed |
| **nullifier** | Prevents same real-world identity from registering multiple DIDs (Sybil resistance). | Stub — function signatures defined, implementation needed |
| **issuer_registry** | Admin-controlled registry of approved KYC provider addresses. | Stub — function signatures defined, implementation needed |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (v1.84.0+)
- [Stellar CLI](https://developers.stellar.org/docs/tools/smart-contracts-cli) (`stellar`)

### Install

```bash
git clone https://github.com/verity-stellar/verity-contracts.git
cd verity-contracts
```

### Build

```bash
cargo check        # Verify compilation
cargo build        # Full build
```

### Test

```bash
cargo test
```

### Lint

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

## Project Structure

```
verity-contracts/
├── Cargo.toml                          # Workspace config
├── contracts/
│   ├── did_registry/                   # Core identity contract
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Data structures + function signatures
│   │       └── test.rs                 # Tests (2 implemented)
│   ├── credential/                     # Credential storage contract
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                  # Data structures + function signatures
│   ├── zk_verifier/                    # ZK proof verification
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                  # Stub with TODO comments
│   ├── nullifier/                      # Sybil resistance
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                  # Stub with TODO comments
│   └── issuer_registry/                # KYC provider management
│       ├── Cargo.toml
│       └── src/lib.rs                  # Stub with TODO comments
├── README.md
├── CONTRIBUTING.md
└── .github/
    ├── ISSUE_TEMPLATE/
    │   ├── bug_report.md
    │   ├── feature_request.md
    │   └── contract_task.md
    └── pull_request_template.md
```

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for how to pick up issues, set up your development environment, and submit PRs.

Look for issues labeled [`good-first-issue`](https://github.com/verity-stellar/verity-contracts/labels/good-first-issue) or [`contract-task`](https://github.com/verity-stellar/verity-contracts/labels/contract-task) to get started.

## License

This project is licensed under the Apache License 2.0 — see [LICENSE](LICENSE) for details.
