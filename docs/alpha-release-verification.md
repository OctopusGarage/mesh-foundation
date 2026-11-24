# Alpha release verification

Candidate: seven mesh-foundation-* packages at 0.1.0-alpha.1.
Date: 2026-11-24. Status: prepared; no registry upload completed yet.

## Completed local checks

- Four release-policy tests cover alpha/beta/rc/stable tags, tag mismatches,
  mixed versions, registry restrictions and exact internal dependency versions.
- Workspace all-feature and no-default-feature tests pass with Rust 1.99.0.
- All-target/all-feature Clippy and rustdoc pass with warnings denied.
- actionlint and git diff whitespace checks pass.
- A separate external consumer uses generated root-level patch.crates-io
  overrides and runs device signature verification and inventory comparison.
- All seven packages pass Cargo's workspace publish dry-run, including packaging,
  unpacking and compilation against staged workspace registry packages. Uploads
  were aborted by Cargo's dry-run mode, as intended.
- Cargo.lock changes are limited to SDK package identities, versions and their
  internal references; third-party resolved versions did not change.

## Environment finding

The first dry-run in the user workspace failed because an ancestor Cargo config
replaces crates.io with rsproxy-sparse. Even a separate CARGO_HOME did not remove
that ancestor config. Validation succeeded after copying the candidate outside
the home-directory ancestry and using an isolated Cargo home with no mirror
configuration. The user's global config was not modified. Real upload should use
a clean checkout in that isolated location and the official crates.io registry.

## Actual publication gates

The user approved the public registry and new package names. A crates.io bootstrap
credential is still required through the secure secret UI. Before actual upload,
commit the verified candidate, identify the exact release SHA/tag, package the
clean source, recheck names and credentials, and run the registry dry-run there.
After upload, verify registry-only consumption and docs.rs. Do not present local
patch success as evidence that registry installation already works.

The OIDC publication workflow is prepared but not exercised remotely. Each crate
must first exist and its trusted publisher must match this repository,
.github/workflows/publish.yml and environment crates-io. Configure environment
reviewers separately. The workflow requires the successful three-platform SDK
checks on its exact tagged commit before attempting authentication or upload.

The bincode maintenance advisory remains visible as documented in dependency-policy.md.
