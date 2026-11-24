# Business-neutral migration verification

Verified locally on 2026-11-24 against the unreleased alpha.2 source. This is not a
new registry release or an independent cryptographic audit.

## SDK

- `cargo test --locked --workspace --all-features`: 198 tests including doctests.
- `cargo test --locked --workspace --no-default-features`: 192 tests including doctests.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --all-features --no-deps`: passed.
- `cargo run --locked -p mesh-foundation-mailbox --example opaque_mailbox`: passed.
- Release-policy check: seven coordinated alpha.2 packages with exact internal pins.
- Python release/boundary suite: seven tests passed. The original pre-migration
  source fails the new architecture guards; the migrated source passes.

The generic mailbox regression first failed because opaque Record/ScopeId APIs
were absent. Additional delta regressions failed before fixing revision-only
unchanged detection and invalid ordinal advancement. Tests now cover these cases,
initial nonadvancing pages, commit failure, source reset, immutable upper windows,
200-record limits, encrypted spool reopen, expiry and authenticated TCP resume.
No chat event enum is needed by the generic tests or public SDK APIs.

Independent final source review found no remaining material boundary/correctness
blockers after fallible advertisement encoding, independent capabilities and
cursor guards were addressed.

## Consumer-owned application protocols

Mesh Talk's protocol suite passes 88 tests, including original event/wire/round,
mailbox, discovery, account and device behavior. Three independent golden tests
pin old device/account fingerprints, certificate domain, event tag19, MTAN v2 and
MTOM1. The core session suite passes 111 tests with the existing scale rehearsal
ignored. Application validation tooling tests pass 66 cases and registers the new
protocol crate in the actual source/contract inventory.

Mesh Forum passes 26 headless tests against local SDK overrides. Its local record
schema contains only Post/Revision/Withdrawal and preserves existing record bytes.
Its private-content composition test encrypts the complete signed host payload
before opaque mailbox deposition, rejects tampering, reopens durable storage,
resumes authenticated TCP and verifies decrypted content/attachments. Two existing
forum attachment API argument-count Clippy findings remain outside this migration;
no broad lint suppression or unrelated API rewrite was introduced.

The forum working tree already contained substantial uncommitted application
work. Consumer migration was applied without replacing or committing that work.
Mesh Talk's unrelated documentation changes were likewise preserved.

No p95/p99, ignored scale rehearsal or full desktop end-to-end coverage is claimed.
Published alpha.1 source, tag and registry artifacts remain immutable.
