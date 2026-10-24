// 辅助函数不在 #[test] 里，clippy 的 allow-unwrap-in-tests 管不到，整个文件放开
#![allow(clippy::unwrap_used, clippy::expect_used)]

use bedrock_core::note::{NoteStore, contract};
use bedrock_store_sqlite::SqliteStore;

#[tokio::test]
async fn sqlite_store_keeps_the_contract() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteStore::open(&dir.path().join("test.db"))
        .await
        .unwrap();
    contract::check_store(&store).await;
}

#[tokio::test]
async fn reopening_keeps_the_data_and_skips_finished_migrations() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    {
        let store = SqliteStore::open(&path).await.unwrap();
        contract::check_store(&store).await;
    }
    let store = SqliteStore::open(&path).await.unwrap();
    assert_eq!(store.list().await.unwrap().len(), 2);
}
