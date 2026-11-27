# Changelog

## Unreleased

- Add bounded persistent-reader preparation with typed failures and commit-gated
  checkpoint release; retain host-owned body verification, transactions and policy.

- Share reader snapshot qualification and summary-frame admission with custom
  host checkpoints, without moving authorization or causal-repair policy.

- Share finite-window pagination across host-owned cursors and record IDs while
  retaining the neutral mailbox convenience types and serialized page layout.
- Add checked depth-only range summary composition for persistent adapters.
- Serialize snapshot updates with OS leases and retain verified recovery backups
  after failed installation/restoration; retries recover interrupted updates.

- Clarify host-owned storage/discovery responsibilities and historical crypto
  profile compatibility; pin HKDF outputs and receiver checkpoint bytes.
- Strengthen architecture guards for compact application references and aliased
  or target-specific Cargo dependencies. Repository checks require Python 3.11+.

- **Breaking:** remove account lifecycle timestamps and implicit sender-chain
  refresh policy; expose explicit generation-checked receiver rebase candidates.
- **Breaking:** split typed mailbox model/store/transport/client from optional
  `wire` and `spool` features; MFOM2 requests carry count/payload allowances.
- Add configurable spool limits and manual retention, byte-aligned discovery,
  backend result validation, and independent adapter/retention regression tests.
  MFMSP1 durable journal bytes remain compatible.

- **Breaking:** remove Mesh Talk event/wire/round APIs and the `legacy-wire` feature;
  consumer-owned protocol crates preserve application compatibility.
- **Breaking:** introduce neutral node/account identity and signed discovery profiles;
  applications retain their old identifier/certificate domains explicitly.
- **Breaking:** mailbox stores opaque records with neutral wire/disk profiles;
  recipient-pair chat envelopes move to Mesh Talk.
- Add independent bounded protocol capability negotiation, source boundary guards,
  and a generic durable mailbox example. See `docs/business-neutral-migration.md`.

- Add typed SDK diagnostics, committed progress callbacks, and durability reports.
- Add source/scope-bound recent-history pages with explicit older navigation and atomic cursor commits.
- Add fallible range-reader checkpoint APIs for disk-backed authorized indexes.
- Add total-turn pull budgets and an indexed range synchronization driver.
- Add cancellation-safe range checkpoints, compressed summaries and dedicated frames.
- Add bounded mailbox retrieval with atomic body/cursor acknowledgement and explicit reset.
- Prepare coordinated 0.1.0-alpha.2 development; published alpha.1 remains immutable.

## 0.1.0-alpha.1

- Prepare the first coordinated registry alpha under `mesh-foundation-*` names.
- Preserve Rust library/import names; Git consumers must add package aliases.
- Add exact internal alpha dependencies, docs.rs configuration and prerelease checks.
- Support registry and Git local patch generation.

### Open-source preparation

- Add public-facing onboarding, crate guides, contribution and security policies,
  executable examples and an administrator launch checklist.
- Extend CI with default/minimal feature tests, rustdoc checks, dependency updates
  and scheduled advisory scanning; retain registry publication guards.

- Define private Git SDK releases, local Cargo overrides, and cross-platform verification.
- Retire interrupted peer sessions and reject concurrent writers to a mailbox spool.
- Centralize Have frame validation and inventory exchange in the replication round.
- Require that installed receive candidates match their sender-chain generation.
- Verify ordered attachment streams and isolate encrypted log replacements.
- Verify device certificates against the expected device signing key.

## Initial extraction (unpublished)

- Initial seven-crate extraction covering connectivity, replication, identity,
  end-to-end encryption, storage, offline mailbox, and blob primitives.
