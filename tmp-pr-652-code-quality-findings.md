# PR #652 Code Quality Findings

Temporary capture for the `RWA: transfer compliance modules` PR.

- PR: https://github.com/OpenZeppelin/stellar-contracts/pull/652
- Branch: `feat/rwa-transfer-standalone`
- Scope:
  - `initial_lockup_period`
  - `time_transfers_limits`
  - `transfer_restrict`

## Review Inputs

Checked against:

- `/openzeppelin-stellar-rwa-code-quality`
- `.cursor/skills/code-quality.md`
- Learnings from PR #650 and PR #651

There are no human reviews yet. There are two unresolved CodeRabbit threads
about fail-closed transfer-hook behavior.

## Findings To Address

### 1. Fail-closed transfer hook in `initial_lockup_period`

File:

- `packages/tokens/src/rwa/compliance/modules/initial_lockup_period/storage.rs`

Issue:

- `on_transfer` mutates balances and lock state without re-checking the
  unlocked-balance predicate.
- If `CanTransfer` is skipped, miswired, or the hook is called directly, the
  state mutation can proceed without proving the transfer amount is unlocked.

Action:

- Add a `panic_with_error!`-based guard before mutation.
- Reuse the same predicate as `can_transfer` or extract a shared helper.
- Add or reuse a concrete `ComplianceModuleError` variant.
- Cover this with a focused test.

### 2. Fail-closed transfer hook in `time_transfers_limits`

File:

- `packages/tokens/src/rwa/compliance/modules/time_transfers_limits/storage.rs`

Issue:

- `on_transfer` increments counters without re-checking the transfer-limit
  predicate.
- If `CanTransfer` is skipped, miswired, or the hook is called directly, the
  counter mutation can happen after an invalid transfer.

Action:

- Add a `panic_with_error!`-based guard before increasing counters.
- Reuse the same predicate as `can_transfer` or extract a shared helper.
- Add or reuse a concrete `ComplianceModuleError` variant.
- Cover this with a focused test.

### 3. Traitless module APIs

Files:

- `packages/tokens/src/rwa/compliance/modules/initial_lockup_period/mod.rs`
- `packages/tokens/src/rwa/compliance/modules/time_transfers_limits/mod.rs`
- `packages/tokens/src/rwa/compliance/modules/transfer_restrict/mod.rs`
- corresponding examples under `examples/rwa-*`

Issue:

- Module-specific APIs are exposed only as inherent example methods and storage
  helpers.
- Based on PR #650/#651, module-specific compliance APIs should live in
  `#[contracttrait]` traits that compose with `ComplianceModule`.

Action:

- Add module-specific traits for all three modules.
- Keep shared hook methods only on `ComplianceModule`.
- Update examples to implement both the module-specific trait and
  `ComplianceModule`.

### 4. Old example authorization pattern

Files:

- `examples/rwa-initial-lockup-period/src/contract.rs`
- `examples/rwa-time-transfers-limits/src/contract.rs`
- `examples/rwa-transfer-restrict/src/contract.rs`

Issue:

- Examples use local `DataKey::Admin`, local `set_admin`/`get_admin`, and
  `require_module_admin_or_compliance_auth`.
- PR #651 established the split:
  - configuration/setters: admin-gated
  - stateful hooks: compliance-gated
- Broader examples prefer access-control macros when they fit.

Action:

- Use `stellar_access::access_control` admin helpers.
- Use `#[only_admin]` for admin-gated configuration methods where applicable.
- Keep a manual compliance-auth helper for stateful compliance hooks.
- Add `stellar-macros` dependency to examples that use macros.

### 5. Non-native storage sections

Files:

- `packages/tokens/src/rwa/compliance/modules/*/storage.rs`

Issue:

- Storage files still use ad-hoc headings:
  - `RAW STORAGE`
  - `ACTIONS`
  - `HOOK WIRING`
  - `COMPLIANCE HOOKS`

Action:

- Normalize to:
  - `QUERY STATE`
  - `CHANGE STATE`
  - `HELPERS` only where needed
- Keep public hook/query wrappers under `QUERY STATE` when they only validate
  or query state.

### 6. Write-side TTL extension

Files:

- `initial_lockup_period/storage.rs`
- `time_transfers_limits/storage.rs`
- `transfer_restrict/storage.rs`

Issue:

- Several write helpers call `extend_ttl` immediately after `set`.
- PR #650/#651 convention: persistent entries extend TTL on successful reads,
  not writes.

Action:

- Remove write-side `extend_ttl`.
- Keep read-side TTL extension where entries exist.

### 7. Inline event publishing from storage

Files:

- all three module `storage.rs` files
- all three module `mod.rs` files

Issue:

- Storage functions publish events inline via `.publish(e)`.

Action:

- Keep event structs and paired `emit_*` helpers in `mod.rs`.
- Add `// ################## EVENTS ##################` before event declarations.
- Call `emit_*` helpers from storage.
- Event helper signatures should use imported `&Env` where `Env` is in scope.

### 8. User-facing `assert!`, `unwrap()`, and `expect()` in production paths

Files:

- `initial_lockup_period/storage.rs`
- `time_transfers_limits/storage.rs`

Issue:

- User-facing failures still use `assert!`.
- Vector indexing uses `unwrap()` / `expect()` in production paths.

Action:

- Replace user-facing failures with `panic_with_error!` and concrete
  `ComplianceModuleError` variants.
- Use `get_unchecked` only where loop bounds prove the index is valid.
- Leave test-only `unreachable!` alone unless clippy flags it.

### 9. `transfer_restrict` set-membership semantics

File:

- `packages/tokens/src/rwa/compliance/modules/transfer_restrict/storage.rs`

Issue:

- Membership stores `bool` instead of key presence with `()`.
- Add/remove emits every time, even if state did not change.
- Batch functions do not delegate to single-item helpers.

Action:

- Use key presence and `()` values.
- Emit add/remove events only when membership changes.
- Make batch helpers delegate to single-item helpers.

### 10. Sparse public docs

Files:

- all three module `storage.rs` files
- all three module `mod.rs` files

Issue:

- Many public helpers are missing required native sections:
  - `# Errors`
  - `# Events`
  - `# Security Warning`

Action:

- Add `# Security Warning` to unauthenticated mutating storage helpers.
- Add `# Events` sections to functions that emit.
- Add concrete or delegated `# Errors` sections where helpers can fail.
- Keep section order:
  `# Arguments` -> `# Errors` -> `# Events` -> `# Notes` ->
  `# Security Warning`.

### 11. Module-level docs need Soroban adaptation context

Files:

- `initial_lockup_period/mod.rs`
- `time_transfers_limits/mod.rs`

Issue:

- These modules maintain internal mirrors/counters through hooks.
- The Stellar-specific adaptation from T-REX should be explained at module
  level.

Action:

- Document how internal balances, locks, and counters are maintained through
  hooks.
- Explain any difference from EVM/T-REX source behavior.

### 12. Shared `ComplianceModule` docs still use `# Security Note`

File:

- `packages/tokens/src/rwa/compliance/modules/mod.rs`

Issue:

- The shared trait docs still use non-native `# Security Note`.

Action:

- Rename to native sections such as `# Security Warning` or `# Notes`.
- Keep docs concise and consistent with current library style.

## Suggested Work Order

1. Fix functional fail-closed hook behavior first.
2. Add/adjust errors in `ComplianceModuleError`.
3. Normalize storage sections and TTL behavior.
4. Move inline event publishes to `emit_*` helpers.
5. Add module-specific traits and update examples.
6. Split example auth into admin-gated setters and compliance-gated hooks.
7. Complete docs and tests.
8. Run focused validation:
   - `cargo +nightly fmt --all --check`
   - `cargo test -p stellar-tokens initial_lockup_period`
   - `cargo test -p stellar-tokens time_transfers_limits`
   - `cargo test -p stellar-tokens transfer_restrict`
   - `cargo test -p rwa-initial-lockup-period -p rwa-time-transfers-limits -p rwa-transfer-restrict`
   - `cargo clippy --release --locked -p stellar-tokens -p rwa-initial-lockup-period -p rwa-time-transfers-limits -p rwa-transfer-restrict --all-targets -- -D warnings`
   - `cargo doc --locked --no-deps -p stellar-tokens -p rwa-initial-lockup-period -p rwa-time-transfers-limits -p rwa-transfer-restrict`
