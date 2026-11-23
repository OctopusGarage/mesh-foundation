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

struct Source(VecDeque<HavePage<u8, usize>>, bool);

impl HaveSource for Source {
    type Id = u8;
    type Cursor = usize;
    type Error = &'static str;

    fn page(
        &mut self,
        _: Option<&usize>,
        ids: usize,
        _: usize,
    ) -> Result<HavePage<u8, usize>, Self::Error> {
        let page = self.0.pop_front().ok_or("no page")?;
        assert!(page.ids.len() <= ids);
        Ok(page)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        if self.1 {
            Err("authority retired")
        } else {
            Ok(())
        }
    }
}

struct Writer(Vec<Vec<u8>>);

impl ReconciliationSession for Writer {
    type Peer = ();
    type Error = &'static str;

    fn peer_identity(&self) -> &Self::Peer {
        &()
    }

    async fn send(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.0.push(frame.to_vec());
        Ok(())
    }

    async fn recv(&mut self) -> Result<Vec<u8>, Self::Error> {
        unreachable!()
    }
}

fn encode(frame: HaveFrame<u8>) -> Result<Vec<u8>, &'static str> {
    let mut bytes = vec![u8::from(frame.more)];
    bytes.extend(frame.ids);
    Ok(bytes)
}

#[tokio::test]
async fn sends_nonterminal_frames_only_after_lookahead_and_checks_exact_count() {
    let mut source = Source(
        [
            HavePage {
                ids: vec![1, 2],
                next: Some(1),
            },
            HavePage {
                ids: vec![],
                next: Some(2),
            },
            HavePage {
                ids: vec![3],
                next: None,
            },
        ]
        .into(),
        false,
    );
    let mut writer = Writer(Vec::new());
    send_have(&mut writer, &mut source, 3, limits(), encode, || {
        std::future::ready(())
    })
    .await
    .unwrap();
    assert_eq!(writer.0, [vec![1, 1, 2], vec![0, 3]]);

    let mut source = Source(
        [HavePage {
            ids: vec![],
            next: None,
        }]
        .into(),
        false,
    );
    let mut writer = Writer(Vec::new());
    assert_eq!(
        send_have(&mut writer, &mut source, 1, limits(), encode, || {
            std::future::ready(())
        })
        .await,
        Err(HaveError::CountMismatch)
    );
    assert!(writer.0.is_empty());
}

#[tokio::test]
async fn rechecks_authority_before_sending_even_an_empty_terminal() {
    let mut source = Source(
        [HavePage {
            ids: vec![],
            next: None,
        }]
        .into(),
        true,
    );
    let mut writer = Writer(Vec::new());
    assert_eq!(
        send_have(&mut writer, &mut source, 0, limits(), encode, || {
            std::future::ready(())
        })
        .await,
        Err(HaveError::Exchange("authority retired"))
    );
    assert!(writer.0.is_empty());
}
