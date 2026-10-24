//! 跨业务的端口，和所有端口的总表。

use jiff::Timestamp;

use crate::note::NoteStore;

/// 现在几点。
pub trait Clock: Send + Sync + 'static {
    /// 精确到毫秒。
    fn now(&self) -> Timestamp;
}

/// 生成新编号。
pub trait Ids: Send + Sync + 'static {
    /// 不会重复，按生成的先后排序。
    fn new_id(&self) -> String;
}

/// 核心要的所有端口。各端（或演示模式、测试）实现一套交给 [`crate::Core::new`]。
pub trait Ports: Send + Sync + 'static {
    type Clock: Clock;
    type Ids: Ids;
    type Notes: NoteStore;

    fn clock(&self) -> &Self::Clock;
    fn ids(&self) -> &Self::Ids;
    fn notes(&self) -> &Self::Notes;
}
