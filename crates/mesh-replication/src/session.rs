//! Transport-independent authenticated frame and inventory transfer contracts.

/// The transport owns peer authentication, ordered delivery and the frame
/// length bound. Discard a session after an interrupted inventory exchange.
pub trait ReconciliationSession {
    type Peer;
    type Error;

    fn peer_identity(&self) -> &Self::Peer;
    fn send(&mut self, frame: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn recv(&mut self) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send;
}

pub struct HaveLimits {
    pub ids_per_frame: usize,
    pub frames: usize,
    pub total_ids: usize,
}

pub struct HaveFrame<Id> {
    pub ids: Vec<Id>,
    pub more: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HaveError<E> {
    Exchange(E),
    Limit,
    Allocation,
    CountMismatch,
}

/// Receive a bounded inventory from framed I/O. The codec checks the protocol
/// variant, conversation and frame contents before returning IDs. This function
/// preserves raw order and duplicates for the caller's admission logic.
pub async fn receive_have<S, Id>(
    session: &mut S,
    limits: HaveLimits,
    mut decode: impl FnMut(&[u8]) -> Result<HaveFrame<Id>, S::Error>,
) -> Result<Vec<Id>, HaveError<S::Error>>
where
    S: ReconciliationSession,
{
    let mut have = Vec::new();
    for _ in 0..limits.frames {
        let bytes = session.recv().await.map_err(HaveError::Exchange)?;
        let frame = decode(&bytes).map_err(HaveError::Exchange)?;
        if frame.ids.len() > limits.ids_per_frame {
            return Err(HaveError::Limit);
        }
        let required = have
            .len()
            .checked_add(frame.ids.len())
            .filter(|&count| count <= limits.total_ids)
            .ok_or(HaveError::Limit)?;
        if required > have.capacity() {
            let target = required
                .max(have.capacity().saturating_mul(2))
                .min(limits.total_ids);
            have.try_reserve_exact(target - have.len())
                .map_err(|_| HaveError::Allocation)?;
            if have.capacity() > limits.total_ids {
                return Err(HaveError::Allocation);
            }
        }
        have.extend(frame.ids);
        if !frame.more {
            return Ok(have);
        }
    }
    Err(HaveError::Limit)
}

/// A finite, already authorized inventory. A page may contain zero IDs and
/// still advance the opaque cursor. The source requalifies authority before
/// every frame is handed to transport.
pub trait HaveSource {
    type Id;
    type Cursor;
    type Error;

    fn page(
        &mut self,
        cursor: Option<&Self::Cursor>,
        ids: usize,
        examined: usize,
    ) -> Result<HavePage<Self::Id, Self::Cursor>, Self::Error>;

    fn validate(&self) -> Result<(), Self::Error>;
}

pub struct HavePage<Id, Cursor> {
    pub ids: Vec<Id>,
    pub next: Option<Cursor>,
}

/// Stream a captured inventory, including an explicit terminal frame. A
/// count disagreement terminates the exchange before publishing a terminal.
pub async fn send_have<S, Source, Yield>(
    session: &mut S,
    source: &mut Source,
    expected_count: usize,
    limits: HaveLimits,
    mut encode: impl FnMut(HaveFrame<Source::Id>) -> Result<Vec<u8>, S::Error>,
    mut yield_work: impl FnMut() -> Yield,
) -> Result<(), HaveError<S::Error>>
where
    S: ReconciliationSession,
    Source: HaveSource<Error = S::Error>,
    Yield: Future<Output = ()>,
{
    if limits.ids_per_frame == 0 || expected_count > limits.total_ids {
        return Err(HaveError::Limit);
    }
    let mut cursor = None;
    let mut total = 0usize;
    let mut frames = 0usize;
    let mut pending = Vec::with_capacity(expected_count.min(limits.ids_per_frame));
    loop {
        let capacity = limits.ids_per_frame - pending.len();
        let budget = capacity.max(1);
        let page = source
            .page(cursor.as_ref(), budget, limits.ids_per_frame)
            .map_err(HaveError::Exchange)?;
        total = total
            .checked_add(page.ids.len())
            .filter(|&n| n <= limits.total_ids)
            .ok_or(HaveError::Limit)?;
        if page.ids.len() > budget {
            return Err(HaveError::Limit);
        }
        cursor = page.next;
        if !page.ids.is_empty() && capacity == 0 {
            frames += 1;
            if frames > limits.frames {
                return Err(HaveError::Limit);
            }
            let bytes = encode(HaveFrame {
                ids: std::mem::take(&mut pending),
                more: true,
            })
            .map_err(HaveError::Exchange)?;
            source.validate().map_err(HaveError::Exchange)?;
            session.send(&bytes).await.map_err(HaveError::Exchange)?;
        }
        pending.reserve_exact(page.ids.len());
        pending.extend(page.ids);
        if cursor.is_none() {
            if total != expected_count {
                return Err(HaveError::CountMismatch);
            }
            if frames == limits.frames {
                return Err(HaveError::Limit);
            }
            let bytes = encode(HaveFrame {
                ids: pending,
                more: false,
            })
            .map_err(HaveError::Exchange)?;
            source.validate().map_err(HaveError::Exchange)?;
            session.send(&bytes).await.map_err(HaveError::Exchange)?;
            return Ok(());
        }
        yield_work().await;
    }
}
