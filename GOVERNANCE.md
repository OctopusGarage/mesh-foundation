# Governance and maintenance

Mesh Foundation is maintained in the OctopusGarage GitHub organization.
Repository maintainers with merge access make API, compatibility and release
decisions through issues and pull requests. This document does not create a
foundation, voting body or support commitment.

Small fixes can proceed directly to a PR. Changes to public APIs, wire formats,
persisted state or security assumptions should first record the problem,
alternatives, compatibility impact and migration plan in an issue or design
note. Keep the resulting decision discoverable from the PR and changelog.

All crates currently share a version and release revision. Development releases use coordinated alpha/beta/rc versions with exact alpha pins.
For stable releases before 1.0, incompatible
changes use a minor bump and compatible fixes use a patch. Wire and durable
compatibility still require explicit review regardless of the version number.
There is no scheduled release cadence or LTS policy. The pinned toolchain is the
verification baseline; changes to it are reviewed and documented.

Maintainers should resolve disagreements against reproducible evidence and
project scope, explain the decision and permit respectful follow-up. Report
conduct concerns privately to a repository maintainer through an established
private channel. Before public launch, administrators must identify active
reviewers, a release owner and a reachable private reporting contact using the
[launch checklist](docs/open-source-checklist.md).
