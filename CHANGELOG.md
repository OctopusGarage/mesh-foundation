# Changelog

## Unreleased

- Define private Git SDK releases, local Cargo overrides, and cross-platform verification.
- Retire interrupted peer sessions and reject concurrent writers to a mailbox spool.
- Centralize Have frame validation and inventory exchange in the replication round.
- Require that installed receive candidates match their sender-chain generation.
- Verify ordered attachment streams and isolate encrypted log replacements.
- Verify device certificates against the expected device signing key.

## 0.1.0

- Initial seven-crate extraction covering connectivity, replication, identity,
  end-to-end encryption, storage, offline mailbox, and blob primitives.
