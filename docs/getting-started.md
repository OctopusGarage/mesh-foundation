# First integration

This guide is for the **unreleased alpha.2 source interfaces**. It is not a
recipe for the published alpha.1 packages. For registry alpha.1 installation,
use [delivery](sdk-delivery.md#registry-consumers) and that release's documentation.
Never copy an alpha.2 example into an alpha.1 project and assume compatibility.

## Choose one first entry point

Start with the smallest operation matching the task. Lower-level interfaces are
for consumers with distinct storage or protocol requirements, not prerequisites
for every integration.

| Task | Start here | Example / next reference |
| --- | --- | --- |
| Sign or verify opaque bytes | `mesh_identity::device::DeviceIdentity` | `device_identity` |
| Connect an admitted peer over TCP or another ordered stream | `mesh_connect::PeerSession::connect` / `accept` | [composition](sdk-composition.md#safe-composition-order) |
| Compare a small, already bounded complete set | `CompleteInventory` | `inventory` |
| Sync bodies with an in-memory canonical index | `driver::SyncDriver::turn` | `bounded_sync` |
| Sync bodies into a maintained persistent index | `reader::prepare`, then host atomic commit | `persistent_peer`; `sqlite_sync` for storage/failure contracts |
| Read recent/older ordered records | `history::fetch_page` | [recent history](recent-history.md) |
| Retrieve opaque offline records from a mailbox | `mesh_mailbox::client::sync_turn` | `typed_mailbox`; enable `wire` or `spool` only when needed |
| Resume verified content chunks | `SparseBlobReceiver` with `ChunkSource` / `ChunkStore` | [blob composition](sdk-composition.md) |
| Encrypt pairwise or multi-recipient content | `mesh_e2ee::ratchet` / `sender_key` | Authenticate/distribute keys and commit the candidate with the host event |
| Append encrypted records | `mesh_storage::record_log` | Host owns keys, indexes and cross-model transactions |

Use complete inventory only when the whole finite set fits the chosen memory
budget. `SyncDriver` is the convenient in-memory path, not a persistent SQL
adapter. `reader::prepare` does not automatically fetch bodies or own a database.
Use the low-level `range` checkpoint interfaces only for genuinely different
repair policy. None of these operations starts a hidden background sync loop.

## Select a source version

Use one reviewed full Git SHA for every SDK dependency. Check it out before
running the examples; do not float production dependencies on `main`.

```sh
git clone https://github.com/OctopusGarage/mesh-foundation.git
cd mesh-foundation
git checkout FULL_REVIEWED_COMMIT_SHA
cargo run --locked -p mesh-foundation-identity --example device_identity
```

Replace `FULL_REVIEWED_COMMIT_SHA` with the actual reviewed revision containing
this guide. Until public launch, Git access requires repository permission;
registry alpha.1 does not. Never put credentials into dependency URLs. The
[delivery guide](sdk-delivery.md#develop-and-consume) owns Git, local `path`,
root-level patches and coordinated upgrade instructions. `mesh-identity` is the
local dependency alias; the published package is `mesh-foundation-identity`.

## First persistent peer exchange

Run the `persistent_peer` example with its explicit session integration feature.
It is a small application over SDK interfaces, not a new SDK runtime. It binds
loopback TCP listeners, pins peer keys and stages bounded opaque records through
the maintained SQLite composition. The host owns peer admission, body validation
and the transaction; trusted peer identity and content hashes do not by themselves
prove application authorship or permission.

```sh
cargo run --locked -p mesh-foundation-replication --features mesh-connect --example persistent_peer
```

The example must establish these observable results: an initial transfer reaches
the durable store; reopening does not insert duplicates; a later source append is
retrieved over a fresh authenticated session. It deliberately chooses a 32-record
reader allowance and a 60 KiB body-batch ceiling to fit one wire batch; these are
example-host choices, not changes to the SDK defaults. Demo identities are
ephemeral and freshly pinned by the host; applications persist their own keys
and trusted peer bindings. The persisted index is updated with
bodies instead of rebuilding all summaries during each query. Check the output
and assertions, not merely a successful connection.

### Use it from a separate Cargo project

The same example can run as an ordinary consumer, outside this monorepo. From
its reviewed checkout, create an isolated demo project with local path dependencies:

```sh
mesh_sdk_checkout=$(pwd)
mesh_demo_dir=$(mktemp -d)
mkdir -p "$mesh_demo_dir/src/support"
cp crates/mesh-replication/examples/persistent_peer.rs "$mesh_demo_dir/src/main.rs"
cp crates/mesh-replication/examples/support/sqlite_store.rs "$mesh_demo_dir/src/support/"
cat > "$mesh_demo_dir/Cargo.toml" <<EOF
[package]
name = "mesh-persistent-demo"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
mesh-replication = { package = "mesh-foundation-replication", path = "$mesh_sdk_checkout/crates/mesh-replication", features = ["mesh-connect"] }
mesh-connect = { package = "mesh-foundation-connect", path = "$mesh_sdk_checkout/crates/mesh-connect", default-features = false }
mesh-identity = { package = "mesh-foundation-identity", path = "$mesh_sdk_checkout/crates/mesh-identity" }
rusqlite = { version = "0.42", features = ["bundled"] }
sha2 = "0.10"
tempfile = "3"
tokio = { version = "1", features = ["macros", "rt", "net", "time"] }
EOF
cargo generate-lockfile --manifest-path "$mesh_demo_dir/Cargo.toml"
cargo run --locked --manifest-path "$mesh_demo_dir/Cargo.toml"
cargo test --locked --manifest-path "$mesh_demo_dir/Cargo.toml"
```

Expected output includes `2 initial, 0 duplicate, 1 after append; 3 durable records`.
The test target additionally checks rejected peer pins, denied commit without
cursor advancement, body-receive cancellation followed by fresh-session retry,
and catchup across several bounded turns. The demo uses a
temporary database: persistence is verified by closing and reopening it within
the executable, not by leaving personal data behind. Choose a host-owned durable
path and protected peer keys in your application. Use Git/release dependencies
from the delivery guide when integrating; do not commit these machine-specific
absolute paths into the real application's manifest.

Use `sqlite_sync` to examine the storage mechanics separately:

```sh
cargo run --locked -p mesh-foundation-replication --example sqlite_sync
cargo test --locked -p mesh-foundation-replication --example sqlite_sync
```

These examples illustrate one scope, one writer and append-only content. They
are not an encrypted production schema, concurrent snapshot manager, account
trust enrollment, author-signature system or performance benchmark. Bring your
own schema and current authorization. For other databases, implement `RangeRead`
using a pinned authorized selection and maintain canonical summaries with writes.
The opt-in `test_support::check_reader` checks sampled mathematical agreement;
it does not certify transactions, permissions or crash recovery.

## The host handoff

1. Admit the authenticated peer and bind one authorized scope/source view.
2. Prepare a bounded turn against the pinned local reader. Treat missing IDs as
   discovery facts; they are not durable received records.
3. Fetch only the selected bodies under record, byte, request and decode limits.
   For one session batch, use `pull::fetch_records`: it validates exact IDs and
   response limits without a commit callback or a durability claim.
   Validate record identity and application signatures/schema where required.
4. In the host write transaction, requalify current permission and source facts;
   atomically accept the bodies and any receive state. Publish continuation only
   after success. Failed acceptance must not expose partial progress.
5. Schedule another turn through the existing application owner. If the reader or
   source view retired, reauthorize and restart from committed data. Do not equate
   zero bodies with completion, or one source's completion with global equality.

The [composition guide](sdk-composition.md#commit-gated-persistent-reader-turns)
owns detailed snapshot, cancellation and commit requirements. A callback returning
`Ok(())` is the host's assertion of durable acceptance, not proof supplied by the
SDK. The SDK cannot make an unrelated database transaction atomic for you.

## Failure handling and safe retries

| Outcome | Host action |
| --- | --- |
| `More` / incomplete turn | Retain valid continuation and schedule within the host budget |
| No-body navigation | Continue if incomplete; do not claim equality |
| Invalidated source or reader | Discard optimization, reauthorize and restart from durable data |
| Local read error | Propagate the error; never substitute an empty summary |
| Body/frame/record rejection | Reject uncommitted work; preserve prior accepted progress |
| Record allowance too small | Choose an explicit larger allowance or another profile; do not loop unchanged |
| Commit rejection/failure | Roll back bodies and receive state; do not install the prepared checkpoint |
| Cancellation during a session frame | Retire the session and dial a new one; never reuse an interrupted frame |

Use typed variants for retry policy and error source chains for diagnosis. Log
low-cardinality categories rather than secret keys or decrypted payloads. Do not
convert all failures into “retry” or “no new data.” Defaults bound one turn; they
do not establish application scheduling, retention or a seven-day recovery floor.
