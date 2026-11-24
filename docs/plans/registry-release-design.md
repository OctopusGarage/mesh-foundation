# Registry release design: first development alpha

Status: approved for implementation and first public alpha publication.
Investigated 2026-11-24. No package has been uploaded.

## Observed blockers

The repository is private, the workspace uses version 0.1.0 and publish=false,
and the SDK policy script accepts only stable tags and requires publishing to be
disabled. Internal dependencies have paths without registry versions. A real
`cargo publish --dry-run --locked -p mesh-identity --allow-dirty --registry crates-io`
stops at the publication guard, before packaging or upload.

The crates.io API reports mesh-identity already exists (0.2.1), belonging to an
unrelated repository, https://github.com/afstanton/mesh-identity. Other current
names returned HTTP 404. All seven proposed mesh-foundation-* names returned
HTTP 404. This is a point-in-time availability check, not a reservation or
ownership guarantee. Recheck immediately before publishing.

No CARGO_REGISTRY_TOKEN or Cargo credentials file is present in this environment.
GitHub authentication does not establish crates.io account authority. Other OS
credential providers have not been tested for a usable crates.io credential.

## Chosen recommendation and alternatives

Recommend crates.io for installable public source packages, docs.rs for API docs,
and GitHub Releases for curated notes and source archive/checksums. Development
consumers use immutable Git SHAs between intentional registry alpha releases.

A GitHub-only prerelease preserves private access but is not a Cargo registry
package; consumers must retain Git dependencies or source snapshots. A private
Cargo registry supports confidential versioned distribution, but requires a
registry operator, availability owner, credentials and consumer configuration.
Choose it only if the SDK source must remain confidential.

## Public package names and import compatibility

Use a consistent project prefix rather than mixing a replacement identity name
with six generic names:

| Current package/directory | Proposed registry package | Rust library name |
| --- | --- | --- |
| mesh-identity | mesh-foundation-identity | mesh_identity |
| mesh-storage | mesh-foundation-storage | mesh_storage |
| mesh-e2ee | mesh-foundation-e2ee | mesh_e2ee |
| mesh-connect | mesh-foundation-connect | mesh_connect |
| mesh-replication | mesh-foundation-replication | mesh_replication |
| mesh-blob | mesh-foundation-blob | mesh_blob |
| mesh-mailbox | mesh-foundation-mailbox | mesh_mailbox |

Keep directories and explicit [lib] names unchanged. Internal dependency keys
retain their current spelling, with package set to the new registry name.
Existing Git consumers must add package aliases after updating to the renamed
revision. Review consumer manifests and local patch output together; preserving
Rust import names alone does not preserve Cargo package identity.

Example consumer after publication:

```toml
[dependencies]
mesh-identity = { package = "mesh-foundation-identity", version = "=0.1.0-alpha.1" }
mesh-replication = { package = "mesh-foundation-replication", version = "=0.1.0-alpha.1", default-features = false }
```

Code continues to import `mesh_identity` and `mesh_replication`. Install only the
crates required by the application. Registry access does not require access to
the private GitHub repository. Publishing nevertheless makes the package's source
public and permanent; no repository visibility change is needed for that exposure.

## Version policy

All seven crates share one version, one release commit and one vVERSION tag.
Start at 0.1.0-alpha.1; intentionally tested integration releases increment to
alpha.2, alpha.3, then beta/rc when their entry criteria are met. Alpha versions
may have breaking APIs; document changes and upgrade all consumed SDK crates
together. Use exact internal and consumer alpha requirements to avoid mixing
unreviewed SDK revisions. Do not treat a prerelease requirement without '=' as
an exact pin: Cargo may advance it to later prereleases or compatible stable
versions. Commit the application lockfile.

A publish is immutable. There is no mutable Maven-style SNAPSHOT version.
Use a full Git SHA as a development snapshot; do not use +sha metadata for Cargo
version selection. If an archived snapshot is needed, use a unique GitHub
prerelease name and retain the actual commit SHA in the consumer contract.

When ready for 0.1.0, verify supported features/platforms, document compatibility,
resolve launch gates and publish a stable version with migration notes. Before
1.0, incompatible stable releases increment the minor version; compatible fixes
increment patch. Stable internal requirements can be revisited at that milestone.
The pinned development compiler is not an untested MSRV promise.

## Local development

Within this workspace, path-plus-version dependencies use sibling sources. An
external consumer can use a direct path dependency and an explicit package alias.
For a consumer normally using crates.io, place overrides at its workspace root:

```toml
[patch.crates-io]
mesh-foundation-identity = { path = "../mesh-foundation/crates/mesh-identity" }
mesh-foundation-replication = { path = "../mesh-foundation/crates/mesh-replication" }
```

Generate overrides for all seven packages to avoid duplicate source instances
when composing crates. Git consumers instead patch the original Git URL. Extend
`scripts/sdk.py patch` with an explicit source choice while preserving its Git
source default. Local versions must satisfy the consumer requirement: an exact
alpha.1 dependency will not select an alpha.2 checkout. Temporarily update the
consumer's requirement or check out the matching version; confirm the resolved
source with cargo tree/metadata. Do not commit absolute local paths. Remove the
patch and regenerate/verify the consumer lockfile before recording an SDK upgrade.

## Implementation and validation

1. Rename package identities, preserve library names and dependency aliases, add
   exact internal versions, select 0.1.0-alpha.1 and publish=["crates-io"]. Update
   Cargo.lock without opportunistic third-party upgrades.
2. Extend and test SDK checks for stable/alpha/beta/rc tags, synchronized versions,
   intended registry and matching internal dependency names/versions. Reject
   leading-zero prerelease numbers and mismatched tags. Update patch generation.
3. Add explicit prepare/dry-run/publish commands. Default preparation must never
   upload. Real publication requires a clean reviewed commit and matching tag.
   Do not use --allow-dirty or --no-verify for the real release.
4. Verify workspace feature combinations, examples, rustdoc and advisory policy.
   Inspect package contents; package all seven crates with the pinned Cargo.
   Test workspace dry-run support before assuming unpublished siblings resolve.
5. Bootstrap dependency-first: identity, storage and E2EE; connect; replication;
   blob and mailbox. Validate the actual Cargo graph including optional and dev
   dependencies. Stop on failures and inspect registry state before resuming.
6. Verify each upload is indexed, then compile a clean registry-only consumer,
   run identity/inventory examples and inspect docs.rs build results. Create a
   matching GitHub prerelease with changelog and checksums after all packages are
   confirmed. Mark GitHub alpha releases as prerelease.

## Authentication and repeat releases

First upload needs an authenticated crates.io account with verified email and
permission to create the chosen packages. Obtain credentials through the secure
secret UI, never in chat, source, command arguments or logs. Use scoped and
short-lived bootstrap authority where the account supports it; revoke it after
setup. Confirm package owners/recovery before expanding automation.

After bootstrap, configure crates.io Trusted Publishing for the exact GitHub
repository, workflow and protected release environment. Restrict id-token:write
to the publishing job and re-run release gates on the exact tag. Keep the
verification workflow read-only. A repository environment name alone does not
create an approval requirement; configure required reviewers server-side.

The registry has no atomic seven-package transaction. If some uploads succeed,
verify their provenance before resuming only the missing exact versions. Never
skip an existing version based only on its name. If source/version identity is
uncertain, stop and cut a new coordinated alpha. Yanking is a mitigation, not
source deletion or a rollback of existing lockfiles.

## Authorization

The user confirmed public crates.io distribution and the mesh-foundation-* names. The first credential card
comes after candidate preparation and verification. No secret is needed for
research, package inspection or dry-run.

Primary references and detailed semantics: [release research](../release-research.md).
