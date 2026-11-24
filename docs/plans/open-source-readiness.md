# Open-source readiness implementation plan

Goal: make the existing seven-crate SDK understandable, contributable and verifiable before public launch.

Preserve private Git delivery and registry publication guards. Add public-facing documentation and independently runnable verification; avoid API or wire changes.

1. Add README navigation, truthful badges, quick start, executable examples and per-crate entry points.
2. Add contribution, security, governance, support and threat-model documents plus issue/PR templates.
3. Harden SDK CI with pinned actions, explicit toolchain installation, default/minimal/all-feature checks and rustdoc validation. Add dependency maintenance and advisory checks.
4. Prepare crate metadata and a release checklist, including registry packaging and external consumer validation gates. Preserve publish=false.
5. Verify formatting, policy, Clippy, tests, examples, rustdoc, workflow syntax, dependency advisories and local documentation links. Record findings and external launch gates.

GitHub visibility, branch rules, private reporting activation, ownership, credentials and actual publishing remain administrator launch actions. No commit, push or publication is part of this local preparation.

## Execution result

All five local preparation steps are implemented and verified. See
[verification evidence](../open-source-verification.md) and the
[administrator launch gates](../open-source-checklist.md). Registry activation,
external review and actual public release remain explicitly separate milestones.
