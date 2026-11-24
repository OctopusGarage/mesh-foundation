# Business-neutral alpha.2 migration

This is an unreleased, intentionally breaking alpha.2 source/API change. The
published alpha.1 crates and immutable `v0.1.0-alpha.1` tag are unchanged. No new
package is published by this refactor. Do not apply neutral identity domains to an
existing application database and assume the old identifiers will still match.

## Moved to Mesh Talk

The consumer-owned `mesh-talk-protocol` crate now contains:

- `legacy_event`, `legacy_wire` and `legacy_round`, including chat event kinds and
  mixed-version negotiation. `mesh-replication/legacy-wire` no longer exists.
- The original device fingerprint, account identifier and certificate profiles.
- Chat discovery with account certificates and post-office role flags.
- Pairwise recipient envelopes, old mailbox negotiation and the `MTMSP1` spool.

Existing Mesh Talk data/signatures stay on those original byte profiles. Its
protocol crate is `publish = false` and maintained with the application. Generic
SDK mechanisms are dependencies of the application, never the other direction.
Golden tests independently verify fixed historical account/device IDs, certificate
signature domain, event kind19, `MTAN` v2 and `MTOM1` bytes.

## Generic SDK entry points

- Replication carries opaque `[u8;32]` IDs and `pull::Record` bodies. Implement
  verification and authorization in host adapters. Default features are empty.
- `device::{DeviceIdentity,PublicIdentity}::node_id` identifies a signing node;
  `node_id_from` derives it from its public signing key. Neutral node/account/
  certificate separators start with `mesh-foundation-`. Raw signing/DH key bytes
  are unchanged. Existing application fingerprints must stay in their old profile.
- Signed discovery uses `MFAN` v1 with a signed application namespace, <=32 unique
  protocol/version ranges and <=2048 opaque metadata bytes. Constructors and
  encoding are fallible. Old chat discovery is not silently reinterpreted.
- `capability` is available without LAN discovery or identity dependencies. It
  validates bounds/duplicates and chooses the highest common protocol version.
  Hosts own authenticated exchange, downgrade policy, routing and dispatch.
- Mailbox uses opaque scoped `Record` bodies, content integrity, source-pinned
  cursors and generic storage/session seams. `RelayMailbox` and chat Event bodies
  are absent. `MFOM1`/`MFMSP1` distinguish new wire/disk profiles from old bytes.
- Cryptography, blob transfer and encrypted storage keep their reusable state
  machines. Group membership, consent and lifecycle decisions remain host-owned.

## Consumer verification

Mesh Talk compiles through its application-owned adapters and retains old-wire
regressions. Mesh Forum defines only Post, Revision and Withdrawal record kinds
and can consume replication/blob/identity/mailbox without a chat event enum. Its
private-content test composes host signatures, ratchet ciphertext, an opaque spool,
authenticated TCP resumption and attachment verification.

Automated architecture tests reject application event types, chat domain strings,
application dependencies and the removed legacy feature in SDK production code.
CI runs those guards with release-policy tests. This boundary is an enforced
source/dependency contract, not merely a README claim.

See [verification evidence](business-neutral-verification.md) for actual SDK and
consumer checks, historical byte fixtures and remaining coverage limits.
