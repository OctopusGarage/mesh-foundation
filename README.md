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
