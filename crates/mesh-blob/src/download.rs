use crate::{ChunkStore, SparseBlobReceiver, SparseReceiveError};
use futures_util::{StreamExt, stream};
use std::{future::Future, num::NonZeroUsize};

/// An authorized source for the exact object version. Sources must bound
/// reads by `length` and enforce deadlines and current authorization.
pub trait ChunkSource {
    type Error;

    fn has_chunk(&self, index: u32) -> bool;

    fn fetch(&self, index: u32, length: u32) -> impl Future<Output = Result<Vec<u8>, Self::Error>>;
}

#[derive(Debug)]
pub enum DownloadError<E> {
    /// All advertised sources failed, returned invalid bytes, or lack the block.
    Unavailable(u32),
    Receive(SparseReceiveError<E>),
}

impl SparseBlobReceiver<'_> {
    /// Pull a bounded number of independent chunks concurrently. Rotate the
    /// starting source by index and retry other sources on failure. Durable
    /// progress survives an error or cancellation; retry with updated sources.
    pub async fn download<S: ChunkStore, P: ChunkSource>(
        &mut self,
        store: &mut S,
        sources: &[P],
        concurrency: NonZeroUsize,
    ) -> Result<(), DownloadError<S::Error>> {
        let layout = self.layout;
        let hashes = self.hashes;
        // Own just the current window of indices so commits can update progress
        // while requests borrow the source list.
        let mut next = 0;
        while next < layout.chunk_count() {
            let batch: Vec<_> = self.missing_from(next).take(concurrency.get()).collect();
            let Some(&last) = batch.last() else { break };
            next = last + 1;
            let mut requests = stream::iter(batch)
                .map(|index| async move {
                    let length = layout
                        .chunk_len(index)
                        .expect("index from validated layout");
                    let start = (index as usize).checked_rem(sources.len()).unwrap_or(0);
                    for source in sources[start..].iter().chain(&sources[..start]) {
                        if source.has_chunk(index)
                            && let Ok(bytes) = source.fetch(index, length).await
                            && bytes.len() == length as usize
                            && crate::chunk_hash(&bytes) == hashes[index as usize]
                        {
                            return Ok((index, bytes));
                        }
                    }
                    Err(DownloadError::Unavailable(index))
                })
                .buffer_unordered(concurrency.get());
            let mut unavailable = None;
            while let Some(reply) = requests.next().await {
                match reply {
                    Ok((index, bytes)) => {
                        self.accept(store, index, &bytes)
                            .map_err(DownloadError::Receive)?;
                    }
                    Err(error) => {
                        unavailable.get_or_insert(error);
                    }
                }
            }
            if let Some(error) = unavailable {
                return Err(error);
            }
        }
        self.verify(store).map_err(DownloadError::Receive)
    }
}

/// Decrypt indexed ciphertext at the receiver. The wrapped source only needs
/// the encrypted chunks, alongside the transport's independent authorization.
pub struct EncryptedChunkSource<'a, P> {
    source: P,
    key: &'a crate::FileKey,
    nonce: [u8; 8],
}

impl<'a, P> EncryptedChunkSource<'a, P> {
    pub fn new(source: P, key: &'a crate::FileKey, nonce: [u8; 8]) -> Self {
        Self { source, key, nonce }
    }
}

#[derive(Debug)]
pub enum EncryptedChunkError<E> {
    Source(E),
    Invalid(crate::FileError),
}

impl<P: ChunkSource> ChunkSource for EncryptedChunkSource<'_, P> {
    type Error = EncryptedChunkError<P::Error>;

    fn has_chunk(&self, index: u32) -> bool {
        self.source.has_chunk(index)
    }

    async fn fetch(&self, index: u32, length: u32) -> Result<Vec<u8>, Self::Error> {
        let wire_length = length.checked_add(crate::TAG_SIZE as u32).ok_or_else(|| {
            EncryptedChunkError::Invalid(crate::FileError::Malformed(
                "encrypted chunk too large".into(),
            ))
        })?;
        let ciphertext = self
            .source
            .fetch(index, wire_length)
            .await
            .map_err(EncryptedChunkError::Source)?;
        if ciphertext.len() != wire_length as usize {
            return Err(EncryptedChunkError::Invalid(crate::FileError::Malformed(
                "unexpected encrypted chunk length".into(),
            )));
        }
        crate::open_chunk_indexed(self.key, &self.nonce, index, &ciphertext)
            .map_err(EncryptedChunkError::Invalid)
    }
}
