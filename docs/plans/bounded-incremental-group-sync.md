# Bounded incremental group synchronization design

Status: approved for implementation; see bounded-sync-implementation.md and ../bounded-sync.md for actual interfaces and verification.

## Findings from current code

- `pull::pull_missing` limits each fetched batch, but loops through the complete
  missing set before returning. Batch size is not a total synchronization budget.
- `session::receive_inventory` builds a complete inventory; page/frame limits
  bound admitted inputs but do not provide a resumable partially reconciled state.
- `legacy_round::Requester::step` probes a whole-set fingerprint, then exchanges
  full Have sets after a mismatch. The code explicitly records O(N) inventory
  cost per reconciliation round. A large round-count ceiling is not an interactive
  scheduling policy.
- Existing mailbox delta scans bind a finite upper watermark and distinguish
  complete/reset. Their source-local cursor is not a universal group frontier.
- Application notification, authorization, durable storage and scheduling belong
  to the host; the SDK currently supplies lower-level exchange primitives.

## Recommended product semantics

Preserve eventual complete synchronization of the authorized retained history,
while limiting each foreground/background turn. First-load recent history and
full-history convergence are distinct states. Never label a truncated selection
as a complete group inventory or persist a global equality claim for it.

A product may display the latest 200 messages initially and expose older history
through explicit paging. Seven-day recent history may be an optional preference;
it must not silently redefine retention, membership access or recoverability.
No automatic unbounded historical drain follows a first-page load. Older history
is loaded on user request or through explicitly budgeted, fair background repair.

## Options

1. Hard latest-200/seven-day protocol cutoff: smallest initial traffic, but loses
   completeness and can omit late records, control dependencies and missing keys.
   Not recommended as the default consistency policy.
2. Total work budgets with continuation: fixes unbounded body draining without a
   wire-format change to the generic pull. Does not alone remove full inventory
   cost. Recommended first milestone.
3. Persistent bounded range reconciliation plus peer-scoped delta acceleration:
   bounds difference discovery as well as transfer. Requires source indexing,
   new negotiated frames and application integration. Recommended second milestone.

## Milestone 1: resumable work budgets

Add an additive bounded-pull entry point. Preserve the existing public function
as an explicit drain-all API for compatibility, document that behavior, and use
bounded defaults in a new high-level driver. Do not silently truncate legacy Have.

Initial configurable candidate defaults, to validate with representative traces:

- At most 200 event records committed per scheduling turn.
- At most 1 MiB of opaque response bodies per turn, independently of record count.
- Bound inventory pages, examined source rows and round trips separately; start
  with 8 pages and 2,000 examined rows per turn for evaluation, not a benchmark claim.
- Host deadlines and cooperative yields prevent one group from monopolizing work.
  Bounds apply to an entire turn, not separately to every nested batch.
- A record larger than the allowed body/frame budget produces an explicit outcome;
  never bypass ceilings, advance its cursor or silently drop it.

Return explicit progress: up-to-date within the agreed scope, more work with a
continuation, invalidated checkpoint, or failure with committed progress. Resuming
against a changed view must requalify/reconcile; a raw offset into a recomputed
missing set is insufficient. Advance durable continuation only with successful
batch commit. Keep partial progress after cancellation, and verify against the
fresh committed inventory on reconnect where a stable view cannot be retained.

Control/dependency recovery has a separate bounded lane. Membership, key changes,
causal parents and revocation checks cannot be removed by a recent-message filter.
Every lane and the aggregate turn remain bounded. A permanently unavailable record
must not block unrelated recoverable work forever.

## Milestone 2: bounded difference discovery

Provide an optional indexed range-summary interface over a stable authorized
view. Start with deterministic event-ID prefix ranges, range counts and strong
canonical digests. Compare summaries and subdivide only mismatched ranges; obtain
IDs only at bounded leaves, then pull missing bodies. The source must support
bounded indexed queries: rebuilding an entire tree or sorting history on every
probe would merely move the bottleneck to local CPU/I/O.

Bind comparisons to conversation, authorization/view identity, negotiated profile
and immutable snapshot/range boundaries. Identical digests are useful only for
identical scopes. Enforce examined-row, node, depth, page, byte and RPC budgets
against adversarial peers. Exceeding a budget returns more work, not equality.
Empty filtered pages may advance a cursor; nonadvancing cursors are errors.

The first index build and lost-index repair remain O(N) overall, but execute as
bounded resumable work. Define index ownership and atomic updates alongside event
commit. Cache invalidation includes removals, retention and authorization changes,
not only the latest append position or message count.

A store-local ingest journal/cursor can accelerate repeat exchanges with the same
peer once a baseline is established. Bind checkpoints to source identity, scope,
authorization epoch and journal generation; acknowledge only durable commits.
Different peers do not share cursors. Journal gaps/compaction trigger explicit
bounded repair. A source cursor alone cannot prove multi-peer set convergence.

## SDK, compatibility protocol and mailbox responsibilities

Introduce a shared high-level bounded driver: probe, reconcile, pull, commit and
return progress. The host supplies authorized indexed views, persistence,
notifications and scheduling. The driver must be usable by CLI/server applications
without embedding a desktop runtime or silently spawning perpetual background work.

Use the same policy/progress vocabulary across generic replication, the Mesh Talk
adapter and mailbox consumers; do not force their wire formats or cursor identities
to be identical. Mailbox cursors track a source-local spool prefix, while group
reconciliation tracks an authorized event set across multiple sources.

Keep existing legacy wire discriminants and behavior interoperable. Negotiate a
new capability/profile for range summaries and resumable bounded exchanges. Old
peers use the existing compatibility engine within host scheduling controls;
resource-policy exhaustion reports deferred/limited/unsupported, never success.
It may be impossible to guarantee both a strict small inventory budget and full
compatibility with an old peer that only supports full Have. State that limitation
explicitly instead of leaking partial Have as a complete set.

Mesh Talk integration must separately update the host store/index, high-level
scheduling, metadata/key recovery and tests. Shipping an SDK helper alone does not
change the application, and its vendored compatibility implementation may need an
explicit revision/snapshot upgrade.

## Required validation

- 201 missing records: first turn commits at most 200 and reports continuation;
  next turn finishes without duplicate delivery or omitted records.
- Independent body-byte, scan and RPC ceilings, including oversized single records.
- Cancellation, restart and concurrent arrivals during partial commit/resume.
- Same count/different IDs; late old messages; clock skew; causal/key dependencies.
- Authorization revocation and policy change invalidate cached views/checkpoints.
- Large equal history avoids scans on qualified cached probes; one insertion
  touches bounded range/index work rather than streaming the entire ID set.
- Many groups/slow peers receive fair scheduling and bounded aggregate memory.
- Journal compaction, retention removal, replica divergence and malformed cursors.
- New/new and old/new peers negotiate correctly without false convergence.
- Mailbox cursor advances only after the consumer durably commits selected bodies.

Measure IDs/bytes scanned and transferred, index rebuild/update cost, group
completion/continuation counts, cancellation recovery, p95/p99 turn duration and
peak memory. A small payload alone is not evidence of cheap difference discovery.

## Release scope

Implement and validate milestone 1 before milestone 2; each needs its own focused
implementation plan. Existing published alpha packages are immutable. Changes
ship in a new coordinated alpha with compatibility/migration notes. This proposal
does not authorize silently truncating old protocol responses or changing retention.
