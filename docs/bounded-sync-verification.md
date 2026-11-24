# Bounded synchronization verification

Development candidate: 0.1.0-alpha.2. Not published; alpha.1 and its tag are unchanged.

## SDK checks

Verified locally with Rust 1.99.0:

- Workspace all-feature suite: 251 passing tests including doctests.
- Replication with default features disabled: 47 passing tests including doctests.
- Final targeted driver suite: six tests pass after progress-reporting refinement.
- All-target/all-feature Clippy with warnings denied; rustdoc warnings denied.
- Release policy tests, alpha.2 tag check, actionlint and whitespace checks.
- GitHub SDK CI passed on Linux, macOS and Windows for commit `4bf8f92`
  ([run](https://github.com/OctopusGarage/mesh-foundation/actions/runs/45368238771)).
- Runnable bounded_sync example commits one record using two bounded requests.
- An additional real TCP regression passes: two budgeted turns reuse the same
  authenticated session and resume from committed inventory without omissions.

The behavioral regressions include 201 records over two direct pull turns,
aggregate bytes/RPC limits, failed and cancelled batches, compressed-index
reconstruction after shuffled insertions/removals, cached equality, bounded remote
queries, invalid frames/authorization/snapshots, and same-revision index replacement.
Mailbox tests cover 201 records across page-aligned turns, failed commit, reset,
nonadvancing partial pages and immutable snapshot upper-watermark enforcement.

A separate SDK review found no blocking correctness issues. Its progress-reporting
finding was addressed by including DriverProgress in invalidation errors.

## Scope and performance limits

Direct bounded pull still takes precomputed complete inventories. Use the indexed
driver to avoid full remote inventory exchange. Range discovery is inbound
coverage, not bidirectional equality. The index is in-memory; hosts own durable
index maintenance, bounded rebuilds and resource accounting. Local summary counts
are not a measurement of a remote SQL backend's rows or disk I/O.

Additional leaf-batch tests verify that one RPC discovers 32 missing IDs and
rejects forged digests, duplicate/out-of-prefix IDs and oversized frames.

No p95/p99 latency or throughput claim is made. Tests verify deterministic budget
and cache/index invariants, not production performance under every workload.

## Application integration

Mesh Talk integration is a separate source change and test run in the adjacent
repository. Do not infer application behavior from SDK test success. Its final
runtime verification is recorded there; the source application must explicitly
upgrade or include these interfaces. See bounded-sync.md for host obligations.

## Recent pages and fallible stored readers

The all-feature workspace suite passes 251 tests including doctests after adding
six recent-page/reader integration tests and a fallible-reader unit regression.
The new cases cover 200+1 pages, source/snapshot/policy mismatch, independent byte
and scan limits, failed commit, revocation, cancellation, malformed continuations
and storage errors without frontier advancement. Workspace Clippy with warnings
denied also passes. See [recent history](recent-history.md) for coverage semantics
and the separate application disk-backed reference implementation.
