# Public launch and release checklist

This checklist records ongoing release and repository administration gates. The
repository is public; seven coordinated alpha.2 crates are published on crates.io.
Use the [delivery guide](sdk-delivery.md) for the current publication workflow.

## Repository launch

- [ ] Identify active reviewers, a release owner and a reachable conduct/security contact.
- [ ] Enable and test GitHub private vulnerability reporting; update SECURITY.md with the verified channel.
- [ ] Review the complete Git history, fixtures and generated archives for secrets and personal data. If a secret is found, revoke it before removing it; rewriting history alone is insufficient.
- [ ] Verify ownership and MIT licensing of contributed and extracted code and third-party material; retain required notices. Inventory dependency licenses and resolve incompatible terms before distribution.
- [ ] Run SDK verification and dependency security workflows on the exact launch commit.
- [ ] Independently review cryptographic composition and public API misuse risks before claiming production security.
- [ ] Set the repository description and relevant Rust/SDK topics; verify public links after visibility changes.
- [ ] Configure main branch protection/rulesets: PR review, required SDK matrix checks, no force pushes and a documented administrator bypass policy.
- [ ] Protect release tags against unauthorized creation, movement and deletion.
- [ ] Configure Dependabot/security notifications and assign a maintainer to triage them.
- [ ] Review the visible bincode maintenance advisory and migration plan in the [dependency policy](dependency-policy.md).
- [ ] Change visibility only after reviewing GitHub's visibility consequences and the gates above.

## Coordinated source release

1. Follow [SDK delivery](sdk-delivery.md), update the shared version and changelog,
   and verify API, wire and durable compatibility against consumers.
2. Tag the reviewed main commit with its matching annotated `vX.Y.Z` tag.
3. Wait for SDK verification and source archive checks to pass on that exact tag.
4. Download the `mesh-foundation-vX.Y.Z` workflow artifact; verify SHA256SUMS and
   confirm COMMIT matches the tag's full commit SHA.
5. Run the SDK workflow manually on the existing version tag with
   `create_draft_release` enabled to create a draft GitHub Release after verification.
   Configure the `release` environment with required reviewers before using it.
   The workflow checks asset hashes and commit identity. Include curated
   changelog notes, compatibility/migration information, the full SHA, source
   archive, COMMIT and SHA256SUMS. Inspect it before publishing. Workflow artifacts
   expire; published Release assets provide a durable download entry point.
6. Verify assets can be downloaded by an unauthenticated user after public launch.
   Never replace an existing release tag to correct faulty source; cut a new version.

## Registry activation (separate reviewed change)

- [ ] Confirm all crate names, verified publisher identities, recovery access and owners.
- [ ] Review package metadata, README rendering and inclusion of license/notice files for every crate.
- [ ] Decide the minimum supported Rust version by testing the intended lockfile and feature sets; the pinned development toolchain alone is not an MSRV.
- [x] Add exact alpha versions alongside internal paths and verify the graph with Cargo metadata.
- [ ] Inspect `cargo package --list` for every candidate, including unexpected private files.
- [ ] Validate dependencies in publication order: identity/storage/E2EE, connect, replication, blob, mailbox. Optional and dev dependencies can constrain packaging too; recompute the actual graph rather than relying on this list alone.
- [ ] Run `cargo publish --dry-run` against the intended registry as dependencies become resolvable. A workspace path build does not establish registry readiness.
- [ ] Test a clean external consumer and docs.rs feature/target configuration.
- [ ] Review release automation with a protected environment and scoped or short-lived publishing credentials. Activate publishing only in that dedicated change.
- [ ] Add crates.io/docs.rs badges only after the corresponding packages and documentation exist.

## Ongoing engineering work

Parser fuzzing, restart fault injection, deeper API documentation, independent
security review and reproducible benchmarks are continuing projects rather than
launch badges. Choose bounded targets and measurable claims before adding tooling.
Keep advisory exceptions explicit, justified, owned and time-limited. Do not add
coverage thresholds or merge automation without an owner and a demonstrated need.
