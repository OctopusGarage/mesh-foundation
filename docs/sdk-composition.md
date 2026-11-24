# Composing Mesh Foundation

## Choose the smallest useful set of crates

The default features preserve the existing Mesh Talk source and wire interfaces.
New consumers can opt into a smaller dependency graph.

| Need | Start with | Internal dependencies |
| --- | --- | --- |
| Pinned-key encrypted frame exchange | `mesh-connect` with default features disabled | No other Mesh crate. Add `signed-discovery` for the existing device and account announcement format, which uses `mesh-identity`. |
| Inventory and bounded reconciliation primitives | `mesh-replication` with default features disabled | No other Mesh crate. Add `mesh-connect` for direct use of its pinned session, or implement the session interface for a different carrier. |
| Existing Mesh Talk reconciliation protocol | `mesh-replication/legacy-wire` | `mesh-identity` for the signed Event schema. Keep this feature for mixed-version Mesh Talk peers. |
| Device or account credential verification | `mesh-identity` | None |
| End-to-end content encryption | `mesh-e2ee` | None |
| Encrypted local record log | `mesh-storage` | None |
| Offline custody and incremental retrieval | `mesh-mailbox` | `mesh-identity`, `mesh-replication/legacy-wire` and `mesh-storage`. |
| Attachment chunks and stream verification | `mesh-blob` | None |

Arrows point from a dependent to its dependency. Dashed arrows are selected
by a feature. The mailbox requests the legacy Event profile explicitly.

```mermaid
flowchart LR
    C[mesh-connect] -. signed-discovery .-> I[mesh-identity]
    R[mesh-replication] -. legacy-wire .-> I
    R -. mesh-connect .-> C
    M[mesh-mailbox] --> I
    M -->|legacy-wire| R
    M --> S[mesh-storage]
    E[mesh-e2ee]
    B[mesh-blob]
```

Cargo unifies features. Turning off defaults in one declaration does not turn off
a feature requested by another dependency. Inspect the resolved graph with
`cargo tree -e features`. See [SDK delivery](sdk-delivery.md) for Git pins.

## Connect and replicate

Enable `mesh-replication/mesh-connect` to pass a
`mesh_connect::PeerSession<S>` directly to
`mesh_replication::session::send_have` and `receive_have`.
The session authenticates a previously trusted peer static key and supplies
encrypted, ordered frames over the caller's byte stream. Pick the key before
dialing. After an interrupted frame, create a new session.

For an SDK sibling checkout, a consumer that uses a pinned session and its own
record schema can start with:

```toml
[dependencies]
mesh-connect = { path = "../mesh-foundation/crates/mesh-connect", default-features = false }
mesh-replication = { path = "../mesh-foundation/crates/mesh-replication", default-features = false, features = ["mesh-connect"] }
```

A consumer testing a newly introduced feature should pin a revision that
declares it, or use direct path dependencies for all SDK crates in a disposable
checkout. An override against an older locked Git revision can fail during
feature resolution. Restore the consumer's Git pins and lockfile before
committing permanent changes.

1. Give the connection a reliable ordered stream such as TCP. Other ordered
   byte streams can use the same session. A differently framed carrier
   implements `ReconciliationSession` and supplies its own verified peer.
2. Choose the scope and authorize the local inventory before exporting it.
   Capture a finite view. The default `inventory_session::send_inventory`
   sends a canonical set with an explicit sender and a scope, and calls
   the authorization callback before each page. The matching
   `inventory_session::receive_inventory` verifies these fields. For
   streaming store views and other codecs, use `HaveSource::validate`
   to recheck authority before each outgoing page.
3. Bound the receive by frames, IDs per frame and total IDs. The Have stream
   preserves raw IDs, including duplicates. Use
   `receive_inventory` when the caller needs the final canonical set. It
   collects and sorts in place, rejecting duplicates across page boundaries.
   Use `CompleteInventory::from_ids` for the local authorized set and
   `remote.missing_from(&local)` for IDs absent locally.
4. Choose the record exchange. With the default profile, the reader calls
   `pull_over_session`. The sender handles each request with
   `serve_record_request` and supplies a loader that checks the authenticated
   peer's current scope access. The SDK binds both frames to the scope and caps
   header, record and body sizes before copying response bodies. Applications
   with a different wire format can use `pull_missing` with their own fetch
   callback.
5. In either profile, the commit callback verifies content IDs, signatures and
   current authorization, then commits the whole batch atomically. A later
   failure leaves earlier commits intact and reports them in
   `PullError::committed`. Resume after reconnect with a fresh local
   inventory. An empty difference performs no fetch.

Mesh Forum's board test runs canonical inventory comparison and the default
record exchange on authenticated TCP. It verifies the board and author,
commits to an encrypted log, then reopens the log to verify persistence.
The host handles exchange dispatch and owns the connection lifetime.

The generic inventory is ordering and set-comparison machinery. It has no
record authorization, delivery receipt or application payload schema. The
legacy round owns the additional Mesh Talk negotiation and transfer rules
when interoperability with that protocol is required.

## What can be replaced today

| Role | Replacement point | Evidence and limits |
| --- | --- | --- |
| Underlying connection | Byte stream under `PeerSession` | TCP and in-memory duplex are exercised. Session encryption and frame limits stay the same. |
| Replication carrier | `ReconciliationSession` | Mesh Talk uses a policy-gated authenticated channel. Mesh Forum uses a pinned-key TCP session. |
| Replication data source | `HaveSource`, `ExportSource` and legacy round store interface | A source owns authorization, finite views and cursor semantics. The round owns wire framing and progress. |
| Mailbox query backend | `MailboxStore` | The standalone spool and Mesh Talk's indexed store both use the same request dispatch path. |
| Mailbox event schema and durable spool | Currently concrete | `RelayMailbox` and the provided spool use the existing signed Event and encrypted log. A new transport does not require a schema change. A different record schema would need its own envelope and durable adapter with independent compatibility evidence. |
| Content encryption, attachment storage and identity policy | Application composition | Replication carries opaque bytes and does not need decryption keys. Blob verification stages bytes through the application. Account trust and key rotation remain application decisions. |

## Naming and safe call order

- **Peer vs device vs account.** `PeerSession::peer_key` is the authenticated
  transport key. `DeviceCertificate::verify_for_device` verifies the
  account's signature over the expected device signing key. An application
  decides whether to trust the issuer before granting access.
- **Inventory vs Have.** Inventory is the general concept. Have is retained
  where the interface represents the paged exchange and historical frames.
  Use inventory in app-facing names and documentation. Qualify the two
  current Event ID types by module when interoperating with legacy bytes.
- **Custody vs delivery.** `Spool::deposit` and `durable_have` describe
  relay retention. They do not prove delivery to a recipient. Use `open`
  on the recipient to authenticate and unwrap the envelope.
- **Prepare vs commit.** `prepare_open` prepares a receive candidate.
  `install` checks the in-memory generation. A durable consumer checks the
  persisted base and commits the accepted event and new checkpoint together.
- **Append vs publication.** `append_durable` acknowledges a synced record.
  `BlobVerifier::finish` validates the staged stream. Only the application
  publishes the verified bytes. Retain and replay a contiguous prefix when
  resuming verification after restart.

For new typed entry points, prefer domain names over protocol abbreviations
and document the state transition an operation confirms. Keep existing public
names and compatible wire tags until there is a tested migration path.


## Further interface work

The optional session adapter removes repeated peer, send and receive forwarding
without introducing a shared transport crate. Keep the carrier swappable at the
existing session interface. Keep the mailbox's concrete envelope for the
existing wire format until another application needs a different signed record
schema behind the same retention lifecycle.

New consumers can start at the inventory-oriented receive entry point. The
low-level Have functions let a custom protocol retain its codec and admit raw
IDs before canonicalizing them.
Two Event ID forms, raw inventory IDs and signed legacy Event IDs, still need an
explicit conversion when crossing profiles. A future typed common ID requires
a migration plan for constructor names and existing callers.

The generic pull accepts either the SDK record exchange adapter or the
application's own fetch callback, together with a commit callback. A new record schema defines its authorization, ID verification and
transaction rules there. The SDK supplies selection, batching, response
validation and durable-progress accounting.

## Safe defaults and application parameters

- `DiscoveryConfig` binds a caller-selected IPv4 multicast group and UDP
  port. The bound socket reports the matching target and can refresh
  group membership after an interface change. The host schedules discovery.
- `DeviceCertificate::verified_issuer_for_device` returns issuer material
  only after verifying the expected device binding. Trust remains with
  the application.
- `RatchetState::ratchet_decrypt` changes live receive state only after
  authentication. Persisting a successful state alongside application
  records still requires the host's transaction.
- `BlobLayout` validates the total size, chunk size and index range.
  `BlobVerifier::for_layout` checks exact per-chunk lengths. Hosts
  select the maximum size and stage bytes until final verification.
- `EncryptedRecordLog::open_with_key_and_frame_limit` takes a key
  resolver and bounds each frame during reads and writes. A lower limit
  can reject an existing log and does not rewrite its stored records.
- `Spool::open_with_key_provider` uses the caller's key resolver and the
  spool's internal frame ceiling. The spool still owns its file
  backend, lease and cursor. `MailboxStore` remains the read adapter.
- The mailbox host owns connection admission, deadlines and idle retirement.
  Reuse one authenticated stream for sequential delta and body requests under
  the same negotiated scope. Recheck authorization for each request. After
  interrupted I/O, reopen the stream and resume from the last cursor whose
  selected bodies the consumer committed.

## Sparse blob reception

Authenticate the object manifest and bind a private staging store to that exact
version before constructing a `SparseBlobReceiver`. Use `restore` to rebuild
progress from staged chunks. Intersect `missing` with each authorized source's
advertised availability and exclude requests already in flight. Feed authenticated,
decrypted replies through `accept`; only successful durable writes advance
progress. Call `verify` before publishing the assembled object. The host owns
the transfer protocol, request concurrency, and final publication.

Use `DirectoryChunkStore` for private file-per-chunk staging, with one writer
per object version. `SparseBlobReceiver::download` rotates across the supplied
sources, bounds concurrent requests, and retries other sources for an
unavailable or invalid chunk. Hosts supply authorized availability snapshots.
Reopen the staging directory and call `restore` before resuming.

With the `mesh-connect` feature, a host uses `SessionChunkSource` on a
dedicated authenticated session and dispatches `serve_chunk` at providers.
The loader enforces current object access on each request, regardless of
whether the provider is an app or a relay. For private objects, providers
serve indexed ciphertext and the receiver wraps each source in
`EncryptedChunkSource`. Keep the plaintext staging directory private.
The transport's frame ceiling includes the AEAD tag, so choose a chunk
layout that fits before publishing a manifest.

On the dedicated session, `SessionChunkSource::discover` fetches the
availability snapshot in bounded pages before requesting data. Providers
dispatch both request types via `serve_request` and check object access
independently for availability and content. The host still discovers
candidate peers and determines which peer identities it can contact.
