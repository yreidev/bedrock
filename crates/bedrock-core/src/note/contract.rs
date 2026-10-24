//! [`NoteStore`] 的约定测试。内存版和每个真的存储都跑同一组，保证两边行为一致。

use jiff::Timestamp;

use super::{Note, NoteBody, NoteId, NoteStore, NoteTitle, StoreError};

/// 检查一个存储守不守约定。传进来的存储必须是空的；不守约定时 panic。
pub async fn check_store<S: NoteStore>(store: &S) {
    assert_eq!(ok(store.list().await), Vec::new(), "新存储应该是空的");

    let first = note("a", "买牛奶", "", 1);
    let second = note("b", "交房租", "月底前\n记得要收据", 2);
    let same_time = note("c", "同一秒建的", "", 2);
    ok(store.insert(&first).await);
    ok(store.insert(&second).await);
    ok(store.insert(&same_time).await);
    assert_eq!(
        ok(store.list().await),
        vec![same_time.clone(), second.clone(), first.clone()],
        "新建的在前，同一时刻的编号大的在前，各字段原样读回"
    );

    assert!(
        matches!(store.insert(&first).await, Err(StoreError::Conflict)),
        "编号重复要报 Conflict"
    );

    ok(store.delete(first.id()).await);
    assert_eq!(ok(store.list().await), vec![same_time, second]);
    assert!(
        matches!(store.delete(first.id()).await, Err(StoreError::NotFound)),
        "删不存在的要报 NotFound"
    );
}

fn note(id: &str, title: &str, body: &str, second: i64) -> Note {
    Note::new(
        NoteId::new(id),
        NoteTitle::new(title).unwrap_or_else(|err| panic!("{err}")),
        NoteBody::new(body).unwrap_or_else(|err| panic!("{err}")),
        // 带毫秒，检查存储没有丢掉精度
        Timestamp::constant(1_767_225_600 + second, 250_000_000),
    )
}

fn ok<T>(result: Result<T, StoreError>) -> T {
    result.unwrap_or_else(|err| panic!("存储出错：{err:?}"))
}
