# Changelog

## Unreleased

- Add typed SDK diagnostics, committed progress callbacks, and durability reports.
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
