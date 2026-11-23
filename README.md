# Mesh Foundation

Rust libraries for peer connectivity and decentralized replication.

Use `mesh-connect` for pinned-key Noise XX sessions over a reliable, ordered
byte stream. Pass a connected TCP stream or an equivalent transport. Provide
a trusted public key to dial and an admission policy to accept. Discard a
session after a failed or cancelled in-flight frame. The package also exposes
LAN sockets, signed discovery announcements and queries, and candidate
selection. A host runs discovery, provides its trust policy and schedules
reconnects. Mesh Talk composes the same frame primitives with its existing
application handshake and session pooling.

`mesh-replication` provides bounded ID inventory assembly, canonical digests,
and exact set differences. It is independent of `mesh-connect`. It also includes the requester and responder for the existing Mesh Talk wire protocol.
Callers supply the current authorized store and authenticated session.

Run `cargo test --workspace` to exercise real loopback TCP and peer-key
rejection.

## Package boundaries

| Package | Responsibility | Extraction status |
| --- | --- | --- |
| `mesh-connect` | Authenticated frame sessions, LAN discovery, route selection and connection lifecycle | Pinned-key sessions, LAN/IPv6 sockets, signed discovery codecs and candidate selection available. Hosts own routing configuration and relay coordination. |
| `mesh-replication` | Scoped event inventories, bounded reconciliation, resumable export and import | Inventory, exact differences, bounded requester and responder, and legacy wire codec available. Applications supply authorized persistence via adapters. |
| `mesh-e2ee` | Content encryption state machines and checkpoint codecs | Pairwise ratchets, group sender keys and checkpoints available. Hosts own key distribution and atomic application commits. |
| `mesh-identity` | Signing, credential verification, and separate node and author identities | Device and account identities plus device certificate verification available. Account trust, enrollment and recovery policy belong to the application. |
| `mesh-storage` | Durable transaction and checkpoint adapters, including encrypted local storage | Record encryption and durable append/rewrite available. Applications own indexed data models and profile lifecycle. |
| `mesh-mailbox` | Opaque recipient-addressed deposits, retrieval, expiry, quotas and custody confirmation | Opaque envelopes, negotiated delta requests, quota policy and durable per-mailbox spool available. Hosts own admission, expiry scheduling and federation. |
| `mesh-blob` | Content verification, bounded chunk transfer and resumable retrieval | Chunk cryptography and integrity available. Hosts own transfer scheduling, manifests, attachment consent and retention. |

Replication owns its session and store contracts. A composition adapter binds a
connection and a store to those contracts. The replication engine handles signed
public records and encrypted records without access to content decryption keys.
Identity supplies verified principals, while each application supplies the
policy deciding what those principals may read, write and export.

Extract implementations alongside their compatibility tests. Mesh Talk adapters
preserve existing wire formats, event IDs and persisted checkpoints. Validate
packages through a forum consumer and existing Mesh Talk paths before publishing
a stable interface. For encrypted content, preserve the atomic commit of an
accepted event and its updated receive state across failures and restarts.

The Mesh Talk-specific mailbox envelope uses the device identities in
`mesh-identity` and the compatible signed event codec under
`mesh-replication::legacy_event`. It can be carried by any suitable transport.
