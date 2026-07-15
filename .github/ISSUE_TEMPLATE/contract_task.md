---
name: Contract Task
about: Pick up a contract implementation task
title: "[contract] "
labels: contract-task
assignees: ''
---

## Contract

<!-- Which contract are you working on? -->
- [ ] did_registry
- [ ] credential
- [ ] zk_verifier
- [ ] nullifier
- [ ] issuer_registry

## Task Type

- [ ] New function implementation
- [ ] Test coverage
- [ ] Bug fix
- [ ] Optimization
- [ ] Documentation

## Description

<!-- What needs to be implemented or changed? -->

## Acceptance Criteria

- [ ] Function compiles and passes `cargo check`
- [ ] At least one test covers the new/changed function
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Doc comments added for public functions

## Related Contracts

<!-- If this involves cross-contract calls, list the other contracts involved. -->

## Implementation Notes

<!-- Any hints about implementation approach, storage patterns, or design decisions. -->

See the contract's `lib.rs` for TODO comments that describe the intended implementation flow.
