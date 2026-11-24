# Business-neutral SDK migration

Authorized direction: keep reusable transport, replication, cryptographic and durable
storage mechanisms in Mesh Foundation. Mesh Talk owns its chat event vocabulary,
legacy sync negotiation, post-office envelope, account certificate profile and
chat discovery announcements. Existing Mesh Talk bytes, identifiers and signatures
must remain unchanged; alpha.1 remains immutable. This is an alpha.2 source/API
migration and will not silently publish a replacement package.

## Boundaries

- Replication accepts opaque IDs/records, source snapshots and host authorization;
  no group membership, announcement, chat capability or attachment-consent enum.
- Connect owns authenticated byte sessions and bounded protocol/version capability
  descriptions. Discovery advertises opaque application protocol capabilities,
  never a built-in post-office role or account enrollment policy.
- Identity supplies neutral signing identities and certificate primitives. The
  old Mesh Talk signing domains and account identifiers move to its protocol crate.
- Mailbox stores/queries opaque host-admitted records under source-pinned cursors,
  quotas and atomic acknowledgement. It does not parse chat events or own recipient
  key distribution. Existing Mesh Talk relay wire/spool formats stay in the app.
- Storage, sender-key cryptography and blob verification remain reusable mechanisms;
  wire/checkpoint constants preserve cryptographic compatibility, not business policy.

## Execution

1. Copy exact historical codecs/tests into application-owned mesh-talk-protocol;
   redirect Mesh Talk adapters and remove SDK legacy exports/features/dependencies.
2. Replace signed discovery/account defaults with neutral SDK profiles; preserve
   the exact old profile in Mesh Talk. Add bounded capability negotiation primitives.
3. Replace mailbox event dependency with opaque records and codec/store/session seams;
   test generic durable quota/delta/commit behavior without any chat event kind.
4. Enforce dependency/source boundaries automatically. Run SDK all-feature/minimal
   tests and clippy, application protocol/session compatibility, and a forum consumer.
5. Rewrite current docs/examples/release checks to match the final surface. Document
   breaking alpha.2 migration explicitly; preserve historical release evidence.

No speculative plugin registry, application router, business event hierarchy or
application worker is introduced. Concrete application schemas are tested in their
own repository. Forum working changes and unrelated Mesh Talk documentation are
preserved. SDK commits stay on authorized main; application commits stay local on
its existing branch. Review the final diff and test evidence before completion.
