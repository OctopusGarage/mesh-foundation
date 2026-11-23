use mesh_replication::session::*;
use std::collections::VecDeque;

struct Session(VecDeque<Vec<u8>>);

impl ReconciliationSession for Session {
    type Peer = ();
    type Error = &'static str;

    fn peer_identity(&self) -> &Self::Peer {
        &()
    }

    async fn send(&mut self, _frame: &[u8]) -> Result<(), Self::Error> {
        unreachable!("the receiver never writes")
    }

    async fn recv(&mut self) -> Result<Vec<u8>, Self::Error> {
        self.0.pop_front().ok_or("closed")
    }
}

fn decode(frame: &[u8]) -> Result<HaveFrame<u8>, &'static str> {
    let (more, ids) = frame.split_first().ok_or("missing header")?;
    Ok(HaveFrame {
        ids: ids.to_vec(),
        more: *more != 0,
    })
}

fn limits() -> HaveLimits {
    HaveLimits {
        ids_per_frame: 2,
        frames: 3,
        total_ids: 3,
    }
}

#[tokio::test]
async fn preserves_duplicates_and_accepts_exact_terminal_boundary() {
    let mut session = Session([vec![1, 8, 8], vec![0, 9]].into());
    assert_eq!(
        receive_have(&mut session, limits(), decode).await.unwrap(),
        [8, 8, 9]
    );
}

#[tokio::test]
async fn rejects_per_frame_total_and_continuation_limits() {
    for chunks in [
        vec![vec![0, 1, 2, 3]],
        vec![vec![1, 1, 2], vec![0, 3, 4]],
        vec![vec![1], vec![1], vec![1]],
    ] {
        let mut session = Session(chunks.into());
        assert_eq!(
            receive_have(&mut session, limits(), decode).await,
            Err(HaveError::Limit)
        );
    }
}

#[tokio::test]
async fn preserves_decode_failure_and_premature_close() {
    for (chunks, expected) in [
        (vec![Vec::new()], "missing header"),
        (vec![vec![1, 1]], "closed"),
    ] {
        let mut session = Session(chunks.into());
        assert_eq!(
            receive_have(&mut session, limits(), decode).await,
            Err(HaveError::Exchange(expected))
        );
    }
}
