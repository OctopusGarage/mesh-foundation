# Security policy

Mesh Foundation is an evolving SDK containing cryptographic and network code.
It does not claim an independent security audit or production certification.
Review the [security model](docs/security-model.md) before integrating it.

## Report a vulnerability

Use GitHub's **Security → Advisories → Report a vulnerability** for this
repository when that option is available. Do not disclose exploit details,
private keys, decrypted messages or sensitive logs in public issues or PRs.
If private reporting is unavailable, ask a repository maintainer to establish a
private channel without posting vulnerability details. Private reporting must be
enabled and tested by an administrator before public launch.

Include the affected commit/version, enabled features, platform, impact,
reproduction steps and a minimal sanitized proof of concept. Describe any
application trust assumptions needed to trigger the issue.

## Support and disclosure

The project currently uses coordinated 0.x development and Git revision pins.
Security fixes target main; there is no maintained historical release branch or
promised response SLA. Consumers should track reviewed fixes and update all SDK
pins together. Maintainers should acknowledge reports, agree on a disclosure
plan with the reporter, prepare a fix and regression coverage, and publish an
advisory with affected revisions, mitigations and a fixed revision when warranted.
A reproducible test suite does not replace independent cryptographic review.

Dependency advisories and known maintenance concerns are tracked in the
[dependency policy](docs/dependency-policy.md).
