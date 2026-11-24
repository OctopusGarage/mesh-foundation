# Business-neutral migration verification

Verified locally on 2026-11-24 against the unreleased alpha.2 source. This is not a
new registry release or an independent cryptographic audit.

## Earlier protocol extraction baseline

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

Mesh Forum passes 26 headless tests against local SDK overrides. A second run
using the reviewed full Git revision also passes all 26 tests without local
overrides; its manifest and lockfile now pin that revision. Its local record
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

## Policy/adapter separation follow-up

The subsequent alpha.2 refactor removes neutral account lifecycle timestamps,
replaces implicit crypto recovery policy with explicit rebase candidates, and
separates mailbox model/ports/client from optional wire/spool profiles.

- Workspace default suite: 211 tests including doctests passed.
- Workspace all-feature suite: 216 tests including doctests passed.
- Workspace no-default suite: 196 tests including doctests passed.
- Mailbox wire-only: 16 tests passed; spool-only: 26 tests passed.
- Strict workspace all-target/all-feature Clippy and warning-denying rustdoc passed.
- Python release/architecture suite: 10 tests passed; release-policy check passed.
- Pure typed example accepted a 128 KiB record with no wire/storage features.
  The spool-only durable example also passed.

Regression evidence includes initially failing tests for empty-source first append,
byte-aligned 16+4 large-record turns, and backend response budget enforcement.
The fixed code passes these plus 1 KiB turns, exact 200-record allowances, malformed
and nonadvancing pages, failed/cancelled atomic commits, source invalidation,
manual/window-age retention and an independently encoded historical MFMSP1 journal.
Account compile-fail tests enforce the removed lifecycle API. Crypto tests cover
explicit rebase, stale installation and preserved consumed/skipped-key semantics.
CI now checks each mailbox feature independently and runs both composition examples.

Mesh Forum's 26 tests pass both with local overrides and without overrides against
public Git commit `072735a62766bf5d88134c638ef12f357f6f1cfc`, including private
relay retrieval after authenticated TCP reconnect. All seven manifest/lockfile
entries pin that reviewed revision. Mesh Talk's 140 channel-related core tests
pass against an identical managed SDK copy; its desktop library check also passes
with Rust 1.99.0. Strict core/protocol all-target Clippy passes. The separate
late-initial-key history repair regression passes. Structural standards and
repository hygiene checks pass. Mesh Talk adaptation is committed locally on
`requirements-work` as `72aacc87`; unrelated search/documentation work is excluded.
Its complete protocol suite passes 88 tests, including independent historical
identity/certificate/event/discovery byte fixtures and lost-reply TCP resumption.

The final Mesh Talk session suite passes 111 tests, with its existing scale
rehearsal ignored. This covers bounded range/history turns, committed progress,
wrong-scope and duplicate responses, unavailable records, compatibility fallbacks,
TCP reconnect and attachment framing. No ignored scale or UI E2E result is claimed.

## Boundary audit follow-up: documentation and compatibility guards

The audit follow-up changes documentation and regression checks, not production
interfaces, KDF labels, key derivation, wire formats or checkpoint encoding.
Outdated DM phase, application file, desktop-only and keystore-owner descriptions
were replaced with host-owned integration contracts.

- Python suite: 14 tests passed, including rule fixtures for compact application
  names, exact/location-bound historical literals, aliased dependencies and
  target-specific build dependencies. The new fixtures first exposed the old
  rules' missing checks before implementation.
- E2EE suite: 41 tests passed. Independent hmac/hashlib HKDF known answers pin root,
  chain and message derivation. An independently built MTRX version-one fixture
  pins receiver checkpoint bytes in both encoding and decoding directions.
- A temporary mutation of the root HKDF label made the known-answer test fail;
  restoring the original bytes returned the complete E2EE suite to green.
- Strict workspace all-target/all-feature Clippy, warning-denying workspace
  rustdoc, format and release-policy checks passed.

No new crypto profile, generic policy framework, application migration or registry
publication was introduced. The full workspace/consumer suites from the previous
implementation remain historical evidence; this follow-up reran the affected
E2EE tests and repository checks rather than claiming another full consumer run.
