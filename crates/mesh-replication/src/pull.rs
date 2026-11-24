//! Bounded pull of opaque record bodies selected by inventory comparison.

use crate::{CompleteInventory, EventId};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
pub struct PullLimits {
    pub records_per_batch: usize,
    /// Total body bytes per response, excluding transport framing.
    pub bytes_per_batch: usize,
}

#[derive(Debug)]
pub struct Record {
    pub id: EventId,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct PullError<Fetch, Commit> {
    /// Records acknowledged by earlier successful atomic commits.
    pub committed: usize,
    pub kind: PullErrorKind<Fetch, Commit>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PullErrorKind<Fetch, Commit> {
    InvalidLimits,
    Fetch(Fetch),
    UnexpectedRecords,
    BodyLimit,
    Commit(Commit),
}

/// Pull IDs present remotely and absent locally, for one already authorized
/// record scope. The fetch adapter must bound decoding before allocating
/// response bodies. This function independently checks the returned set and
/// aggregate size. It preserves the order in which the peer returned records.
///
/// The commit adapter must verify content IDs, authenticity and current
/// application authorization, then commit the entire batch atomically. A
/// failed or cancelled pull can be resumed from a fresh committed inventory.
/// Commit is synchronous, with no suspension between validation and reporting.
pub async fn pull_missing<Fetch, Commit>(
    remote: &CompleteInventory,
    local: &CompleteInventory,
    limits: PullLimits,
    mut fetch: impl AsyncFnMut(&[EventId], usize) -> Result<Vec<Record>, Fetch>,
    mut commit: impl FnMut(&[Record]) -> Result<(), Commit>,
) -> Result<usize, PullError<Fetch, Commit>> {
    if limits.records_per_batch == 0 || limits.bytes_per_batch == 0 {
        return Err(PullError {
            committed: 0,
            kind: PullErrorKind::InvalidLimits,
        });
    }
    let mut missing = remote.missing_from(local);
    let mut committed = 0;
    loop {
        let ids: Vec<_> = missing.by_ref().take(limits.records_per_batch).collect();
        if ids.is_empty() {
            return Ok(committed);
        }
        let fail = |kind| PullError { committed, kind };
        let records = fetch(&ids, limits.bytes_per_batch)
            .await
            .map_err(|error| fail(PullErrorKind::Fetch(error)))?;
        if records.len() != ids.len() {
            return Err(fail(PullErrorKind::UnexpectedRecords));
        }
        let mut expected: HashSet<_> = ids.iter().collect();
        let mut remaining = limits.bytes_per_batch;
        for record in &records {
            if !expected.remove(&record.id) {
                return Err(fail(PullErrorKind::UnexpectedRecords));
            }
            remaining = remaining
                .checked_sub(record.body.len())
                .ok_or_else(|| fail(PullErrorKind::BodyLimit))?;
        }
        commit(&records).map_err(|error| fail(PullErrorKind::Commit(error)))?;
        committed += records.len();
    }
}
