use mesh_identity::device::DeviceIdentity;
use mesh_replication::{
    legacy_event::{ConversationId, Event, EventId, EventKind},
    legacy_round::{
        ApplyReport, Probe, QuietComparison, RoundBackend, RoundFault, SyncMetadata, SyncProgress,
    },
    legacy_wire::{self, FpResponse, HaveChunk, SyncFollowup, SyncResponse, SyncWire},
    session::ReconciliationSession,
};
use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
};

#[derive(Default)]
struct Connection {
    incoming: VecDeque<Vec<u8>>,
    sent: Vec<Vec<u8>>,
}
impl ReconciliationSession for Connection {
    type Peer = ();
    type Error = RoundFault;

    fn peer_identity(&self) -> &Self::Peer {
        &()
    }
    async fn send(&mut self, bytes: &[u8]) -> Result<(), RoundFault> {
        self.sent.push(bytes.to_vec());
        Ok(())
    }
    async fn recv(&mut self) -> Result<Vec<u8>, RoundFault> {
        self.incoming
            .pop_front()
            .ok_or(RoundFault::UnexpectedMessage)
    }
}

#[derive(Default)]
struct Replica {
    fingerprint: Option<[u8; 32]>,
    accepted: RefCell<HashSet<EventId>>,
}
impl RoundBackend<RoundFault> for Replica {
    type Peer = ();
    type View = ();
    type RemoteHave = Box<HashSet<EventId>>;

    fn bind_session_peer(&self, _: &()) {}
    async fn prepared_probe(&self, _: &ConversationId) -> Result<Probe<()>, RoundFault> {
        Ok((
            SyncMetadata {
                count: self.accepted.borrow().len() as u64,
                fingerprint: self.fingerprint,
                ..SyncMetadata::default()
            },
            None,
        ))
    }
    async fn open_prepared_view(&self, _: &ConversationId) -> Result<(), RoundFault> {
        Ok(())
    }
    fn advertised_count(&self, _: &()) -> Result<usize, RoundFault> {
        Ok(self.accepted.borrow().len())
    }
    fn have_count(&self, count: u64) -> Result<usize, RoundFault> {
        usize::try_from(count).map_err(|_| RoundFault::CounterExhausted)
    }
    fn validate_view(&self, _: &()) -> Result<(), RoundFault> {
        Ok(())
    }
    fn reusable_view(&self, _: &()) -> Result<bool, RoundFault> {
        Ok(true)
    }
    fn retain_custody_view(&self, _: ()) -> Result<Option<()>, RoundFault> {
        Ok(None)
    }
    fn quiet_comparison(&self, _: &ConversationId) -> Result<Option<QuietComparison>, RoundFault> {
        Ok(None)
    }
    fn validate_quiet_comparison(
        &self,
        _: &ConversationId,
        _: QuietComparison,
    ) -> Result<bool, RoundFault> {
        Ok(false)
    }
    fn quiet_comparison_token(&self, _: &()) -> Result<Option<[u8; 32]>, RoundFault> {
        Ok(None)
    }
    fn retain_quiet_comparison(&self, _: &(), _: QuietComparison) -> Result<(), RoundFault> {
        Ok(())
    }
    async fn send_have_pages(
        &self,
        session: &mut impl ReconciliationSession<Peer = (), Error = RoundFault>,
        _: &(),
        scope: ConversationId,
        wrap: fn(HaveChunk) -> SyncWire,
    ) -> Result<(), RoundFault> {
        let ids = self.accepted.borrow().iter().copied().collect();
        let frame = legacy_wire::encode(&wrap(HaveChunk {
            conversation: scope,
            ids,
            more: false,
        }))
        .map_err(RoundFault::Wire)?;
        session.send(&frame).await
    }
    async fn receive_have(
        &self,
        session: &mut impl ReconciliationSession<Peer = (), Error = RoundFault>,
        scope: ConversationId,
        _: bool,
    ) -> Result<Self::RemoteHave, RoundFault> {
        let bytes = session.recv().await?;
        match legacy_wire::decode(&bytes).map_err(RoundFault::Wire)? {
            SyncWire::RespHave(page) if page.conversation == scope && !page.more => {
                Ok(Box::new(page.ids.into_iter().collect()))
            }
            _ => Err(RoundFault::UnexpectedMessage),
        }
    }
    async fn intersection_count(
        &self,
        _: &(),
        remote: &Self::RemoteHave,
    ) -> Result<usize, RoundFault> {
        Ok(self.accepted.borrow().intersection(remote).count())
    }
    fn comparison_fingerprint(&self, _: &Self::RemoteHave) -> Option<[u8; 32]> {
        None
    }
    fn apply(&self, events: &[Event]) -> Result<ApplyReport, RoundFault> {
        let mut report = ApplyReport::default();
        for event in events {
            if !event.verify_integrity() || !event.verify_signature() {
                return Err(RoundFault::UnexpectedMessage);
            }
            if self.accepted.borrow_mut().insert(event.id) {
                report.applied += 1;
            } else {
                report.duplicates += 1;
            }
        }
        Ok(report)
    }
    fn admission_denied(&self) -> bool {
        false
    }
    async fn select_export_frames(
        &self,
        _: &(),
        _: ConversationId,
        _: &Self::RemoteHave,
        _: usize,
    ) -> Result<Vec<SyncFollowup>, RoundFault> {
        Ok(vec![])
    }
}

fn incoming(frames: impl IntoIterator<Item = SyncWire>) -> Connection {
    Connection {
        incoming: frames
            .into_iter()
            .map(|frame| legacy_wire::encode(&frame).unwrap())
            .collect(),
        sent: Vec::new(),
    }
}

#[tokio::test]
async fn matching_fingerprint_exits_without_inventory_or_export() {
    let replica = Replica {
        fingerprint: Some([19; 32]),
        ..Replica::default()
    };
    let mut connection = incoming([SyncWire::FpResponse(FpResponse { matched: true })]);
    let mut progress = SyncProgress::default();
    assert!(
        progress
            .step(&mut connection, &replica, ConversationId::new([7; 32]), 1)
            .await
            .unwrap()
    );
    assert_eq!(connection.sent.len(), 1);
    assert!(matches!(
        legacy_wire::decode(&connection.sent[0]).unwrap(),
        SyncWire::FpRequest(_)
    ));
}

#[tokio::test]
async fn a_reconnected_requester_advertises_the_event_already_committed() {
    let scope = ConversationId::new([7; 32]);
    let event = Event::new(
        &DeviceIdentity::generate(),
        scope,
        1,
        vec![],
        1,
        0,
        EventKind::Message,
        vec![1],
    );
    let response_have = || {
        SyncWire::RespHave(HaveChunk {
            conversation: scope,
            ids: vec![event.id],
            more: false,
        })
    };
    let replica = Replica::default();
    let mut first = incoming([
        SyncWire::Response(SyncResponse {
            conversation: scope,
            events: vec![event.clone()],
            have: vec![],
        }),
        response_have(),
    ]);
    assert!(
        !SyncProgress::default()
            .step(&mut first, &replica, scope, 1)
            .await
            .unwrap()
    );
    assert!(replica.accepted.borrow().contains(&event.id));
    let mut second = incoming([
        SyncWire::Response(SyncResponse {
            conversation: scope,
            events: vec![],
            have: vec![],
        }),
        response_have(),
    ]);
    assert!(
        SyncProgress::default()
            .step(&mut second, &replica, scope, 1)
            .await
            .unwrap()
    );
    let sent = second
        .sent
        .into_iter()
        .map(|bytes| legacy_wire::decode(&bytes).unwrap())
        .collect::<Vec<_>>();
    assert!(
        matches!(&sent[..], [SyncWire::Request(_), SyncWire::ReqHave(have), SyncWire::Followup(_)]
        if have.ids == [event.id] && !have.more)
    );
}
