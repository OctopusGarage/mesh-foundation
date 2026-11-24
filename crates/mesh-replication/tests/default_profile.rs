#![cfg(feature = "mesh-connect")]

use mesh_connect::PeerSession;
use mesh_replication::{
    CompleteInventory,
    inventory_session::{InventorySender, receive_inventory, send_inventory},
    pull::{PullLimits, Record, pull_over_session, serve_record_request},
    session::HaveLimits,
};
use sha2::{Digest, Sha256};
use std::convert::Infallible;
use tokio::net::{TcpListener, TcpStream};
use x25519_dalek::{PublicKey, StaticSecret};

fn public(secret: [u8; 32]) -> [u8; 32] {
    PublicKey::from(&StaticSecret::from(secret)).to_bytes()
}

fn inventory_limits() -> HaveLimits {
    HaveLimits {
        ids_per_frame: 1,
        frames: 1,
        total_ids: 1,
    }
}

const BODY_LIMITS: PullLimits = PullLimits {
    records_per_batch: 1,
    bytes_per_batch: 64,
};

#[tokio::test]
async fn default_inventory_and_record_exchange_over_authenticated_tcp() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let scope = [7; 32];
    let body = b"application-defined record";
    let id: [u8; 32] = Sha256::digest(body).into();
    let source = CompleteInventory::from_ids(&[id], 1).unwrap();
    let local = CompleteInventory::from_ids(&[], 1).unwrap();

    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::join!(
            async {
                let stream = TcpStream::connect(address).await.unwrap();
                let mut peer = PeerSession::connect(stream, &[11; 32], &public([22; 32]))
                    .await
                    .unwrap();
                let remote = receive_inventory(
                    &mut peer,
                    scope,
                    InventorySender::Responder,
                    inventory_limits(),
                )
                .await
                .unwrap();
                let mut stored = None;
                let count =
                    pull_over_session(&mut peer, scope, &remote, &local, BODY_LIMITS, |records| {
                        let [record] = records else {
                            panic!("expected one record")
                        };
                        assert_eq!(<[u8; 32]>::from(Sha256::digest(&record.body)), record.id);
                        stored = Some(record.body.clone());
                        Ok::<_, Infallible>(())
                    })
                    .await
                    .unwrap();
                assert_eq!(count, 1);
                assert_eq!(stored.as_deref(), Some(body.as_slice()));
            },
            async {
                let (stream, _) = listener.accept().await.unwrap();
                let mut peer =
                    PeerSession::accept(stream, &[22; 32], |key| *key == public([11; 32]))
                        .await
                        .unwrap();
                send_inventory(
                    &mut peer,
                    scope,
                    InventorySender::Responder,
                    &source,
                    inventory_limits(),
                    || Ok(()),
                )
                .await
                .unwrap();
                serve_record_request(&mut peer, scope, BODY_LIMITS, async |key, ids, budget| {
                    assert_eq!(*key, public([11; 32]));
                    assert_eq!(ids, &[id]);
                    assert!(body.len() <= budget);
                    Ok::<_, Infallible>(vec![Record {
                        id,
                        body: body.to_vec(),
                    }])
                })
                .await
                .unwrap();
            }
        );
    })
    .await
    .unwrap();
}
