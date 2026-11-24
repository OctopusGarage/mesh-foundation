# Composing Mesh Foundation

This document describes the unreleased alpha.2 interfaces. Published alpha.1 has
its original compatibility surface; see [migration](business-neutral-migration.md).

## Select mechanisms, define your own application

| Need | Crate / entry point | Host responsibilities |
| --- | --- | --- |
| Authenticated encrypted byte stream | `mesh-connect::PeerSession` | Trusted peer keys, admission, deadlines and reconnect scheduling |
| Protocol/version agreement | `mesh-connect::capability` | Application protocol IDs, authenticated negotiation and dispatch |
| Signed discovery | `mesh-connect::discovery::announce` | Namespace, capability meaning, freshness and trust |
| Scoped inventory and differences | `mesh-replication::{CompleteInventory, inventory_session, range}` | Authorized finite snapshots and index maintenance |
| Bounded opaque bodies | `mesh-replication::{pull, driver, history}` | Record schema, verification and atomic durable commit |
| Signing and certificates | `mesh-identity::{device, account}` | Account trust, enrollment, revocation and application identifiers |
| End-to-end encryption | `mesh-e2ee::{ratchet, sender_key}` | Key establishment/distribution, membership policy and atomic checkpoint commit |
| Durable encrypted records | `mesh-storage::record_log` | Indexed models, path ownership and lifecycle |
| Store-and-forward opaque bytes | `mesh-mailbox::{Record, service, client, spool}` | Authentication, payload encryption, recipient addressing, expiry scheduling and federation |
| Verified large content | `mesh-blob` | Manifests, consent, staging, publication and retention |

`mesh-replication` has no default features and no application event enum.
`mesh-connect` without default features still provides sessions and capability
version intersection. Its optional `signed-discovery` feature adds identity and
serialization for neutral advertisements. Mailbox production code depends only
on generic replication/storage, not account or device identities. Blob's optional
`mesh-connect` feature provides the direct byte-session adapter.

## Safe composition order

1. Define application-owned protocol IDs, record schemas and scope identifiers.
   A forum may define posts, revisions and withdrawals; a chat application may
   define membership changes. Neither vocabulary is part of this SDK.
2. Authenticate a pinned session and apply current host admission policy.
   A signed discovery capability is a hint, never permission or a session binding.
3. Negotiate a supported protocol/version using bounded capability descriptions.
   `capability::negotiate` intersects two validated version ranges. It is a pure
   mechanism: hosts exchange and authenticate those descriptions themselves.
4. Capture an authorized source snapshot. Compare summaries or bounded inventory,
   then retrieve bodies under record, byte, request and scan budgets. SDK recent
   pages operate on opaque ordered records, not a special chat Message type.
5. Authenticate application payloads and recheck current access. Atomically commit
   accepted bodies, cryptographic receive state and the corresponding cursor.
   A failed/cancelled batch must not acknowledge progress it did not commit.
6. Schedule another bounded turn through the application's existing owner.
   Page exhaustion proves only that source snapshot's coverage, not global equality.

## Opaque mailbox profile

`Record::new(scope, bytes)` content-addresses opaque bytes under that scope. The
SDK verifies this hash and scope; it cannot establish authorship or permission.
Encrypt and authenticate your payload before depositing it and before accepting
it at retrieval. `MailboxStore` replaces the query backend; `ReconciliationSession`
replaces the transport. The supplied spool owns its exclusive writer lease and
applies count/byte limits. The supplied spool expires its entire first-seen
retention window; it does not implement per-record TTL. Expiry invalidates old
cursors by changing its epoch. Supply a different MailboxStore for another policy.

The generic mailbox has a new `MFOM1` wire profile and `MFMSP1` spool format. It
never interprets chat events. An application-specific envelope can contain a
signed document, an encrypted notification, a forum update or any host schema.
Use `cargo run -p mesh-foundation-mailbox --example opaque_mailbox` for a durable
example and the TCP tests for authenticated reconnect/cursor resumption.

## Application compatibility belongs to consumers

Mesh Talk's `mesh-talk-protocol` crate owns its original event IDs, event-kind
indices, account/device fingerprint domains, certificates, discovery packets,
legacy sync rounds, recipient-pair envelopes and old mailbox spool format. It
composes the SDK's generic session, storage and quota mechanisms. The SDK does
not import or publish that crate. Forum records are defined in Mesh Forum and
contain no group membership, announcement or chat consent types.

Cryptographic ratchets, sender-key state machines and encrypted record framing
remain reusable SDK mechanisms. Keeping a stable cryptographic checkpoint is
separate from retaining an application's group lifecycle or access policy.
