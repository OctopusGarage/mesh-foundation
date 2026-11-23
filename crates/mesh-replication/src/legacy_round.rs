//! Requester engine for the existing Mesh Talk wire protocol.
//! Applications supply a bounded, freshly authorized event store and own the
//! cancellation lifetime of one authenticated session.

use crate::legacy_event::{ConversationId, Event, EventId, EventKind};
use crate::legacy_wire::{
    self, FpRequest, FpResponse, HaveChunk, SyncFollowup, SyncRequest, SyncWire,
};
use crate::session::ReconciliationSession;
use std::collections::HashSet;
use std::ops::Deref;

pub const MAX_SYNC_ROUNDS: usize = 10_000;
pub const MAX_REJECTED_DETAIL: usize = 64;
pub const FILE_FOLLOWUP_BURST: usize = 4;

#[derive(Debug, PartialEq, Eq)]
pub enum RoundFault {
    Wire(String),
    UnexpectedMessage,
    NoProgress,
    CounterExhausted,
}

fn encode_frame(wire: &SyncWire) -> Result<Vec<u8>, RoundFault> {
    legacy_wire::encode(wire).map_err(RoundFault::Wire)
}

fn decode_frame(bytes: &[u8]) -> Result<SyncWire, RoundFault> {
    legacy_wire::decode(bytes).map_err(RoundFault::Wire)
}

/// Scalar metadata from a finite owned view or an exactly qualified probe cache.
/// This grants no Have/export authority. A missing fingerprint skips the optional
/// probe; it never denotes an empty set.
#[derive(Clone, Copy, Debug, Default)]
pub struct SyncMetadata {
    pub count: u64,
    pub channel_v2: bool,
    pub channel_v3: bool,
    pub channel_v4: bool,
    pub channel_v5: bool,
    pub fingerprint: Option<[u8; 32]>,
}

/// Local scheduling evidence for a completed quiet remote subset, never equality,
/// custody or delivery. The issuing backend must revalidate the opaque local token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuietComparison {
    pub local_token: [u8; 32],
    pub remote_fingerprint: [u8; 32],
    pub remote_count: u64,
}

/// Outcome of ingesting a batch of received events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApplyReport {
    /// Events newly appended.
    pub applied: usize,
    /// Events already present (content-addressed duplicates).
    pub duplicates: usize,
    /// Events rejected by validation, with a human-readable reason. (id, message)
    /// Typed validation/admission errors are classified before formatting these
    /// bounded diagnostics. Local store failures abort sync instead.
    pub rejected: Vec<(EventId, String)>,
}

/// Storage and authority operations for a bounded legacy round. Calls that
/// await may yield between finite preparation steps. No store lock should
/// survive a yield or an await on the transport.
#[allow(async_fn_in_trait)]
pub trait RoundBackend<E> {
    type Peer;
    type View;
    type RemoteHave: Deref<Target = HashSet<EventId>>;

    fn bind_session_peer(&self, peer: &Self::Peer);
    async fn prepared_probe(&self, conversation: &ConversationId) -> Result<Probe<Self::View>, E>;
    async fn open_prepared_view(&self, conversation: &ConversationId) -> Result<Self::View, E>;
    fn advertised_count(&self, view: &Self::View) -> Result<usize, E>;
    fn have_count(&self, count: u64) -> Result<usize, E>;
    fn validate_view(&self, view: &Self::View) -> Result<(), E>;
    fn reusable_view(&self, view: &Self::View) -> Result<bool, E>;
    fn retain_custody_view(&self, view: Self::View) -> Result<Option<Self::View>, E>;
    fn quiet_comparison(&self, conversation: &ConversationId)
    -> Result<Option<QuietComparison>, E>;
    fn validate_quiet_comparison(
        &self,
        conversation: &ConversationId,
        hint: QuietComparison,
    ) -> Result<bool, E>;
    fn quiet_comparison_token(&self, view: &Self::View) -> Result<Option<[u8; 32]>, E>;
    fn retain_quiet_comparison(&self, view: &Self::View, hint: QuietComparison) -> Result<(), E>;
    async fn send_have_pages(
        &self,
        session: &mut impl ReconciliationSession<Peer = Self::Peer, Error = E>,
        view: &Self::View,
        conversation: ConversationId,
        wrap: fn(HaveChunk) -> SyncWire,
    ) -> Result<(), E>;
    async fn receive_have(
        &self,
        session: &mut impl ReconciliationSession<Peer = Self::Peer, Error = E>,
        conversation: ConversationId,
        compare: bool,
    ) -> Result<Self::RemoteHave, E>;
    async fn intersection_count(
        &self,
        view: &Self::View,
        remote: &Self::RemoteHave,
    ) -> Result<usize, E>;
    fn comparison_fingerprint(&self, remote: &Self::RemoteHave) -> Option<[u8; 32]>;
    fn apply(&self, events: &[Event]) -> Result<ApplyReport, E>;
    fn admission_denied(&self) -> bool;
    async fn select_export_frames(
        &self,
        view: &Self::View,
        scope: ConversationId,
        remote: &Self::RemoteHave,
        max_frames: usize,
    ) -> Result<Vec<SyncFollowup>, E>;
}

pub type Probe<View> = (SyncMetadata, Option<View>);

pub fn supports_event_kind(kind: EventKind, v2: bool, v3: bool, v4: bool, v5: bool) -> bool {
    match kind {
        EventKind::ChannelMetadata => v5,
        EventKind::ChannelAccountAuthority | EventKind::ChannelDeviceInheritance => v4,
        EventKind::ChannelHistoryPolicy | EventKind::ChannelJoinHistory => v3,
        EventKind::ChannelReplica => v2,
        _ => true,
    }
}

/// Resume only between complete protocol exchanges, never midway through
/// a Noise frame. A caller timing out a step must discard its channel and state.
#[derive(Default)]
pub struct SyncProgress {
    pub started: bool,
    pub v2: bool,
    pub v3: bool,
    pub v4: bool,
    pub v5: bool,
    pub rounds: usize,
    pub previous_push: std::collections::HashSet<EventId>,
    pub total: ApplyReport,
    pub first_have: usize,
    pub pushed_total: usize,
    pub received_wire_total: usize,
    pub comparison_disabled: bool,
    /// Intersection of local advertised IDs and peer availability, never custody.
    pub remote_have_count: Option<usize>,
}

impl SyncProgress {
    pub async fn step<B, S, E>(
        &mut self,
        channel: &mut S,
        backend: &B,
        conversation: ConversationId,
        max_followups: usize,
    ) -> Result<bool, E>
    where
        B: RoundBackend<E>,
        S: ReconciliationSession<Peer = B::Peer, Error = E>,
        E: From<RoundFault>,
    {
        if max_followups == 0 || max_followups > FILE_FOLLOWUP_BURST {
            return Err(RoundFault::UnexpectedMessage.into());
        }
        backend.bind_session_peer(channel.peer_identity());
        // Phase 0 — fingerprint short-circuit. Exchange a single 32-byte digest of our whole
        // id-set; if the responder's matches, the logs are identical and we skip the O(N)
        // have-set streaming entirely (the common idle-drain case). A mismatch falls through to
        // the full reconciliation below. (Correctness: a digest match means equal sets; a
        // false match is cryptographically negligible and, since ingest re-validates, could
        // only skip a diff — never corrupt — so this is safe.)
        if !self.started {
            self.started = true;
            let (metadata, probe_view) = backend.prepared_probe(&conversation).await?;
            let local_count =
                usize::try_from(metadata.count).map_err(|_| RoundFault::CounterExhausted)?;
            let (local_v2, local_v3, local_v4, local_v5) = (
                metadata.channel_v2,
                metadata.channel_v3,
                metadata.channel_v4,
                metadata.channel_v5,
            );
            self.v2 |= local_v2 || local_v3 || local_v4 || local_v5;
            self.v3 |= local_v3 || local_v4 || local_v5;
            self.v4 |= local_v4 || local_v5;
            self.v5 |= local_v5;
            let comparison = if self.comparison_disabled {
                None
            } else {
                backend.quiet_comparison(&conversation)?
            };
            if let Some(fp) = comparison
                .map(|hint| hint.remote_fingerprint)
                .or(metadata.fingerprint)
            {
                if let Some(view) = &probe_view {
                    backend.validate_view(view)?;
                }
                let request = FpRequest {
                    conversation,
                    fingerprint: fp,
                };
                channel
                    .send(&encode_frame(&if self.v5 {
                        SyncWire::FpRequestV5(request)
                    } else if self.v4 {
                        SyncWire::FpRequestV4(request)
                    } else if self.v3 {
                        SyncWire::FpRequestV3(request)
                    } else if self.v2 {
                        SyncWire::FpRequestV2(request)
                    } else {
                        SyncWire::FpRequest(request)
                    })?)
                    .await?;
                match decode_frame(&channel.recv().await?)? {
                    SyncWire::UpgradeRequired { conversation: c }
                        if c == conversation && !self.v5 =>
                    {
                        self.comparison_disabled = true;
                        self.v5 = self.v4;
                        self.v4 = self.v3;
                        self.v3 = self.v2;
                        self.v2 = true;
                        self.started = false;
                        return Ok(false);
                    }
                    SyncWire::FpResponse(FpResponse { matched: true }) => {
                        if let Some(hint) = comparison {
                            if !backend.validate_quiet_comparison(&conversation, hint)? {
                                self.comparison_disabled = true;
                                self.started = false;
                                return Ok(false);
                            }
                            if let Some(view) = probe_view {
                                backend.validate_view(&view)?;
                                drop(backend.retain_custody_view(view)?);
                            }
                            self.remote_have_count = Some(backend.have_count(hint.remote_count)?);
                            return Ok(true);
                        }
                        if let Some(view) = probe_view {
                            backend.validate_view(&view)?;
                            drop(backend.retain_custody_view(view)?);
                        }
                        self.remote_have_count = Some(local_count);
                        log::debug!(
                            target: "mesh_talk::sync",
                            "sync conv={} fp_match skipped",
                            hex::encode(&conversation.as_bytes()[..4]),
                        );
                        return Ok(true);
                    }
                    SyncWire::FpResponse(FpResponse { matched: false }) => {
                        self.comparison_disabled |= comparison.is_some();
                        // A differing remote digest does not invalidate our
                        // local authorization selection. Transfer its one
                        // existing lease to the following Have/export phase.
                        if let Some(view) = probe_view {
                            backend.validate_view(&view)?;
                            drop(backend.retain_custody_view(view)?);
                        }
                        // A complete exchange is a scheduling boundary for
                        // bounded callers, even when reconciliation must follow.
                        return Ok(false);
                    }
                    _ => return Err(RoundFault::UnexpectedMessage.into()),
                }
            }
        }

        // Sync-cost telemetry (see node/session.rs sync docs): we re-stream the FULL id-set
        // (`have`) every round in both directions regardless of how small the diff is, so the
        // metadata cost is O(N) per round. Capture N, the diff, and the round count so we can
        // evaluate reconciliation scaling (e.g. range-based / Negentropy) on real workloads.
        let round = self.rounds;
        let have_view = backend.open_prepared_view(&conversation).await?;
        let local_count = backend.advertised_count(&have_view)?;
        if round == 0 {
            self.first_have = local_count;
        }
        self.rounds += 1;
        let comparison_token = if !self.comparison_disabled
            && self.received_wire_total == 0
            && self.pushed_total == 0
            && self.total.applied == 0
            && self.total.rejected.is_empty()
        {
            backend.quiet_comparison_token(&have_view)?
        } else {
            None
        };
        // Opening Request carries an empty inline have; the real set is streamed.
        let request = SyncRequest {
            conversation,
            have: Vec::new(),
        };
        channel
            .send(&encode_frame(&if self.v5 {
                SyncWire::RequestV5(request)
            } else if self.v4 {
                SyncWire::RequestV4(request)
            } else if self.v3 {
                SyncWire::RequestV3(request)
            } else if self.v2 {
                SyncWire::RequestV2(request)
            } else {
                SyncWire::Request(request)
            })?)
            .await?;
        backend
            .send_have_pages(channel, &have_view, conversation, SyncWire::ReqHave)
            .await?;

        // Response carries the (bounded) events; the responder's have is streamed after.
        let response = match decode_frame(&channel.recv().await?)? {
            SyncWire::Response(r) => r,
            SyncWire::UpgradeRequired { conversation: c } if c == conversation && !self.v5 => {
                self.v5 = self.v4;
                self.v4 = self.v3;
                self.v3 = self.v2;
                self.v2 = true;
                self.comparison_disabled = true;
                self.started = false;
                return Ok(false);
            }
            _ => return Err(RoundFault::UnexpectedMessage.into()),
        };
        if response.conversation != conversation
            || response.events.iter().any(|event| {
                event.conversation_id != conversation
                    || !supports_event_kind(event.kind, self.v2, self.v3, self.v4, self.v5)
            })
        {
            return Err(RoundFault::UnexpectedMessage.into());
        }
        self.received_wire_total += response.events.len();
        let remote_have = backend
            .receive_have(
                channel,
                conversation,
                comparison_token.is_some() && self.received_wire_total == 0,
            )
            .await?;
        self.remote_have_count = Some(backend.intersection_count(&have_view, &remote_have).await?);
        let report = backend.apply(&response.events)?;
        if backend.admission_denied() {
            return Err(RoundFault::UnexpectedMessage.into());
        }
        // Keep only the owned selection across apply, never a store guard.
        // Ingest or concurrent changes may add causal parents or change policy;
        // only an exact freshly qualified source may reuse prepared decisions.
        let outgoing_view = if backend.reusable_view(&have_view)? {
            have_view
        } else {
            drop(have_view);
            backend.open_prepared_view(&conversation).await?
        };
        let mut followups = backend
            .select_export_frames(&outgoing_view, conversation, &remote_have, max_followups)
            .await?;
        let made_progress = report.applied > 0;
        // New controls may arrive after the fingerprint probe. Complete this
        // round empty and restart with the required capability before sending.
        let needs_v3 = followups
            .iter()
            .flat_map(|frame| &frame.events)
            .any(|event| !supports_event_kind(event.kind, true, false, false, false));
        let needs_v4 = followups
            .iter()
            .flat_map(|frame| &frame.events)
            .any(|event| !supports_event_kind(event.kind, true, true, false, false));
        let needs_v5 = followups
            .iter()
            .flat_map(|frame| &frame.events)
            .any(|event| event.kind == EventKind::ChannelMetadata);
        let unsupported = followups
            .iter()
            .flat_map(|frame| &frame.events)
            .any(|event| !supports_event_kind(event.kind, self.v2, self.v3, self.v4, self.v5));
        if unsupported {
            let empty = SyncFollowup {
                conversation,
                events: Vec::new(),
            };
            let wire = if self.v5 {
                SyncWire::FollowupV5(empty)
            } else if self.v4 {
                SyncWire::FollowupV4(empty)
            } else if self.v3 {
                SyncWire::FollowupV3(empty)
            } else if self.v2 {
                SyncWire::FollowupV2(empty)
            } else {
                SyncWire::Followup(empty)
            };
            channel.send(&encode_frame(&wire)?).await?;
            self.total.applied += report.applied;
            self.total.duplicates += report.duplicates;
            self.v2 = true;
            self.v3 |= needs_v3;
            self.v4 |= needs_v4;
            self.v5 |= needs_v5;
            self.comparison_disabled = true;
            self.started = false;
            self.previous_push.clear();
            return Ok(false);
        }
        // Sending is not progress: projected stores can ingest a duplicate but
        // omit it from their Have set. Require evidence of acceptance before
        // repeating a push, unless this round advanced our own store instead.
        if !self.previous_push.is_empty()
            && !made_progress
            && !remote_have.iter().any(|id| self.previous_push.contains(id))
        {
            // The peer did not accept our push. Preserve only qualified local
            // preparation, never remote acceptance; cache failure cannot replace
            // the protocol outcome. I/O failures/cancellation still drop the view.
            drop(backend.retain_custody_view(outgoing_view));
            return Err(RoundFault::NoProgress.into());
        }
        let more_to_push = !followups.is_empty();
        self.pushed_total += followups
            .iter()
            .map(|frame| frame.events.len())
            .sum::<usize>();
        self.previous_push = followups
            .iter()
            .flat_map(|frame| frame.events.iter().map(|event| event.id))
            .collect();
        if followups.is_empty() {
            followups.push(SyncFollowup {
                conversation,
                events: Vec::new(),
            });
        }
        for followup in followups {
            backend.validate_view(&outgoing_view)?;
            channel
                .send(&encode_frame(&if self.v5 {
                    SyncWire::FollowupV5(followup)
                } else if self.v4 {
                    SyncWire::FollowupV4(followup)
                } else if self.v3 {
                    SyncWire::FollowupV3(followup)
                } else if self.v2 {
                    SyncWire::FollowupV2(followup)
                } else {
                    SyncWire::Followup(followup)
                })?)
                .await?;
        }

        self.total.applied += report.applied;
        self.total.duplicates += report.duplicates;
        // Bound accumulated reject detail across rounds (diagnostics only) so a peer that
        // keeps a multi-round sync alive while interleaving bad events can't grow it.
        if self.total.rejected.len() < MAX_REJECTED_DETAIL {
            self.total.rejected.extend(report.rejected);
        }

        let complete = !made_progress && !more_to_push;
        if !complete && round + 1 == MAX_SYNC_ROUNDS {
            // All followups completed; the bounded-round failure also leaves
            // unchanged local preparation reusable without claiming delivery.
            drop(backend.retain_custody_view(outgoing_view));
            return Err(RoundFault::NoProgress.into());
        }
        if complete
            && self.received_wire_total == 0
            && self.pushed_total == 0
            && self.total.applied == 0
            && self.total.rejected.is_empty()
            && remote_have.len() < local_count
            && self.remote_have_count == Some(remote_have.len())
            && let (Some(local_token), Some(remote_fingerprint)) = (
                comparison_token,
                backend.comparison_fingerprint(&remote_have),
            )
        {
            backend.retain_quiet_comparison(
                &outgoing_view,
                QuietComparison {
                    local_token,
                    remote_fingerprint,
                    remote_count: remote_have.len() as u64,
                },
            )?;
        }
        // Retain after every followup was sent successfully. Changed
        // source/policy rejects retention; I/O failures/cancellation drop the lease.
        // Repeated queries reuse preparation without retaining historical bodies.
        drop(backend.retain_custody_view(outgoing_view)?);
        Ok(complete)
    }
}
