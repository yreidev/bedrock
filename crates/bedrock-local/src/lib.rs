//! 在本机跑核心：数据存在数据目录里的 SQLite 文件，时间取系统时间，编号用 UUID v7。
//! 命令行、服务端、桌面和手机应用、原生接口出口都用它组装核心。

use std::io;
use std::path::{Path, PathBuf};

use bedrock_core::{Clock, Core, Ids, Ports};
use bedrock_store_sqlite::SqliteStore;
use jiff::Timestamp;

/// 数据目录里的数据库文件名。
pub const DB_FILE: &str = "bedrock.db";

/// 系统的应用数据目录下的 `bedrock`，比如 Linux 上的 `~/.local/share/bedrock`。
pub fn default_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("bedrock"))
}

#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    #[error("建不了数据目录 {}", path.display())]
    DataDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    Store(#[from] bedrock_store_sqlite::OpenError),
}

/// 打开数据目录里的核心，目录不存在就建一个。
pub async fn open(data_dir: &Path) -> Result<Core<LocalPorts>, OpenError> {
    std::fs::create_dir_all(data_dir).map_err(|source| OpenError::DataDir {
        path: data_dir.to_owned(),
        source,
    })?;
    let notes = SqliteStore::open(&data_dir.join(DB_FILE)).await?;
    Ok(Core::new(LocalPorts {
        clock: SystemClock,
        ids: RandomIds,
        notes,
    }))
}

/// 系统时间，精确到毫秒。
#[derive(Debug, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let now = Timestamp::now();
        // 当前时间不会超出范围，失败时用原值
        Timestamp::from_millisecond(now.as_millisecond()).unwrap_or(now)
    }
}

/// UUID v7：不会重复，按生成的先后排序。
#[derive(Debug, Clone, Copy)]
pub struct RandomIds;

impl Ids for RandomIds {
    fn new_id(&self) -> String {
        uuid::Uuid::now_v7().to_string()
    }
}

#[derive(Debug)]
pub struct LocalPorts {
    clock: SystemClock,
    ids: RandomIds,
    notes: SqliteStore,
}

impl Ports for LocalPorts {
    type Clock = SystemClock;
    type Ids = RandomIds;
    type Notes = SqliteStore;

    fn clock(&self) -> &SystemClock {
        &self.clock
    }

    fn ids(&self) -> &RandomIds {
        &self.ids
    }

    fn notes(&self) -> &SqliteStore {
        &self.notes
    }
}
