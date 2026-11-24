# SDK development and delivery

## Delivery model

Mesh Foundation ships all seven crates as one private Git revision. Every crate
inherits its version and registry publishing policy from the workspace.
Consumers pin the same full commit SHA for every Mesh Foundation dependency
and commit their Cargo.lock. A release tag identifies the source for humans.
The SHA is the dependency pin and the lockfile records the resolved source.

- Private Git uses existing GitHub read access and keeps inter-crate path
  dependencies usable. It is the default while the SDK and its API evolve.
- A private Cargo registry is an option when consumers need SemVer-based
  resolution and Cargo package distribution. It requires an authenticated
  index, package hosting, credential management and an owner for availability.
- crates.io distributes source publicly and published versions are permanent.
  It requires an explicit decision to publish the SDK, crate-name checks and
  crate ownership. The current manifests disable registry publication.

Sources: [Cargo dependency locations](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#multiple-locations),
[alternate registries](https://doc.rust-lang.org/cargo/reference/registries.html),
[crates.io publishing](https://doc.rust-lang.org/cargo/reference/publishing.html).

## Develop and consume

Install rustup and Python 3. The SDK checkout selects the Rust toolchain
through rust-toolchain.toml. From its root, run the same commands as CI:

```sh
python3 scripts/sdk.py check
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

For a private consumer such as Mesh Forum, use the canonical HTTPS Git URL and
the full SDK commit SHA in the manifest. Set `CARGO_NET_GIT_FETCH_WITH_CLI=true`
to use Git's configured credential helper. CI needs explicit read access to
the SDK repo. Keep credentials out of dependency URLs and lockfiles.

To use an uncommitted SDK checkout, generate a temporary Cargo config and pass
it to Cargo from the consumer workspace. The SDK path below assumes sibling
checkouts. The command emits absolute paths for all seven crates to avoid
mixed local and Git instances of the same SDK types.

```sh
mesh_patch=$(mktemp -d)/mesh-sdk-patch.toml
python3 ../mesh-foundation/scripts/sdk.py patch > "$mesh_patch"
cargo --config "$mesh_patch" test
```

Use a clean consumer worktree for local overrides. The first run can update its
lockfile. Inspect the resolved packages with
`cargo --config "$mesh_patch" tree`. Before committing, remove the override and
restore the original lockfile in that clean worktree. For a permanent upgrade,
update all Git pins together, regenerate the unpatched lockfile and run
`cargo test --locked`. See [Cargo patch resolution](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html).

Mesh Talk builds without access to this private repository by consuming a
committed source snapshot. When upgrading it, export the seven crate sources
and license from the same commit, preserve the SDK manifests' effective
metadata when expanding inheritance, and record the full SHA in its snapshot
README. Compare copied source bytes against the Git tree and validate the
Mesh Talk workspace plus the affected sync, relay and compatibility tests.

## Cut a private release

1. Review the API, wire and durable compatibility changes. Maintain
   `CHANGELOG.md` and bump the shared workspace version when warranted. Until
   1.0, use a minor bump for incompatible changes and a patch for compatible
   fixes. Regenerate Cargo.lock and verify the exact candidate.
2. Run the SDK CI matrix and the independent forum tests against the exact
   candidate SHA. For changes that affect Mesh Talk's compatibility, verify
   the corresponding Mesh Talk snapshot and runtime scenarios.
3. After the candidate lands on main, run `python3 scripts/sdk.py check --tag
   vX.Y.Z` with the actual version. Create and push an annotated tag at the
   reviewed commit. SDK CI reruns all verification, checks main ancestry, and
   uploads a source archive, commit SHA and SHA-256 checksums to that run.
4. Share the verified tag and SHA within the organization. Roll back a
   consumer by restoring its previous manifest pins and lockfile. Correct a
   faulty release with a new version and tag.

The CI artifact is retention-limited. The Git commit remains the canonical
source. Configure GitHub rulesets to require SDK verification on main and
restrict creation, movement and deletion of release tags. The workflow uses
read-only repository permissions. It requires no publishing credentials.

## Registry migration

Before enabling any registry, choose an operator, define read/publish roles,
and make every registry-facing manifest declare package metadata and versioned
internal dependencies. A combined `path` and `version` lets Cargo resolve
siblings locally and resolve registry copies once packaged. Test packaging
and a clean external consumer against the intended registry. Plan dependency
order from the Cargo graph. Identity, storage, blob and E2EE have no internal
predecessors. Connect and replication follow identity. Mailbox follows
identity, replication and storage. Confirm index visibility before proceeding
to dependents. Keep publish credentials in a scoped release environment and
give registry activation its own reviewed change.
