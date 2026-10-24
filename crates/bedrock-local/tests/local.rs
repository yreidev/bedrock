// 辅助函数不在 #[test] 里，clippy 的 allow-unwrap-in-tests 管不到，整个文件放开
#![allow(clippy::unwrap_used, clippy::expect_used)]

use bedrock_core::{Ctx, Ids};
use serde_json::json;

#[tokio::test]
async fn notes_survive_reopening_the_data_dir() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("新目录");
    let created = {
        let core = bedrock_local::open(&data).await.unwrap();
        core.call(&Ctx::local(), "note.create", json!({ "title": "买牛奶" }))
            .await
            .unwrap()
    };
    assert!(data.join(bedrock_local::DB_FILE).exists());

    let core = bedrock_local::open(&data).await.unwrap();
    let list = core
        .call(&Ctx::local(), "note.list", json!({}))
        .await
        .unwrap();
    assert_eq!(list, json!({ "notes": [created] }));
}

#[test]
fn ids_sort_in_creation_order() {
    let ids: Vec<String> = (0..100)
        .map(|_| bedrock_local::RandomIds.new_id())
        .collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
}
