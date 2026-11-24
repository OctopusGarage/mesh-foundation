# Bounded synchronization implementation plan

Goal: implement the approved bounded/incremental group synchronization design.

Architecture: additive SDK pull/driver and indexed range interfaces preserve existing codecs. Application integration separately enforces fair scheduling and explicit incomplete outcomes. Mailbox retains source-local cursor semantics.

1. Add tests for per-turn record/byte budgets and cancellation-safe fresh-inventory resume in crates/mesh-replication/tests/pull.rs; implement additive APIs in pull.rs and its session adapter. Preserve drain-all compatibility entry points.
2. Add indexed canonical prefix summaries and bounded resumable reconciliation in a new replication module, with insert/delete cache updates and scope/view-bound continuation tests. Add negotiated, scope-bound range transport with bounded decoding rather than changing legacy frame discriminants.
3. Integrate SDK policy in Mesh Talk host scheduling/store paths and preserve old-peer fallback. Cover incomplete results, metadata/key recovery, authorization invalidation and source snapshot integration. Do not modify unrelated uncommitted work.
4. Align mailbox acknowledgement/progress with shared budgets without replacing its cursor identity. Add fairness and durable-ack tests.
5. Run targeted tests first, then formatting/Clippy/rustdoc and workspace verification. Document performance evidence and limits; do not publish an existing immutable alpha version.

Execution is authorized by the user's approval of docs/plans/bounded-incremental-group-sync.md. Track actual completed scope and unresolved gaps; do not claim runtime integration based only on SDK helpers.
