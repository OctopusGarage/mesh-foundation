# mesh-foundation-e2ee

Pairwise ratchets, group sender keys and checkpoint codecs.

Start with the `ratchet` and `sender_key` modules. Applications own authenticated key distribution and must atomically persist accepted events with receive checkpoints. In-memory candidate checks do not replace durable concurrency control.

## Development and documentation

From the workspace root:

```sh
cargo test --locked -p mesh-foundation-e2ee
cargo doc --locked -p mesh-foundation-e2ee --all-features --no-deps --open
```

This crate is part of [Mesh Foundation](https://github.com/OctopusGarage/mesh-foundation).
All seven crates share a version and Git revision. The initial registry candidate is `0.1.0-alpha.1`. Use exact alpha pins after
publication, or a reviewed full Git SHA during development. See the delivery
guide for registry aliases and local overrides.

See the repository [README](https://github.com/OctopusGarage/mesh-foundation#readme),
[contribution guide](https://github.com/OctopusGarage/mesh-foundation/blob/main/CONTRIBUTING.md)
and [security policy](https://github.com/OctopusGarage/mesh-foundation/blob/main/SECURITY.md).
Licensed under MIT; see the workspace LICENSE.
