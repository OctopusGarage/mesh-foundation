# Mesh Foundation

Rust libraries for peer connectivity and decentralized replication.

The first extraction is `mesh-connect`: pinned-key Noise XX sessions over a
reliable, ordered byte stream. Pass a connected TCP stream or an equivalent
transport. Provide a trusted public key to dial, and an admission policy to
accept. Discard the session after a failed or cancelled in-flight frame.

The Noise and frame implementation comes from Mesh Talk. Its product-specific
identity proof, discovery, session pooling and offline custody remain in Mesh
Talk. The SDK's pinned-key profile is for new applications. It does not
implement Mesh Talk's application handshake or wire protocol.

`mesh-replication` provides bounded ID inventory assembly, canonical digests,
and exact set differences. It is independent of `mesh-connect`. Session
orchestration, authorization adapters and application integration are pending.

Run `cargo test --workspace` to exercise real loopback TCP and peer-key
rejection.

## Package boundaries

| Package | Responsibility | Extraction status |
| --- | --- | --- |
| `mesh-connect` | Authenticated frame sessions, LAN discovery, route selection and connection lifecycle | Pinned-key sessions available. Discovery and routing pending. |
| `mesh-replication` | Scoped event inventories, bounded reconciliation, resumable export and import | Inventory and exact differences available. Session and store integration pending. |
| `mesh-e2ee` | Content encryption state machines and checkpoint codecs | Mesh Talk ratchets available. Key distribution and application commits pending. |
| `mesh-identity` | Signing, credential verification, and separate node and author identities | Planned. Account trust, enrollment and recovery policy belong to the application. |
| `mesh-storage` | Durable transaction and checkpoint adapters, including encrypted local storage | Planned. Profile lifecycle belongs to the application. |
| `mesh-mailbox` | Opaque recipient-addressed deposits, retrieval, expiry, quotas and custody confirmation | Planned. Online sessions and recipient delivery have separate semantics. |
| `mesh-blob` | Content verification, bounded chunk transfer and resumable retrieval | Planned. Attachment consent and retention policy belong to the application. |

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
