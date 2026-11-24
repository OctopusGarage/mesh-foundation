# Recent history and scalable range synchronization research

## Recommendation

Implement recent history as an explicitly scoped, finite page, separate from full
history reconciliation. Default the initial page to 200 authorized records, keep
an older-history continuation, and report recent-page completion separately from
full coverage. Do not use a seven-day cutoff as a retention or consistency rule.

Matrix provides a useful precedent: limited timelines preserve gaps and expose
backfill tokens, while omitted membership/state changes still need to reach the
client. This is a protocol pattern, not a claim that Mesh Talk can reuse Matrix
state semantics. [Matrix sync and messages](https://spec.matrix.org/v1.19/client-server-api/).

## Ordering and cursors

Use a stable indexed source-local ordering with a deterministic tie-breaker,
immutable upper watermark and explicit cursor identity. A local commit sequence
measures ingestion order, not global message creation time. A seven-day filter
based on sender timestamps can hide late events or skewed clocks; if offered,
label it a presentation preference and keep incremental ingestion independent.
Bind continuations to store identity, conversation, authenticated peer, policy
revision, query profile and snapshot generation. A cursor is not authority.

SQLite row-value comparisons support keyset pagination. OFFSET work grows with
the skipped prefix, while indexed continuation can seek directly. Avoid loading
all IDs, sorting the whole history or COUNTing an unindexed filtered set merely
to return the first page. [SQLite row values](https://sqlite.org/rowvalue.html).

## Persistent summary tier

The current process-wide in-memory range admission limit is a real scalability
boundary. Persist derived IDs and branching summaries in SQLite under a qualified
view/job identity. Store only branching/leaf summaries (compressed trie) rather
than 256 materialized nodes per ID. A summary lookup must perform bounded indexed
seeks; leaf IDs must be capped independently. Update derived IDs, summaries,
revision and build cursor atomically, so cancellation cannot expose a partially
updated root. Rebuilds remain explicit bounded work after authorization, removal,
retention or schema-generation changes. This recommendation is architectural
inference from the existing compressed index and SQLite indexing behavior, not a
SQLite-provided Merkle-tree feature.

SQLite can use multi-column indexes for conjunctions and ordering; a covering
index can avoid extra table reads. Validate actual queries with EXPLAIN QUERY
PLAN, looking for indexed SEARCH rather than a full SCAN or temporary sort.
Query-plan checks are evidence about access paths, not latency benchmarks.
[Query planner](https://sqlite.org/queryplanner.html),
[EXPLAIN QUERY PLAN](https://sqlite.org/eqp.html).

## Security and dependency lane

Recent message selection must come from the existing authorized outgoing view,
not raw event rows. Before disclosing either bodies or IDs, revalidate current
access. Keep membership, revocation, cryptographic keys and causal dependencies
outside the recent-message filter, under their own bounded recovery budget.
Authorization changes invalidate qualified derived views and remote comparisons.
A recent page or a source-local journal cursor cannot prove multi-peer equality.

## Required acceptance cases

- First 200-record page from a large group without a full RAM inventory.
- Backfill after 201+ records, stable boundaries during concurrent arrivals,
  duplicate timestamps and late old-timestamp ingestion.
- Wrong peer/scope/policy/snapshot continuation rejection.
- Interrupted page commit resumes without lost bodies or cursor advancement.
- Persisted range roots match the canonical in-memory digest and survive reopen.
- Index insertion touches a bounded path; large equal histories use cached roots.
- Key/control recovery and revocation remain correct when old message bodies are
  absent; limited recent coverage never becomes full-history equality.
