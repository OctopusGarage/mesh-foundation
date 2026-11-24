# Open-source readiness research

Research date: 2026-11-24. This is a proposed readiness baseline, not a claim that the repository is public, published, audited or production certified.

## Current evidence

The seven-crate workspace already has an MIT license, shared version, lockfile, pinned toolchain, changelog, composition and delivery guides. `.github/workflows/sdk.yml` checks formatting, Clippy, all-feature tests and selected minimal feature combinations on Linux, macOS and Windows. Tagged builds produce a source archive and checksums. Existing documentation explicitly describes private Git delivery; registry publishing is disabled. This is a useful foundation and should be preserved until a separate publication decision.

## Recommended sequence

### 1. Public repository baseline

- Explain audience, supported use cases, maturity and trust boundaries before the detailed package table. Provide a minimal reproducible consumer example, crate selection guide, feature matrix and links to local API documentation.
- Add CONTRIBUTING with exact local checks, test conventions, compatibility expectations, issue/PR process and review expectations. Add issue forms and a concise PR template; avoid requesting sensitive logs or keys.
- Establish SECURITY with a working private reporting channel, supported versions and disclosure process. Verify the channel before public launch; do not invent response-time guarantees.
- State maintenance, ownership and decision process. A short governance document is sufficient; adopt a code of conduct only with a real enforcement contact and process.
- Review Git history, fixtures, archives, copyright provenance and copied consumer material before changing visibility. A clean current tree alone does not establish clean history.
- README badges should report real signals: existing CI workflow and MIT license now; crates.io/docs.rs only after those resources exist. Avoid unsupported coverage, audit or stability claims.

These recommendations adapt the onboarding and contribution organization of [Tokio](https://github.com/tokio-rs/tokio/blob/master/CONTRIBUTING.md) and the SDK documentation entry points in [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust). They are design recommendations, not requirements imposed by those projects.

### 2. Verifiable CI and dependency maintenance

- Retain the current operating-system matrix and independent feature combinations. Add a rustdoc build with warnings denied and verify examples as executable documentation. An all-features build alone does not prove default or minimal configurations work.
- Declare an MSRV only after testing the exact version with the lockfile and intended feature sets. Document whether raising it is a compatibility change. Cargo's [rust-version reference](https://doc.rust-lang.org/cargo/reference/rust-version.html) describes the field and dependency-policy implications.
- Add scheduled dependency advisory checks and Cargo/GitHub Actions update automation. Treat advisory findings as triage input, with explicit documented exceptions rather than silent ignores.
- Keep workflow token permissions minimal. Pin external actions to verified full commit SHAs and automate their updates; avoid privileged execution of untrusted PR code. See GitHub's [secure use reference](https://docs.github.com/en/actions/reference/security/secure-use).
- Add cancellation of superseded PR runs and bounded job timeouts. Cache only as an optimization; correctness must not depend on a warm cache.
- Configure required checks and protected release tags in GitHub settings. Repository YAML cannot establish that server-side policy is active.

### 3. Public API and security review

Audit each crate against the [Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html): examples, error behavior, panic conditions, thread-safety, standard traits and compatibility. Prioritize entry points and security-sensitive state transitions over indiscriminate lint suppression.

Document threat model and non-goals for identity, E2EE, blob and mailbox APIs: authentication versus authorization, application-owned key distribution, memory/disk bounds, replay handling, cancellation and atomic durable state. Add reproducible misuse and recovery examples. Existing compatibility tests are valuable evidence but do not replace independent cryptographic review.

Fuzz parser and framing boundaries after defining bounded inputs and selecting stable targets. Add benchmarks only for meaningful public performance claims; publish environment and methodology with numbers.

### 4. Registry publication as a separate milestone

Do not remove `publish = false` merely to make the repository look public. Public source delivery and crates.io distribution are separate decisions.

Before registry activation:

1. Confirm crate names, ownership and recovery access.
2. Complete package metadata and crate-local README/license packaging; inspect `cargo package --list`.
3. Add registry versions alongside internal path dependencies and derive publication order from the actual Cargo graph.
4. Run packaging and `cargo publish --dry-run` for each candidate. Local path success does not prove a clean registry consumer can resolve the graph.
5. Choose coordinated or independent version policy; the existing shared version makes coordinated releases the least surprising initial choice.
6. Validate a clean external consumer and docs.rs feature/target settings.
7. Introduce a deliberately scoped release workflow, protected release environment, changelog checks and post-release installation verification. Prefer short-lived identity-based publishing when supported by the selected registry; otherwise use scoped credentials and an explicit rotation policy.

Cargo recommends packaging/dry runs and explains that publication is permanent in its [publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html). Internal path-plus-version resolution is documented under [multiple dependency locations](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#multiple-locations). Credential storage considerations are described in [registry authentication](https://doc.rust-lang.org/cargo/reference/registry-authentication.html).

## Launch acceptance criteria

A new contributor can select a crate, build an example and run checks using only public documentation. CI verifies documented commands and relevant feature sets. Security reports reach a verified private channel. License/provenance and history review have named owners. README status matches actual delivery and support policy. Public release artifacts have durable URLs and provenance information. Registry claims remain absent until packages and documentation are actually available.

## Deliberate deferrals

Do not add CLA requirements, automated merge bots, elaborate release tooling, coverage thresholds or a documentation website solely for appearance. Start with maintainable Markdown, rustdoc, reliable checks and a clear release contract; add infrastructure when there is an owner and a demonstrated need.
