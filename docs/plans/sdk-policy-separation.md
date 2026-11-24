# SDK policy separation implementation plan

Goal: finish the reviewed business-neutral boundary without moving generic crypto
or persistence into applications. Authorized by the user's follow-up request.
Published alpha.1 stays immutable; alpha.2 remains unpublished.

Architecture: opaque records and cursor/page DTOs belong to the mailbox model;
a typed transport port and budgeted client operate without RPC dependencies. A
versioned network adapter owns framing limits, while a configurable spool owns
storage limits and optional retention. Identity carries signing material, not
account lifecycle dates. Crypto exposes explicit checkpoint preparation/install;
Mesh Talk owns the policy deciding when recovery is authorized and needed.

Tasks:
1. Remove account creation metadata/clock rules from neutral identity; retain
   signing/certificate functionality. Mesh Talk's existing own account profile
   retains lifecycle metadata. Test compiler/API and signing invariants.
2. Replace sender-chain policy-shaped refresh with explicit, generation-checked
   rebase preparation/install. Keep checkpoint bytes and skipped/consumed-key
   semantics. Move the previous automatic threshold decision into Mesh Talk.
3. Extract mailbox model/store/transport ports. Integrity only validates hashes;
   implement bounded typed sync and a versioned RPC adapter with requested page
   and byte budgets. Reject invalid/oversized/nonadvancing responses before commit.
4. Configure spool record/query limits independently of RPC. Support manual and
   explicit first-window-age retention; document that per-record TTL belongs to
   a custom backend. Preserve durable ownership, quota and cursor invalidation.
5. Add semantic architecture tests and generic examples; test small turns, large
   local records, atomic failure/cancellation, versioned wire bounds and retention.
6. Verify SDK all-feature/minimal, lint/docs, Mesh Talk compatibility/session and
   actual forum consumer. Preserve unrelated dirty consumer work. Commit/push SDK
   on authorized main; commit own Mesh Talk changes locally on requirements-work.

No UI redesign, implicit workers, application event schema or new publication.


Implementation outcomes:
- Steps 1–5 implemented and verified; typed core and each optional adapter compile
  independently. SDK all-feature/default/minimal suites pass 216/211/196 tests.
- Code committed/pushed on SDK main at `072735a`; published alpha.1 tag unchanged.
- Forum's 26 tests pass against both local paths and that public Git revision.
- Mesh Talk source migration committed locally on requirements-work at `72aacc87`;
  managed SDK source is byte-identical to the reviewed code revision. Its 140
  channel tests, 88 protocol tests, late key-repair regression, strict Clippy,
  desktop compilation, structural standards and hygiene checks pass.
- Step 6 completed: session suite 111 passed/one existing scale case ignored;
  final evidence recorded in business-neutral-verification.md. No application
  release, new registry publication, UI changes or unrelated consumer commit.
