//! 核心存储端口的 SQLite 实现。表结构和迁移在 `migrations/`，属于这个 crate。

use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bedrock_core::note::{Note, NoteBody, NoteId, NoteStore, NoteTitle, StoreError};
use jiff::Timestamp;
use sqlx::Row;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteRow,
};

static MIGRATOR: Migrator = sqlx::migrate!();

/// 一个 SQLite 数据库文件，各业务的存储端口都由它实现。
#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    #[error("打不开数据库 {}", path.display())]
    Connect {
        path: PathBuf,
        #[source]
        source: sqlx::Error,
    },
    #[error("数据库迁移失败")]
    Migrate(#[from] MigrateError),
}

impl SqliteStore {
    /// 打开数据库文件，不存在就新建，然后跑完还没跑的迁移。
    pub async fn open(path: &Path) -> Result<Self, OpenError> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|source| OpenError::Connect {
                path: path.to_owned(),
                source,
            })?;
        MIGRATOR.run(&pool).await?;
        Ok(Self { pool })
    }
}

impl NoteStore for SqliteStore {
    async fn insert(&self, note: &Note) -> Result<(), StoreError> {
        let result =
            sqlx::query("INSERT INTO notes (id, title, body, created_at) VALUES (?, ?, ?, ?)")
                .bind(note.id().as_str())
                .bind(note.title().as_str())
                .bind(note.body().as_str())
                .bind(note.created_at().as_millisecond())
                .execute(&self.pool)
                .await;
        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
                Err(StoreError::Conflict)
            }
            Err(err) => Err(other(err)),
        }
    }

    async fn list(&self) -> Result<Vec<Note>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, title, body, created_at FROM notes ORDER BY created_at DESC, id DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(other)?;
        rows.iter().map(note_from_row).collect()
    }

    async fn delete(&self, id: &NoteId) -> Result<(), StoreError> {
        let result = sqlx::query("DELETE FROM notes WHERE id = ?")
            .bind(id.as_str())
            .execute(&self.pool)
            .await
            .map_err(other)?;
        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }
}

/// 读回来的数据也走领域类型的构造函数，库里的脏数据不会混进核心。
fn note_from_row(row: &SqliteRow) -> Result<Note, StoreError> {
    let id: String = row.try_get("id").map_err(other)?;
    let title: String = row.try_get("title").map_err(other)?;
    let body: String = row.try_get("body").map_err(other)?;
    let created_at: i64 = row.try_get("created_at").map_err(other)?;
    Ok(Note::new(
        NoteId::new(id),
        NoteTitle::new(&title).map_err(other)?,
        NoteBody::new(&body).map_err(other)?,
        Timestamp::from_millisecond(created_at).map_err(other)?,
    ))
}

fn other(err: impl Into<Box<dyn Error + Send + Sync>>) -> StoreError {
    StoreError::Other(err.into())
}
