use std::error::Error;
use std::future::Future;

use super::{Note, NoteId};

/// 存备忘的地方。约定见 [`super::contract::check_store`]，真的和假的实现都要通过。
pub trait NoteStore: Send + Sync + 'static {
    /// 存一条新备忘。编号已存在时返回 [`StoreError::Conflict`]。
    fn insert(&self, note: &Note) -> impl Future<Output = Result<(), StoreError>> + Send;

    /// 所有备忘，新建的在前；同一时刻建的，编号大的在前。
    fn list(&self) -> impl Future<Output = Result<Vec<Note>, StoreError>> + Send;

    /// 删掉一条备忘。不存在时返回 [`StoreError::NotFound`]。
    fn delete(&self, id: &NoteId) -> impl Future<Output = Result<(), StoreError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("编号已存在")]
    Conflict,
    #[error("备忘不存在")]
    NotFound,
    /// 适配器认不出的错误，原样带着
    #[error("存储出错")]
    Other(#[source] Box<dyn Error + Send + Sync>),
}
