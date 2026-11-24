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
position. The binding includes source, conversation scope, authorization identity,
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

Mesh Talk's reference integration uses an encrypted, disk-spilling SQLite
compressed tree under the existing qualified selection owner. It preserves the
canonical digest, inserts at most 32 IDs atomically per page, and bounds cleanup
and restart after failed builds. It removes the former 32 MiB/32,768-ID RAM
admission barrier. It uses the existing encrypted scratch pager, not a plaintext
temporary database. Scope retirement invalidates leases and reclaims rows in
bounded slices.

This derived tree lives for the authorized selection's lifetime. It is not a
cross-restart cache of permissions: process restart requalifies and reconstructs
its derived selections. First authorization preparation can still examine O(N)
headers overall, in bounded work units. Recent retrieval does not bypass expiry,
privacy, receipt suppression or disclosure closure to avoid that preparation.

## Application profile

Mesh Talk negotiates a separate recent-history profile over an authenticated
session. It returns at most 200 selected Message records, capped further by the
60 KiB body/transport-frame budget, with an opaque older-page cursor. It never
falls back to full Have merely to render a recent page. Unsupported, preparing,
blocked and invalidated/reset states are explicit. An oversized item can report
a gap and an explicit continuation to older items; it is not silently counted
as complete coverage.

The foreground `Node::fetch_channel_history_page` and desktop history action use
this profile. Existing key/control recovery and full-group repair retain their
own bounded scheduling. A recent-page result establishes neither custody nor
whole-history equality. The existing local history display window is a separate
presentation limit; remote paging does not turn it into an unbounded UI list.

## Causal gaps in the reference application

Mesh Talk's current canonical event graph requires parents before child admission.
A recent window can therefore download signed messages whose older external
parents are missing. It does not relax signature, ancestry or authorization checks
to make those messages appear committed. The page reports Blocked and can expose
an explicit older navigation cursor across that gap; such a cursor is not a
storage acknowledgement for rejected messages. The UI retains the gap warning
and offers a fresh recent retry after older navigation ends. Existing budgeted
repair can supply dependencies. With a long causal chain, resolving that closure
can require older message bodies; the current signed event format cannot replace
them with unauthenticated header hints. Immediate ancestor-free cold rendering
is not promised by this optimization.
