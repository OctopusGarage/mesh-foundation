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
  are absent. `MFOM2`/`MFMSP1` distinguish new wire/disk profiles from old bytes.
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

## Policy and adapter separation

Neutral `AccountIdentity` no longer exposes creation timestamps or timestamped
restoration. Store lifecycle metadata in the application; raw signing material
and certificate operations remain available. Mesh Talk keeps its original
account profile, including historical lifecycle bytes.

Replace implicit sender-chain refresh with `prepare_rebase` and `install`.
Authenticate and authorize distributions in the application, choose its recovery
threshold there, and atomically persist the prepared checkpoint. Stale candidates
are rejected; skipped/consumed key semantics and checkpoint encoding are retained.

Mailbox `model`, `ports` and `client` are usable with no default features.
`client::sync_turn` now accepts `MailboxTransport`; the session convenience lives
at `wire::sync_turn`. Use the `wire` and/or `spool` features explicitly when needed.
`MailboxStore::delta_bounded` selects IDs under both count and raw-payload budgets;
`bodies` obeys the same budget and cursor source/snapshot. Custom adapters must
bound decoding before allocation and implement current access checks.

`MFOM2` replaces the earlier unreleased neutral MFOM1 profile: Delta/Get requests
carry allowances, and responses are checked independently. `read_get_response`
no longer takes a caller-supplied cost function; byte accounting is raw payload.
This does not change Mesh Talk's application-owned MTOM1/MTMSP1 compatibility.

Use `Spool::open_configured` for explicit `SpoolOptions`. Manual retention and
independent record/query limits support non-chat applications; convenience open
methods retain their earlier window-age behavior. `MailboxCursor::first_seen` is
removed; timestamps are storage-private. Source epoch still invalidates cleared
history. MFMSP1 journal bytes remain compatible and are fixture-tested.
