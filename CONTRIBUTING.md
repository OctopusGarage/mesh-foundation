# Contributing

Thanks for helping improve Mesh Foundation. Start with the [README](README.md),
[composition guide](docs/sdk-composition.md) and [security boundaries](docs/security-model.md).
For security issues, follow [SECURITY.md](SECURITY.md) rather than opening a public issue.

## Development

Install Rust with rustup and Python 3. The repository selects Rust 1.99.0 with
rustfmt and Clippy. This is the tested toolchain, not a minimum supported Rust
version promise. If Cargo comes from another installation, check `rustc --version`.

Run from the repository root:

```sh
python3 scripts/sdk.py check
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace
cargo test --locked --workspace --all-features
cargo test --locked --workspace --no-default-features
cargo test --locked -p mesh-foundation-connect --no-default-features
cargo test --locked -p mesh-foundation-replication --no-default-features
cargo test --locked -p mesh-foundation-replication --no-default-features --features mesh-connect
cargo test --locked -p mesh-foundation-blob --no-default-features --features mesh-connect
cargo run --locked -p mesh-foundation-identity --example device_identity
cargo run --locked -p mesh-foundation-replication --no-default-features --example inventory
```

Build API documentation with warnings denied (POSIX shell):

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --all-features --no-deps
```

On PowerShell, set `$env:RUSTDOCFLAGS = "-D warnings"` before `cargo doc`.
See [SDK delivery](docs/sdk-delivery.md) for temporary local consumer overrides.

## Propose a change

Discuss broad API or protocol changes in an issue before implementing them.
Include a small reproducer, expected behavior, crate/features, OS and Rust version
for bugs. Remove keys, message content, credentials and personal information.

Keep pull requests focused. Explain the user-visible effect, compatibility
implications and checks run. Add regression coverage for behavior changes;
exercise failure, cancellation and restart paths where applicable. Preserve wire
formats, event IDs and checkpoint compatibility unless the change explicitly
introduces a reviewed migration. Update rustdoc and the Unreleased changelog.
Do not silently suppress security advisories or relax input bounds to pass tests.

Maintainers review correctness, API shape, security boundaries and compatibility.
There is no guaranteed review time. Contributions are provided under the
repository's MIT license; retain attribution and disclose third-party source.
Be respectful, assume good intent and keep discussion focused on the work.
