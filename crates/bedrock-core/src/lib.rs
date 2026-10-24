//! bedrock 的核心：业务规则和操作面，所有端共用。
//!
//! 核心不碰外部世界，存储、时间、编号都经端口（[`Ports`]）由适配器提供。
//! 各端经 [`Core::call`] 按名字调用操作，Rust 写的端也可以用 [`Core::run`] 直接调。

#![warn(clippy::print_stdout, clippy::print_stderr)]

mod error;
#[cfg(any(test, feature = "fake"))]
pub mod fake;
pub mod note;
mod ops;
mod ports;

pub use error::{ErrorBody, ErrorKind, Internal, OpError};
pub use ops::{Caller, Ctx, Kind, Manifest, Operation, OperationInfo};
pub use ports::{Clock, Ids, Ports};

/// 核心：拿着一套端口，所有操作都经它执行。
pub struct Core<P: Ports> {
    ports: P,
}

impl<P: Ports> Core<P> {
    pub fn new(ports: P) -> Self {
        Self { ports }
    }

    pub(crate) fn ports(&self) -> &P {
        &self.ports
    }
}

ops::operations! {
    note::CreateNote,
    note::ListNotes,
    note::DeleteNote,
}
