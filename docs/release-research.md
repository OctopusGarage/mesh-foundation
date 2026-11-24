# Release channel research

Investigated 2026-11-24 against primary Cargo and Rust project sources. This is a
recommendation, not evidence that any package has been published.

## Channels and recommendation

Use crates.io for public package distribution and docs.rs for generated public
API documentation. Keep GitHub Releases for human release notes, source archives
and checksums; a GitHub release does not install a crate into Cargo's registry.
Cargo uploads source `.crate` archives, validates their build and waits for index
visibility. Publishing is generally permanent: versions cannot be overwritten.
See [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
and [cargo publish](https://doc.rust-lang.org/cargo/commands/cargo-publish.html).

Recommendation: publish the seven crates as a coordinated `0.1.0-alpha.1` SDK.
This explicitly communicates ongoing API development while providing normal
registry installation. Use full-SHA Git dependencies between intentional alpha
releases; do not upload every commit just to imitate a mutable Maven snapshot.
Each published alpha is immutable. Increment to `alpha.2`, `alpha.3`, then beta,
release candidate and `0.1.0` as appropriate. Do not promise a schedule.

## Consumer version semantics

Cargo excludes prereleases unless the requirement opts into them. A requirement
`"0.1.0-alpha.1"` can select later prereleases on that release line and compatible
stable releases. Use `"=0.1.0-alpha.1"` when an exact reviewed alpha is required,
and keep the application Cargo.lock committed. Build metadata such as `+sha` is
ignored in dependency requirements and must not be used as snapshot identity.
These rules are defined in [Cargo dependency requirements](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#pre-releases).

Exact internal alpha requirements are a reasonable deliberate restriction while
cross-crate APIs evolve: they avoid mixed SDK alpha revisions. Upgrade the set
together. Revisit exact internal pins when the SDK's API compatibility stabilizes.

## Local development and integration

A direct `path` dependency is supported for unpublished local work. For a consumer
normally using crates.io, put `[patch.crates-io]` overrides at the consuming
workspace root. For a Git consumer, patch the original Git source URL instead.
Patch all SDK crates together so public types do not split across source copies.
The local version must satisfy the consumer's requirement; an exact alpha.1 pin
will not select a checkout already bumped to alpha.2. Inspect `cargo tree` and
`cargo metadata`, update the consumer requirement temporarily if needed, and
remove local overrides before validating the published integration. Cargo
patches are transitive but only honored from the top-level workspace. See
[Cargo overrides](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html).

## Packaging and order

Published internal dependencies need registry version requirements alongside
local paths. The path is used in the workspace; the version is used in the
published package. Preserve optional/default feature settings while adding
versions. See [multiple locations](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#multiple-locations).

Inspect package contents, run publish dry-runs, and build a clean registry-only
consumer after upload. A package file listing alone does not verify compilation.
Publish prerequisites first and wait for index visibility. The current graph
suggests identity, storage and E2EE first; then connect, replication, blob and
mailbox. Optional and development dependencies must also be reviewed. Current
Cargo documents `cargo publish --workspace`, but validate support and behavior
on the pinned toolchain; an explicit sequential script is easier to resume and
inspect for the first release. Do not use `--no-verify` to bypass registry errors.

## Authentication and automated releases

First-time registration requires crates.io account setup, a verified email and
publishing authentication. Check crate names and ownership immediately before
upload because names are first-come-first-served. A package source upload exposes
that source publicly even when its GitHub repository remains private.

The Rust project's [July 2025 announcement](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/)
states initial publication must be manual before configuring Trusted Publishing.
The [January 2026 update](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/)
adds an option to enforce Trusted Publishing after setup. The current
[crates.io setup page](https://crates.io/docs/trusted-publishing) requires JavaScript
and could not be fully extracted here; verify its live account prerequisites
before bootstrap rather than assuming pending-publisher support.

After bootstrap, prefer GitHub Actions OIDC with narrowly matched repository,
workflow and protected environment rather than a long-lived publishing token.
Keep CI token read permissions minimal, grant id-token only to the publisher,
and separate verification from upload. The first release still needs actual
account authority; GitHub authentication is not crates.io publishing permission.

## Release acceptance

A reviewed commit and immutable version tag identify each release. Release checks
must verify synchronized versions, internal requirements, changelog, clean tree,
package contents and supported features. A partial monorepo upload cannot be
rolled back atomically: resume only missing exact versions after confirming the
already-published versions came from the intended commit. Otherwise cut a new
alpha rather than overwrite or silently mix sources. Yank faulty versions if
appropriate, recognizing that yanking does not remove downloaded source or
invalidate existing lockfiles.

## Live repository and name checks

The 2026-11-24 crates.io API check found `mesh-identity` occupied by an unrelated
project (latest 0.2.1, repository https://github.com/afstanton/mesh-identity).
Sources: [crate metadata](https://crates.io/api/v1/crates/mesh-identity) and
[crate page](https://crates.io/crates/mesh-identity). The six other existing names
and all seven proposed `mesh-foundation-*` names returned HTTP 404 at check time;
this does not reserve names. The current repository remains private and its
publish=false guard blocks a real attempted dry-run before upload.

A concrete proposal, package naming migration and consumer examples are in the
[registry release design](plans/registry-release-design.md). No upload has occurred.

## First-publication rate limits confirmed during bootstrap

The first real upload succeeded for five new packages, then crates.io returned
HTTP 429 before replication. The official rate-limit documentation specifies a
burst of five new crates per account, followed by one new crate every ten minutes.
Existing-crate version uploads have a burst of thirty, followed by one per minute.
Plan bootstrap separately from routine version releases and honor server retry
timestamps. Sources: [rate limits](https://crates.io/docs/rate-limits) and the
[official page source](https://github.com/rust-lang/crates.io/blob/main/svelte/src/routes/docs/rate-limits/+page.svelte).
