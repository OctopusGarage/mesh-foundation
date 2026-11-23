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
