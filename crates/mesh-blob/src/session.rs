//! Single-object chunk exchange over an authenticated mesh-connect session.
use crate::ChunkSource;
use mesh_connect::{MAX_PLAINTEXT, PeerSession, TransportError};
use std::{io, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
    time::timeout,
};

const REQUEST: &[u8; 4] = b"MBC1";
const DATA: u8 = 1;
const MISSING: u8 = 0;
pub const MAX_CHUNK_BODY: usize = MAX_PLAINTEXT - 1;

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid blob chunk exchange")
}

/// One peer's availability snapshot and reusable authenticated connection.
/// The host performs discovery, verifies the peer and updates availability
/// by constructing a new source. This adapter returns the wire body as-is.
pub struct SessionChunkSource<S> {
    object: [u8; 32],
    available: Vec<bool>,
    session: Mutex<Option<PeerSession<S>>>,
    deadline: Duration,
}

impl<S> SessionChunkSource<S> {
    pub fn new(
        object: [u8; 32],
        available: Vec<bool>,
        session: PeerSession<S>,
        deadline: Duration,
    ) -> Self {
        Self {
            object,
            available,
            session: Mutex::new(Some(session)),
            deadline,
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> ChunkSource for SessionChunkSource<S> {
    type Error = TransportError;

    fn has_chunk(&self, index: u32) -> bool {
        self.available.get(index as usize).copied().unwrap_or(false)
    }

    async fn fetch(&self, index: u32, length: u32) -> Result<Vec<u8>, TransportError> {
        if length as usize > MAX_CHUNK_BODY {
            return Err(invalid().into());
        }
        timeout(self.deadline, async {
            let mut slot = self.session.lock().await;
            let mut session = slot
                .take()
                .ok_or(io::Error::from(io::ErrorKind::NotConnected))?;
            let mut request = [0; 44];
            request[..4].copy_from_slice(REQUEST);
            request[4..36].copy_from_slice(&self.object);
            request[36..40].copy_from_slice(&index.to_be_bytes());
            request[40..].copy_from_slice(&length.to_be_bytes());
            session.send(&request).await?;
            let reply = session.recv().await?;
            let result = match reply.split_first() {
                Some((&DATA, body)) if body.len() == length as usize => Ok(body.to_vec()),
                Some((&MISSING, [])) => Err(io::Error::from(io::ErrorKind::NotFound).into()),
                _ => return Err(invalid().into()),
            };
            *slot = Some(session);
            result
        })
        .await
        .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?
    }
}

/// Serve one request. The loader must authorize the authenticated peer for
/// every object and index before reading. Return None for unavailable data.
pub async fn serve_chunk<S: AsyncRead + AsyncWrite + Unpin>(
    session: &mut PeerSession<S>,
    load: impl FnOnce(&[u8; 32], [u8; 32], u32, u32) -> Option<Vec<u8>>,
) -> Result<(), TransportError> {
    let bytes = session.recv().await?;
    let request: [u8; 44] = bytes.try_into().map_err(|_| invalid())?;
    if &request[..4] != REQUEST {
        return Err(invalid().into());
    }
    let object = request[4..36].try_into().unwrap();
    let index = u32::from_be_bytes(request[36..40].try_into().unwrap());
    let length = u32::from_be_bytes(request[40..].try_into().unwrap());
    if length as usize > MAX_CHUNK_BODY {
        return Err(invalid().into());
    }
    let body = load(session.peer_key(), object, index, length);
    if let Some(body) = body {
        if body.len() != length as usize {
            return Err(invalid().into());
        }
        let mut reply = Vec::with_capacity(body.len() + 1);
        reply.push(DATA);
        reply.extend_from_slice(&body);
        session.send(&reply).await
    } else {
        session.send(&[MISSING]).await
    }
}
