//! Bounded pull of opaque record bodies selected by inventory comparison.

use crate::{CompleteInventory, EventId};
use std::collections::HashSet;

mod session;
pub use session::{MAX_RECORD_FRAME, RecordExchangeError, pull_over_session, serve_record_request};

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

impl<Fetch: std::fmt::Display, Commit: std::fmt::Display> std::fmt::Display
    for PullError<Fetch, Commit>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "record pull stopped after {} committed records: ",
            self.committed
        )?;
        match &self.kind {
            PullErrorKind::InvalidLimits => f.write_str("invalid batch limits"),
            PullErrorKind::Fetch(error) => write!(f, "fetch failed: {error}"),
            PullErrorKind::UnexpectedRecords => {
                f.write_str("response does not match requested IDs")
            }
            PullErrorKind::BodyLimit => f.write_str("response exceeds the body byte budget"),
            PullErrorKind::Commit(error) => write!(f, "commit failed: {error}"),
        }
    }
}

impl<Fetch: std::error::Error + 'static, Commit: std::error::Error + 'static> std::error::Error
    for PullError<Fetch, Commit>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            PullErrorKind::Fetch(error) => Some(error),
            PullErrorKind::Commit(error) => Some(error),
            _ => None,
        }
    }
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
        validate_batch(&ids, &records, limits.bytes_per_batch).map_err(|error| {
            fail(match error {
                InvalidBatch::Records => PullErrorKind::UnexpectedRecords,
                InvalidBatch::Bytes => PullErrorKind::BodyLimit,
            })
        })?;
        commit(&records).map_err(|error| fail(PullErrorKind::Commit(error)))?;
        committed += records.len();
    }
}

enum InvalidBatch {
    Records,
    Bytes,
}

fn validate_batch(ids: &[EventId], records: &[Record], bytes: usize) -> Result<(), InvalidBatch> {
    if records.len() != ids.len() {
        return Err(InvalidBatch::Records);
    }
    let mut expected: HashSet<_> = ids.iter().collect();
    let mut remaining = bytes;
    for record in records {
        if !expected.remove(&record.id) {
            return Err(InvalidBatch::Records);
        }
        remaining = remaining
            .checked_sub(record.body.len())
            .ok_or(InvalidBatch::Bytes)?;
    }
    Ok(())
}
