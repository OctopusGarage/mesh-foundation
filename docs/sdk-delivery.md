# SDK development and delivery

## Release channels

Mesh Foundation uses coordinated versions across seven `mesh-foundation-*`
packages. All seven packages are published at `0.1.0-alpha.2`; see the
[GitHub prerelease](https://github.com/OctopusGarage/mesh-foundation/releases/tag/v0.1.0-alpha.2). Public registry uploads expose package source
regardless of the GitHub repository's visibility.

- **crates.io** distributes immutable source packages. docs.rs builds API docs.
- **GitHub Releases** provide notes, source archives and checksums. They do not
  register Cargo packages. Mark alpha/beta/rc releases as prereleases.
- **Development snapshots** use full Git SHAs, not mutable snapshot versions.
- **Local development** uses paths or root-level Cargo patches.

## Registry consumers

After the selected version has been published:

```toml
[dependencies]
mesh-identity = { package = "mesh-foundation-identity", version = "=0.1.0-alpha.2" }
mesh-replication = { package = "mesh-foundation-replication", version = "=0.1.0-alpha.2", default-features = false }
```

Rust imports remain `mesh_identity` and `mesh_replication`. Old package names are
not registry aliases: `mesh-identity` belongs to another project. Package keys
are local aliases, while `package` selects the actual published identity.

Exact prerelease pins deliberately prevent unreviewed alpha upgrades. Upgrade
all used SDK crates together and commit the application's Cargo.lock. Check the
changelog, regenerate the lockfile and run `cargo test --locked` in the consumer.

## Develop and consume

Install rustup and Python 3; `rust-toolchain.toml` selects Rust 1.99.0, rustfmt and
Clippy. This is the tested compiler, not a minimum supported compiler promise.
See [CONTRIBUTING](../CONTRIBUTING.md) for the full verification commands.

For Git snapshots, use the same full commit SHA for all SDK dependencies:

```toml
[dependencies]
mesh-identity = { package = "mesh-foundation-identity", git = "https://github.com/OctopusGarage/mesh-foundation.git", rev = "FULL_REVIEWED_COMMIT_SHA" }
```

Replace the illustrative SHA with a real full revision. A private repository
requires GitHub read access. Use Git's credential helper and, where necessary,
`CARGO_NET_GIT_FETCH_WITH_CLI=true`; never put tokens in URLs or Cargo.lock.
Existing Git consumers upgrading across the package rename must add `package`
aliases. Directory paths and Rust import names remain unchanged.

For registry-based consumers, generate temporary local overrides for all seven
packages from a sibling SDK checkout:

```sh
mesh_patch=$(mktemp -d)/mesh-sdk-patch.toml
python3 ../mesh-foundation/scripts/sdk.py patch --source crates-io > "$mesh_patch"
cargo --config "$mesh_patch" test
cargo --config "$mesh_patch" tree
```

For a Git consumer, omit `--source crates-io` (the default patches the canonical
Git URL). Cargo honors patches only at the consuming workspace root. Local
versions must satisfy the consumer's requirements: an exact alpha.1 dependency
will not select an alpha.2 checkout. Use a matching checkout or temporarily
change the consumer requirement. Keep absolute paths out of commits. Overrides
can change Cargo.lock; test in a clean consumer worktree and restore its original
lockfile before validating a real unpatched upgrade.

A direct path is also supported:

```toml
mesh-identity = { package = "mesh-foundation-identity", path = "../mesh-foundation/crates/mesh-identity" }
```

## Version policy

Use `0.1.0-alpha.2`, `alpha.2`, etc. for intentional integration releases. Alpha
APIs may break; describe changes and upgrade the coordinated set together. Move
to beta/rc when interface and compatibility criteria warrant it, then `0.1.0`.
After stable 0.1.0, incompatible changes bump minor and compatible fixes bump
patch. Wire/storage compatibility needs review independently of the version.

Cargo prerelease requirements without `=` can advance to later prereleases and
compatible stable releases. Build metadata (`+sha`) is ignored for requirements;
it is not a snapshot selection mechanism. Published versions cannot be replaced.

## Prepare and publish

1. Update the shared workspace version and every exact internal version together;
   regenerate Cargo.lock without unrelated dependency updates. Update changelog
   and consumer migration notes. Verify the crate names are still available and
   the account has authority to publish them.
2. Run `python3 scripts/sdk.py check --tag v0.1.0-alpha.2` (substitute the candidate),
   the SDK tests and dependency security checks. Inspect package contents and run:

   ```sh
   cargo publish --workspace --dry-run --locked --registry crates-io
   ```

   For local preparation only, `--allow-dirty` permits inspecting uncommitted
   candidates. A real release must use a clean, committed and reviewed source.
   Do not bypass package compilation with `--no-verify`.
3. Record the exact release commit, create its matching annotated version tag,
   and run SDK CI on that commit. Release from reviewed main history. The SDK
   policy script checks prerelease and stable tags and internal requirements.
4. Bootstrap with authenticated crates.io credentials and verified email.
   GitHub login is not registry authentication. Publish prerequisites first:
   identity, storage, E2EE, connect, replication, blob, mailbox, using their full
   `mesh-foundation-*` package names. Cargo waits for registry index visibility.
   The first command is:

   ```sh
   cargo publish --locked --registry crates-io -p mesh-foundation-identity
   ```

   Continue only after verifying the preceding upload. Never use a dirty tree
   or `--no-verify` for actual upload. For subsequent coordinated uploads, the
   pinned Cargo also supports `cargo publish --workspace`; verify its dry-run
   before selecting it for a release.
5. Build and run a clean external project using only registry dependencies, with
   no Git/path patches. Confirm docs.rs builds, owners and recovery access.
6. Create a GitHub prerelease with curated notes, full SHA, archive and checksums.
   The SDK workflow can create a draft when dispatched on the existing version
   tag with `create_draft_release=true`. Configure required reviewers on the
   `release` environment; a YAML environment name alone does not enforce approval.

First publication of a seven-crate workspace can take more than twenty minutes:
crates.io allows five new crates in a burst, then one additional new crate every
ten minutes. New versions of existing crates use a different limit (a burst of
30, then one per minute). Follow the exact retry timestamp returned by HTTP 429;
do not repeatedly restart the whole workspace upload. See the official
[publishing rate limits](https://crates.io/docs/rate-limits).

There is no atomic multi-crate registry transaction. On partial failure, inspect
published versions and provenance before resuming only missing packages. Never
skip an existing version based only on its name. If provenance is uncertain,
stop and cut a new coordinated alpha. Roll back consumers to previous pins and
lockfiles; yanking does not delete source or invalidate existing lockfiles.

## Repeat release automation

After initial manual bootstrap, configure crates.io Trusted Publishing for the
exact GitHub repository, publishing workflow and protected environment. Prefer
short-lived OIDC credentials over a permanent token. Grant `id-token: write`
only to the upload job. Validate the exact tag and main ancestry, require clean
source and successful verification, then publish and verify consumers before
publishing release notes. The `Publish crates.io` workflow is manually dispatched from main with an existing
version tag. It requires successful SDK checks for all three operating systems
on that exact commit, performs a registry dry-run, then obtains an OIDC token and
publishes. Register `.github/workflows/publish.yml` and environment `crates-io` in
each crate's Trusted Publishing settings and configure environment reviewers.
Until that account-side setup exists, publishing is manual; the draft source-release
workflow does not upload Cargo packages.

## Sources and launch gates

[Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html),
[prerelease requirements](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#pre-releases),
[local overrides](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html),
[launch checklist](open-source-checklist.md).


## Managed source snapshots

Git or registry dependencies are usually simplest. A consumer that vendors a
reviewed SDK can use the existing delivery tool instead of copying files by hand:

```sh
python3 scripts/sdk.py snapshot export ../consumer/vendor/mesh-foundation
python3 scripts/sdk.py snapshot verify ../consumer/vendor/mesh-foundation --source .
```

Export requires a clean, committed SDK checkout and records its full SHA in
`SOURCE`. A bundled `.gitattributes` preserves LF bytes even in consumers using
`core.autocrlf`; verification compares raw source bytes. `SNAPSHOT.json` covers the managed Cargo manifest, README, licenses,
crate sources/tests/examples and the standalone verifier. Updates refuse changed,
extra, missing or symlinked destination files. An older SOURCE-only snapshot is
adopted only after exact comparison with its recorded Git commit. Commit the
consumer snapshot and lockfile together and run its affected acceptance checks.

Consumer CI can verify bytes offline without credentials or a sibling checkout:

```sh
python3 vendor/mesh-foundation/scripts/sdk_snapshot.py verify vendor/mesh-foundation
```

Offline digests detect drift, not a maliciously rewritten manifest. Use `--source`
with the reviewed SDK Git checkout to verify provenance against the recorded
commit. Export builds and verifies a staged directory before replacing an existing
verified snapshot; keep ordinary Git recovery for interrupted updates. Do not
modify the managed copy; change the SDK, commit, export, and verify the consumer.

Snapshot updates acquire a nonblocking OS lease for the canonical destination.
The stable one-byte `.mesh-NAME.snapshot-lock` inode lives beside the destination,
so processes with different temporary directories still share one lease. Ignore
`.mesh-*.snapshot-lock` in the consumer repository; the inode is runtime metadata,
not part of the managed source, and process exit releases ownership. Concurrent
updates report busy rather than racing verification and replacement.

The previous verified tree is kept at the sibling `.NAME.snapshot-backup` path,
outside automatic staging cleanup. If installation and restoration both fail,
the error names that retained backup. A retry from a clean SDK checkout verifies
and restores an interrupted backup, or removes it only after verifying the
installed destination too. Unknown/modified backups are preserved and rejected.
This is source-update recovery, not a database transaction or filesystem durability
promise across power loss; the reviewed Git commit remains authoritative.

Recovery verification never treats an empty target as an installed snapshot.
If an external operation recreated the target as an empty directory while a
verified backup remains, export rejects the ambiguous state and preserves both.
Verify the backup and remove only the empty placeholder with `rmdir` before
retrying. An empty directory remains supported for a fresh installation when
there is no recovery backup.

## Published source identity

All seven `0.1.0-alpha.2` registry archives were downloaded and their registry
checksums and `.cargo_vcs_info.json` verified against immutable tag
`v0.1.0-alpha.2`, commit `0aca20d10262aa7c0126d958bc7627d05f8a2424`.
The three-platform SDK matrix and seven-package dry-run passed before GitHub
Actions published with OIDC. Each crate trusts repository
`OctopusGarage/mesh-foundation`, workflow `publish.yml`, environment `crates-io`.

Alpha.1 was bootstrapped using a temporary local API token, deleted after upload.
Its immutable tag remains at `2812256454ee7dbbc7a2e9bf73bfeb51ca570c61`.
