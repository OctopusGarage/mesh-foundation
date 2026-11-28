# Bounded and incremental synchronization

These interfaces are new development work after 0.1.0-alpha.1. Published alpha.1
packages are immutable and do not include these APIs. Use the matching checkout
until a new coordinated development release is published.

## A turn is not a batch

`pull::pull_missing` and `pull_over_session` retain their existing drain-all
semantics for compatibility. Their `PullLimits` bound each batch, not total work.
New consumers should select `pull_missing_bounded`, `pull_over_session_bounded`,
or the indexed `driver::SyncDriver`.

`PullTurnLimits::default()` caps one caller-scheduled turn at 200 committed
records, 1 MiB of body bytes and eight requests. These are ceilings, not promises
that all allowances can be used. Frame ceilings, source page boundaries and
summary queries can end a turn earlier. Always inspect `complete`; successful
return with `complete=false` is retained work, not synchronized history.

An oversized record or rejected response produces an explicit error without
acknowledging it. Earlier atomic commits remain durable. Retrying the direct
bounded pull uses fresh committed inventory, not an offset into a changing set.
The host provides deadlines, fair per-group scheduling and durable transactions.
Do not immediately loop through every historical continuation after loading the
first screen; schedule older history only under explicit user/background policy.

## Indexed difference discovery

`range::RangeIndex` maintains canonical event-ID prefix summaries on insertion
and removal. It compresses unary paths and stores at most 2N-1 summaries, with a
separate ID index. Queries use cached branch/leaf summaries and fixed-depth hash
expansion, not full-history collection. Initial index construction remains O(N)
overall and must be budgeted by the host. An in-memory index does not make restart
rebuild free or provide a persistent database index automatically.

`range::Checkpoint` discovers only remote IDs absent locally. Scope, authorization
identity, local root/revision and pinned remote view invalidate stale progress.
A complete inbound comparison does not establish bidirectional convergence or
message delivery. Small ranges carry up to 32 sorted, unique IDs with a verified canonical digest
in a frame capped at 1,166 bytes. Single-record ranges retain a singleton shortcut.
This avoids a separate RPC for every missing message or every prefix bit.

`range::query_summary` and `serve_summary` operate on an already negotiated,
dedicated authenticated range session. They use distinct bounded frame profiles,
validate scope/view/prefix and recheck authorization. Hosts must establish profile
support and the immutable view token before sending queries; these frames must
not be injected into an old peer's unnegotiated legacy exchange.

## High-level driver

`driver::SyncDriver` retains bounded difference-discovery work and uncommitted
IDs, then pulls bodies under one shared turn budget. Its callbacks query remote
summaries, fetch requested records and atomically verify/commit them. Each
successful commit synchronously advances the index/checkpoint before another
await. Cancellation never acknowledges uncommitted bodies. Changed authorization
or externally changed local index invalidates the driver; rebuild from durable
records. Dropping the driver loses optimization, not acknowledged records.

Summary RPCs count against request/page/node allowances. `scanned` reports local
indexed-summary operations; it is not a claim about a remote database's SQL work.
The remote host must separately bound source preparation, query work and memory.
The driver does not spawn timers or background tasks. Application notifications,
connection admission, scheduling and separate bounded control/key recovery remain
host responsibilities.

Run the callback example with:

```sh
cargo run --locked -p mesh-foundation-replication --example bounded_sync
```

## Mailbox cursor acknowledgements

`mesh_mailbox::client::sync_turn` provides the same record/byte/request budget
vocabulary on an already negotiated mailbox session. The callback must durably
commit the whole selected page and its cursor together. No callback success means
no cursor acknowledgement. Empty pages can still advance a source snapshot.
A reset is explicit `CheckpointInvalidated`; reauthorize and start bounded repair.

Existing mailbox frames cannot advertise an arbitrary smaller Get byte budget or
acknowledge only part of a page. The client conservatively reserves the protocol's
full body-response budget and can stop before 200 records (for example after
three full 64-ID pages). It never advances over the remaining page. Too-small
limits fail explicitly instead of producing an endless zero-progress loop.

Mailbox completion covers that source's finite spool snapshot. It does not prove
that every group device has the same history. Keep cursor source/epoch identity,
group reconciliation state and delivery acknowledgements distinct.

## History policy and compatibility

Recent-200 UI loading and an optional seven-day preference do not redefine which
history exists or who is authorized to read it. Membership/key events, causal
parents, late arrivals and recovery controls cannot be discarded by that filter.
A partial inventory must never be encoded as a complete legacy Have set.
Old-peer policy exhaustion reports limited/deferred rather than equality; adding
a bounded SDK helper does not itself change an application protocol.

Run the replication tests and feature matrix documented in [Contributing](../CONTRIBUTING.md).

## Recent pages and storage-backed readers

For explicit recent-first/older navigation use [history::fetch_page](recent-history.md).
For SQL-backed range snapshots use `range::RangeRead` and the fallible checkpoint
reader methods. The original `SyncDriver` continues to own its in-memory index;
the storage adapter owns snapshot consistency, durable updates and disk limits.
