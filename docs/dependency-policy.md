# Dependency policy

Dependabot proposes weekly Cargo and GitHub Actions updates. SDK CI must verify
updates; maintainers review API, feature, license, wire and persistence effects.
Actions are pinned to full commit SHAs; Dependabot maintains those pins.

The dependency security workflow runs cargo-audit 0.23.0 against the current
RustSec database on dependency PRs, main changes and a weekly schedule. Known
vulnerabilities fail the job. Informational maintenance warnings remain visible
and require triage; there is no ignore list. A passing job is not a security audit.

## Bincode maintenance warning

As of 2026-11-24, `bincode` 1.3.3 has the informational unmaintained advisory
[RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141.html).
RustSec lists no patched release. This is a maintenance warning, not a finding
that all uses of bincode are exploitable.

Bincode is used in compatibility codecs and persisted checkpoints. Replacing it
requires a versioned wire/storage design, golden fixtures, bounded decode review
and consumer migration tests. Keep this warning visible; do not suppress it or
claim the dependency tree is warning-free. Repository maintainers should review
this decision before the first public release and again by 2027-02-24. Any new
vulnerability advisory must be assessed separately and remains blocking.

Before redistribution, inventory dependency license expressions with Cargo
metadata and inspect actual license/notice files for the shipped source and
artifacts. A registry SPDX expression alone is not a complete provenance review.

## Initial metadata inventory

The 2026-11-24 `cargo metadata --locked --format-version 1` inventory includes
112 resolved third-party packages, including target-specific packages. All have
license expressions; none are unspecified. Expressions include MIT, Apache-2.0,
BSD, ISC, Unicode-3.0 and LLVM-exception combinations. One package offers LGPL as
an alternative alongside MIT/Apache-2.0; select and honor an applicable license
rather than assuming every listed alternative applies simultaneously.
This metadata inventory is not a legal or source-provenance sign-off. Recompute
it for the actual release lockfile and preserve notices required by selected
licenses. Each SDK crate includes a copy of the workspace MIT LICENSE for packaging.
