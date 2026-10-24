//! Dart 那边看得到的全部东西：[`Bedrock`] 和 [`CallError`]。

use std::path::Path;
use std::sync::{Arc, LazyLock};

use bedrock_core::{Core, Ctx, ErrorBody};
use bedrock_local::LocalPorts;
use flutter_rust_bridge::frb;

/// 打不开数据目录，或者操作失败。`code` 给程序判断，`message` 给人看。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallError {
    pub code: String,
    pub message: String,
}

impl CallError {
    fn internal(message: String) -> Self {
        Self {
            code: "internal".to_owned(),
            message,
        }
    }
}

impl From<ErrorBody> for CallError {
    fn from(body: ErrorBody) -> Self {
        Self {
            code: body.code.to_owned(),
            message: body.message,
        }
    }
}

/// 本机的核心，数据在打开时给的目录里。
#[frb(opaque)]
pub struct Bedrock {
    core: Arc<Core<LocalPorts>>,
}

impl Bedrock {
    /// 打开 `data_dir` 里的数据，目录不在就建，表结构自动迁移到最新。
    pub async fn open(data_dir: String) -> Result<Bedrock, CallError> {
        let core = spawn(async move { bedrock_local::open(Path::new(&data_dir)).await })
            .await?
            .map_err(|err| CallError::internal(format!("打不开数据目录：{err}")))?;
        Ok(Self {
            core: Arc::new(core),
        })
    }

    /// 按名字调一个操作，输入输出都是 JSON 对象的字符串。
    pub async fn call(&self, name: String, input_json: String) -> Result<String, CallError> {
        let input = serde_json::from_str(&input_json).map_err(ErrorBody::invalid_input)?;
        let core = Arc::clone(&self.core);
        let output = spawn(async move { core.call(&Ctx::local(), &name, input).await }).await??;
        Ok(output.to_string())
    }
}

/// 存储要在 tokio 里跑，而 Dart 调进来的线程不归 tokio 管，所以自己带一个小运行时。
static RUNTIME: LazyLock<Result<tokio::runtime::Runtime, String>> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("bedrock")
        .enable_all()
        .build()
        .map_err(|err| err.to_string())
});

async fn spawn<T: Send + 'static>(
    future: impl Future<Output = T> + Send + 'static,
) -> Result<T, CallError> {
    let runtime = RUNTIME
        .as_ref()
        .map_err(|err| CallError::internal(format!("起不来异步运行时：{err}")))?;
    runtime
        .spawn(future)
        .await
        .map_err(|err| CallError::internal(format!("操作中途出错：{err}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn opens_and_calls_by_name() {
        let data = tempfile::tempdir().unwrap();
        let bedrock = Bedrock::open(data.path().to_string_lossy().into_owned())
            .await
            .unwrap();

        let created = bedrock
            .call("note.create".to_owned(), r#"{"title":"买牛奶"}"#.to_owned())
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_str(&created).unwrap();
        assert_eq!(created["title"], "买牛奶");

        let empty = bedrock
            .call("note.create".to_owned(), r#"{"title":" "}"#.to_owned())
            .await
            .unwrap_err();
        assert_eq!(empty.code, "empty_title");

        let bad = bedrock
            .call("note.list".to_owned(), "不是 JSON".to_owned())
            .await
            .unwrap_err();
        assert_eq!(bad.code, "invalid_input");
    }
}
