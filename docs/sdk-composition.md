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

## Boundary review criteria

Keep mechanisms when their contract is useful without knowing application record
kinds: signing/certificates, pairwise or multi-recipient cryptography, encrypted
logs, scoped hashes, finite snapshots, bounded retrieval and verified blob bytes.
A sender key is a cryptographic primitive; it does not imply a chat group model.
An account certificate is a key binding; it does not imply registration, profile,
creation date, membership or recovery policy.

Move application schemas and decisions to the consumer: post/message types,
announcements, membership/roles, profile metadata, recipient addressing, consent,
compatibility domains, scheduling and authorization. Configurable limits are
resource controls, not application schemas. Defaults remain documented and
replaceable; the SDK does not infer a seven-day business history rule.

Versioned wire and disk profiles may remain supplied SDK adapters, provided the
model and operation can be used independently. Keeping a historical checkpoint
magic value preserves encrypted local data compatibility; it does not import the
application's event model. Changing those bytes merely to rename a prefix would
create a migration without improving the module boundary.


Architecture guards parse actual Cargo dependency package names, including aliases
and target-specific tables. Source checks also catch compact application names.
Only the three exact historical KDF byte literals are exempted at their reviewed
source location; new references on that line still fail. These checks catch known
regressions, not arbitrary application semantics. Semantic review and independent
consumer tests remain necessary. See the E2EE crate README for the retained
cryptographic compatibility profile.

## Persistent synchronization composition

The high-level `SyncDriver` currently owns an in-memory `RangeIndex`. SQL hosts
can implement `RangeRead` and use `Checkpoint::for_reader`/`accept_reader` without
materializing a tree on each query. `Summary::{empty,leaf,combine}` and
`singleton_summary` define the same canonical hashes for independent index writers.
Custom checkpoints can use `ReaderBinding::valid_reader` for scope, authority,
root/revision and pinned-view qualification, and `SummaryFrame::validate_for`
for frontier/frame admission. These checks do not mutate progress or grant access;
read errors remain distinct from invalidation. Read all required facts before
advancing a checkpoint, and retain or discard discovered work with its commit.
The host owns SQL transactions, snapshot qualification, commit atomicity and
scheduling. Keep policy/causal recovery outside this shared mathematical kernel.

The runnable `sqlite_sync` example uses actual SQLite reads and atomic body
transactions. It discards discovery state on failure/cancellation and reconstructs
from committed records; it never persists a discovery cursor as a receipt. It
also demonstrates reopen without duplicate commits. Preparation deliberately
rebuilds a small derived summary table once per turn: production hosts must budget
initial preparation and maintain their own persistent index. The example is not
a production schema, encrypted database, authenticated transport or signature policy.

Mesh Talk uses application-specific SQL selections and causal repair. Mesh Forum's
production protocol currently uses its own post index/revision exchanges;
its generic inventory and private mailbox compositions are integration tests.
These are independent-consumer evidence, not proof that every production path
uses the high-level driver. Preserve that distinction when describing maturity.

The optional blob session adapter intentionally uses mesh-connect framing and
errors. Use the transport-independent chunk interfaces for other carriers; extend
the session port only when a real additional carrier needs it.

## Shared finite-window pagination

`delta::WindowSelection<C, I>` and `WindowScan<C, I>` own bounded ordinal selection,
lookahead, pinned upper windows and completion/reset transitions. `DeltaCursor`
lets a host qualify its own binding and expose only `DeltaPosition`; updating the
position must preserve identity/epoch/host metadata. `WindowPage<C, I>` has the
same four-field page shape. The existing neutral `DeltaSelection`, `DeltaScan`
and `MailboxDeltaPage` names remain concrete convenience aliases.

An application with a historical protocol can implement this small cursor port,
retain its old serialized cursor and IDs, and impose its own request ceiling.
It must not omit historical binding fields when implementing `accepts`. The SDK
neither interprets those fields nor gains authority from a cursor. Page completion
is still source-local; the host commits bodies and cursor together.

Persistent range writers can call `Summary::combine_at_depth` with a known parent
depth. It checks leaf depth and count overflow without constructing synthetic
prefixes; `Summary::combine(Prefix, ...)` remains available and uses the same rule.
