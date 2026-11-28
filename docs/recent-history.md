# Recent history, older pages and scalable range storage

The replication SDK now exposes `history::fetch_page`, `HistoryBinding`,
`HistoryCursor` and `HistoryLimits`. Defaults admit up to 200 records, 1 MiB of
body bytes and 2,000 examined source rows in one page. The host supplies a bounded
fetch callback and a synchronous atomic body/cursor commit callback. It owns
transport framing, record verification, durable storage and current permission.

This is an additive interface; existing bounded pull and full-range repair APIs
remain available. No timer, task or automatic old-history drain is created.

## Coverage and ordering

A recent page is a finite, authorized source snapshot. `next` requests older
records from that same snapshot, using an exclusive descending source-local
position. The binding includes source, application scope, authorization identity,
snapshot identity and immutable upper position. It cannot be reused across peers
or policy generations. A nonterminal empty filtered page must still advance.

`exhausted` means this source snapshot ended, never that all replicas contain the
same history. Applications keep page coverage and full repair state separate.
A source-local ingest position is not global creation time. Seven-day presentation
filters must not remove old authorization, keys, causal parents or late arrivals
from recovery. They are deliberately not the default consistency boundary.

The SDK validates the complete bounded response before invoking commit. Failure,
revocation or cancellation during fetch does not acknowledge a cursor. Commit
must persist bodies and continuation atomically; a failed commit must change
neither. Retry ingestion must be idempotent. Byte, record and examined-row limits
are independent, and a malicious page cannot acknowledge an advancing cursor
without progress.

## Disk-backed range reads

`range::RangeRead` supports fallible snapshot reads. `Checkpoint::for_reader`,
`valid_reader_binding`, `next_query` and `accept_reader` let SQL-backed hosts use
the canonical range codec without materializing an entire local `RangeIndex`. Summary-discovery completion
is not body durability: retain discovered IDs with the checkpoint until committed,
or discard the checkpoint after failure/cancellation.
The outer error from `accept_reader` is a storage failure; its inner error is a
protocol/invalidation failure. All needed local reads succeed before the frontier
is changed, so failed storage cannot be confused with an empty/equal range.

## Host integration

A persistent adapter owns its indexed storage, snapshot qualification and lease
lifetime. Keep derived authorization indexes separate from durable permissions;
requalify access when authority changes or the process restarts. Bound index
preparation and cleanup independently of page retrieval. An initial qualification
may still require O(N) work overall even when each turn is bounded.

Define the application protocol over an authenticated session, including explicit
unsupported, preparing, blocked and invalidated outcomes. Choose a byte allowance
that fits the carrier frame limit. Do not fall back to full inventory transfer
merely to render a recent page. Report oversized records and incomplete coverage
explicitly. Page retrieval establishes neither durable custody nor global equality.

Applications with causal records must validate dependencies before admission.
Keep key recovery, dependency repair and full-range reconciliation on separately
bounded schedules. Never substitute unauthenticated header hints for required
parent records. Recent navigation does not promise dependency-free cold rendering.

Account retained memory and persistent index storage separately. Bound cursor and
missing-ID frontiers, reclaim retired snapshots, and acknowledge progress only
after durable admission. Use range reconciliation to verify convergence rather
than inferring equality from an exhausted history page.
