# Open-source preparation verification (historical baseline)

This records the initial pre-registry baseline. For the renamed alpha candidate, see
[alpha verification](alpha-release-verification.md).

Verified locally on macOS with the repository Rust 1.99.0 toolchain on 2026-11-24.
This records local evidence, not a successful GitHub-hosted three-platform run.

## Passed

- Workspace policy and matching `v0.1.0` tag validation.
- rustfmt and all-target/all-feature Clippy with warnings denied.
- Workspace default-feature tests: 213 passing tests, including doctests.
- Workspace all-feature tests, including the new identity and inventory doctests.
- Workspace no-default-feature tests: 185 passing tests, including doctests.
- Standalone no-default-feature tests for connect (14) and replication (21).
- Independent mesh-connect adapter tests for replication (23) and blob (21).
- Both executable examples: device signing/tampering rejection and bounded inventory comparison.
- Workspace all-feature rustdoc with warnings denied. Fixed two existing links to `Self::serialize`.
- actionlint, local Markdown file-link checks, changed-document spelling checks and `git diff --check`.
- `cargo package --list --allow-dirty` for every crate: README and LICENSE included.
- Gitleaks history scan: 64 commits; no detected leaks. A separate scan of tracked and untracked non-ignored source files also found no matches. These scans do not prove the absence of secrets.
- Third-party Cargo metadata license inventory: 112 packages, no unspecified license expressions. Actual provenance and redistribution notices still require review.

## Known maintenance warning

cargo-audit 0.23.0 completed successfully against 1,563 RustSec advisories and
119 lockfile dependencies, with the visible bincode unmaintained warning
RUSTSEC-2025-0141. No advisory ignore list was added. See the
[dependency policy](dependency-policy.md) for compatibility constraints and review timing.
The older locally installed cargo-audit 0.21.2 failed to parse a CVSS 4.0 advisory;
CI therefore installs 0.23.0 explicitly, with default binary-scanning features
disabled because this job audits the lockfile.

## Not exercised here

GitHub-hosted Linux/Windows jobs, Dependabot execution, the manual draft Release
job, environment approval enforcement, public unauthenticated downloads and
registry publication have not been exercised. Their YAML was statically validated.
The release workflow creates drafts only when explicitly requested on an existing
version tag; no Release was created during this preparation.

Registry package compilation/dry-run and a clean registry consumer remain blocked
on the separate registry activation work: names/owners and versioned internal
dependencies. A successful package file listing is not a successful publish dry-run.
The pinned toolchain is not an MSRV claim. External security review, parser fuzzing
and restart fault injection remain continuing engineering work.

Follow the [launch checklist](open-source-checklist.md) before changing visibility
or enabling registry publishing. No repository settings, tags or registry state
were changed, and no source changes were committed or pushed in this preparation.
