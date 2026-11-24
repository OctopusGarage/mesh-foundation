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
| Store-and-forward opaque bytes | `mesh-mailbox::{model, ports, client}` | Authentication, payload encryption, recipient addressing, expiry scheduling and federation |
| Verified large content | `mesh-blob` | Manifests, consent, staging, publication and retention |

`mesh-replication` has no default features and no application event enum.
`mesh-connect` without default features still provides sessions and capability
version intersection. Its optional `signed-discovery` feature adds identity and
serialization for neutral advertisements. Mailbox production code depends only
on generic replication, not account or device identities; storage and wire codecs are optional. Blob's optional
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

`Record::new(scope, bytes)` content-addresses opaque bytes under that scope. Hash
integrity is independent of frame size and cannot establish authorship or permission.
Hosts encrypt/authenticate payloads and check current access before accepting them.

`MailboxStore` is the synchronous authorized backend port; `MailboxTransport` is
the asynchronous carrier port. `client::sync_turn` only consumes typed pages and
records. It creates no timer, network connection or background worker. Its default
turn allows 200 records, 1 MiB of raw payload and eight operations, with up to eight
pages of 64 IDs. Smaller caller budgets propagate to both discovery and body
retrieval. Byte-aligned discovery avoids selecting more bodies than a turn can
accept. A record larger than the available allowance produces an explicit error;
it is never acknowledged as an empty completion. Failed/cancelled batches leave
that batch's cursor uncommitted; earlier successful atomic commits remain valid.

The optional `wire` feature supplies `WireTransport` over `ReconciliationSession`,
`wire::sync_turn`, RPC codecs and dispatch. Its `MFOM2` profile carries page and
payload allowances in Delta/Get requests. This adapter imposes its own bounded
frame profile, independently of the typed model. Service dispatch also validates
backend results before emitting them. The earlier unreleased `MFOM1` wire is
rejected; published alpha.1 and Mesh Talk's own protocol remain separate.

The optional `spool` feature supplies `Spool::open_configured` and `SpoolOptions`:
record/query limits are independent of wire limits. Choose `Retention::Manual`
for host-triggered clearing or `WindowAge` for the entire first-seen window, not
per-record TTL. The convenience open methods preserve first-window-age behavior.
The spool owns its writer lease, encoded-storage quotas and destructive epoch.
Retention timestamps stay private to storage; cursors carry source/snapshot
identity and position. Initial empty retrieval followed by an append resumes
normally. Clearing/expiry invalidates old cursors. The `MFMSP1` disk layout is
unchanged and covered by an independently constructed historical journal fixture.
A custom backend may implement another retention/index policy.

Both features are enabled by default for convenience; use `default-features =
false` for the pure model/ports/client, or enable either feature independently.
Run these examples from the workspace root:

```sh
cargo run -p mesh-foundation-mailbox --no-default-features --example typed_mailbox
cargo run -p mesh-foundation-mailbox --no-default-features --features spool --example opaque_mailbox
```

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

Neutral account identities contain signing material, not creation dates or a
wall-clock lifecycle. Applications retain that metadata in their own schema.
Sender-key recovery uses explicit `prepare_rebase` and generation-checked
`install`; hosts authenticate the distribution and decide when rebasing is
allowed. Persist the candidate checkpoint atomically with the host state. No
implicit gap threshold inside the SDK decides application recovery policy.
