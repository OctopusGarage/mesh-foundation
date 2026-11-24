# Recent history and scalable range indexing

Execution: authorized by the user's request to evaluate and finish the omitted work.

The previous work bounded turns, but an in-memory full-history index still blocks large groups. Implement a disk-backed authorized range-index tier with bounded updates and queries. Keep canonical digests, scope/policy validation and immutable leases. SQL failures must propagate; they must never become empty summaries or equality. Existing in-memory SDK callers remain supported.

Add a separate recent-history page contract: default 200 message records, explicit older-page continuation, immutable source cutoff, descending stable source order and source/scope/authorization binding. Page coverage is distinct from whole-group convergence. Controls and dependencies use existing authorization/key recovery, never a seven-day deletion/filter. No trusted wall-clock cutoff: the event format has causal clocks and author-controlled wall-clock timestamps, so seven days is not a sound consistency boundary.

Tasks:
1. Capture primary-source research and document ordering/coverage semantics.
2. Add and test storage-independent bounded recent-page validation/commit APIs in the SDK. Commit and cursor acknowledgement are atomic; cancellation cannot acknowledge a page.
3. Replace application full-history RAM index admission with a SQL-backed compressed range index, bounded page transactions and fallible query integration. Test beyond the former 32,768-ID capacity, canonical equivalence, rollback, invalidation and leases.
4. Implement negotiated application recent/older pages over authorized outgoing selections; connect product history paging and existing scheduling. Retain full-history repair as an explicit distinct capability, and report limitations truthfully when source preparation needs bounded work.
5. Run regressions, Clippy, formatting, repository gates, and document actual cold-preparation and restart behavior. Preserve alpha.1. Do not publish alpha.2 or unrelated application commits implicitly.
