# Security model and integration boundaries

This is an integration guide, not a formal proof or independent security audit.
The host application remains responsible for its trust policy and durable state.

## Threats and boundaries

Treat peers, discovery traffic, stored envelopes and transport bytes as untrusted.
An attacker may send malformed frames, replay old data, exhaust budgets,
disconnect mid-frame or cause a process restart during a durable update.
Authentication establishes possession of a key; it does not authorize access.

| Component | SDK boundary | Application responsibility |
| --- | --- | --- |
| Identity | Signatures and device/account certificate verification | Trusted account roots, enrollment, revocation, recovery and secure key storage |
| Connectivity | Pinned-key sessions and frame/discovery primitives | Obtain trusted peer keys independently; admission, routing, deadlines and reconnect policy |
| Replication | Bounded inventories and reconciliation contracts | Scope authorization, stable snapshots, resource reservations and transactional stores |
| E2EE | Ratchet/sender-key transitions and checkpoint codecs | Authenticated key distribution, group membership and atomic state installation |
| Storage | Encrypted records and durable log primitives | Key protection, data model, access control and rollback-resistant state management |
| Mailbox | Opaque envelopes, quotas and leased spool writes | Admission, expiry scheduling, recipient policy and federation |
| Blob | Chunk integrity and bounded transfer primitives | Trusted manifests/keys, staging, scheduling, consent and retention |

## Failure and recovery rules

- Retire a session after a failed or cancelled in-flight frame; establish a new
  authenticated session before retrying. Do not resume from an uncertain byte offset.
- Reserve inventory memory and set frame, page, record and round budgets before
  consuming attacker-controlled data. Bound concurrency and apply host deadlines.
- Commit an accepted encrypted event and its updated receive checkpoint atomically.
  Compare the persisted checkpoint before installing a candidate; in-memory stale
  candidate rejection cannot protect against a concurrent durable writer or rollback.
- Stage received blob bytes until all required integrity checks pass. A matching
  inventory digest or custody response does not grant permission to read content.
- Keep secret material and plaintext out of logs, bug reports and fixtures. Handle
  exported secret bytes as sensitive copies; library zeroization does not erase
  copies made by the host, crash dumps, swap or backups.

## Non-goals

The SDK does not supply a complete account service, trust directory, keystore,
relay federation policy, anonymous transport, traffic-analysis protection or
application-level backup and recovery system. A compromised endpoint can expose
its accessible secrets and plaintext. Availability against arbitrary resource
exhaustion requires application and deployment controls.

## Review priorities

Before security-sensitive production use, arrange independent review of protocol
composition, cryptographic state transitions and durable failure recovery. Extend
existing tests with bounded parser fuzzing and restart fault injection. Publish
performance or security claims only with a reproducible method and evidence.
