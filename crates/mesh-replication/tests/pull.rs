use mesh_replication::{
    CompleteInventory,
    pull::{PullErrorKind, PullLimits, PullProgress, Record, pull_missing, pull_missing_observed},
};

fn inventory(ids: &[u8]) -> CompleteInventory {
    let ids: Vec<_> = ids.iter().map(|id| [*id; 32]).collect();
    CompleteInventory::from_ids(&ids, 4).unwrap()
}

#[tokio::test]
async fn reports_committed_prefix_when_the_next_batch_is_invalid() {
    let remote = inventory(&[1, 2, 3]);
    let local = inventory(&[1]);
    let mut committed = Vec::new();
    let mut progress = Vec::new();
    let error = pull_missing_observed(
        &remote,
        &local,
        PullLimits {
            records_per_batch: 1,
            bytes_per_batch: 4,
        },
        async |ids, bytes| {
            assert_eq!(bytes, 4);
            let [id] = ids else {
                panic!("expected one requested record")
            };
            let reply = if *id == [3; 32] { [4; 32] } else { *id };
            Ok::<_, ()>(vec![Record {
                id: reply,
                body: vec![1],
            }])
        },
        |records| {
            committed.push(records[0].id);
            Ok::<_, ()>(())
        },
        |event| progress.push(event),
    )
    .await
    .unwrap_err();
    assert_eq!(committed, [[2; 32]]);
    assert_eq!(error.committed, 1);
    assert_eq!(
        progress,
        [PullProgress {
            batch_records: 1,
            committed_records: 1
        }]
    );
    assert_eq!(error.kind, PullErrorKind::UnexpectedRecords);
}

#[tokio::test]
async fn rejects_duplicate_and_oversized_responses_before_commit() {
    let remote = inventory(&[1, 2]);
    let local = inventory(&[]);
    for (second, expected) in [
        (1, PullErrorKind::UnexpectedRecords),
        (2, PullErrorKind::BodyLimit),
    ] {
        let error = pull_missing(
            &remote,
            &local,
            PullLimits {
                records_per_batch: 2,
                bytes_per_batch: 1,
            },
            async |_: &[[u8; 32]], _| {
                Ok::<_, ()>(vec![
                    Record {
                        id: [1; 32],
                        body: vec![],
                    },
                    Record {
                        id: [second; 32],
                        body: if second == 2 { vec![0, 1] } else { vec![] },
                    },
                ])
            },
            |_| -> Result<(), ()> { panic!("invalid response was committed") },
        )
        .await
        .unwrap_err();
        assert_eq!(error.committed, 0);
        assert_eq!(error.kind, expected);
    }
}

#[tokio::test]
async fn equal_inventory_does_not_fetch_or_commit() {
    let local = inventory(&[1]);
    let committed = pull_missing(
        &local,
        &local,
        PullLimits {
            records_per_batch: 1,
            bytes_per_batch: 1,
        },
        async |_: &[[u8; 32]], _| -> Result<Vec<Record>, ()> { panic!("nothing is missing") },
        |_| -> Result<(), ()> { panic!("nothing was fetched") },
    )
    .await
    .unwrap();
    assert_eq!(committed, 0);
}
