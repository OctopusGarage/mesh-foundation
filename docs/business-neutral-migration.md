# Business-neutral alpha.2 migration

Alpha.2 is a published, intentionally breaking API release. Alpha.1 packages and
their immutable release tag remain unchanged. Do not reinterpret an existing
application database under new identity domains and assume identifiers will match.

## Application-owned compatibility

Keep existing event schemas, wire discriminants, fingerprint domains, certificate
profiles, discovery metadata and persisted envelope formats in application-owned
adapters. Generic SDK mechanisms are dependencies of the application, never the
other direction. Preserve compatibility with independent golden-byte fixtures and
mixed-version integration tests. The removed `legacy-wire` feature is not an SDK
compatibility layer; consumers own compatibility with their historical protocols.

## Generic SDK entry points

- Replication carries opaque `[u8;32]` IDs and `pull::Record` bodies. Implement
  verification and authorization in host adapters. Default features are empty.
- `device::{DeviceIdentity,PublicIdentity}::node_id` identifies a signing node;
  `node_id_from` derives it from its public signing key. Neutral node/account/
  certificate separators start with `mesh-foundation-`. Raw signing/DH key bytes
  are unchanged. Existing application fingerprints must stay in their old profile.
- Signed discovery uses `MFAN` v1 with a signed application namespace, <=32 unique
  protocol/version ranges and <=2048 opaque metadata bytes. Constructors and
  encoding are fallible. Existing application discovery formats are not silently reinterpreted.
- `capability` is available without LAN discovery or identity dependencies. It
  validates bounds/duplicates and chooses the highest common protocol version.
  Hosts own authenticated exchange, downgrade policy, routing and dispatch.
- Mailbox uses opaque scoped `Record` bodies, content integrity, source-pinned
  cursors and generic storage/session seams. Application envelopes and event bodies
  are absent. `MFOM2`/`MFMSP1` distinguish new wire/disk profiles from old bytes.
- Cryptography, blob transfer and encrypted storage keep their reusable state
  machines. Group membership, consent and lifecycle decisions remain host-owned.

## Adapter verification

Test signatures, encrypted payloads, durable cursor commits, authenticated session
resumption and attachment verification through the actual host adapters. Automated
SDK architecture guards reject application event types, application dependencies
and removed legacy features in production modules. Those guards enforce dependency
boundaries; they do not certify a consumer's authorization or compatibility policy.

## Policy and adapter separation

Neutral `AccountIdentity` no longer exposes creation timestamps or timestamped
restoration. Store lifecycle metadata in the application; raw signing material
and certificate operations remain available. Applications retain historical account lifecycle bytes in their own profile.

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
Existing application wire and disk formats require their own compatibility adapters.

Use `Spool::open_configured` for explicit `SpoolOptions`. Manual retention and
independent record/query limits support different application models; convenience open
methods retain their earlier window-age behavior. `MailboxCursor::first_seen` is
removed; timestamps are storage-private. Source epoch still invalidates cleared
history. MFMSP1 journal bytes remain compatible and are fixture-tested.
