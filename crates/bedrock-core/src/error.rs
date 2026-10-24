use std::error::Error;
use std::fmt;

use serde::Serialize;

/// 错误的大类，决定各端怎么报，比如服务端的 HTTP 状态码。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// 输入不对：格式错误、校验没过
    Invalid,
    /// 不知道调用方是谁
    Unauthorized,
    /// 知道是谁，但没有权限
    Forbidden,
    /// 要找的东西不存在
    NotFound,
    /// 和现有的状态冲突
    Conflict,
    /// 意料之外的错误，详情只在日志里
    Internal,
}

/// 操作的错误。每种情况有一个稳定的 `code` 给程序判断，`Display` 是给人看的中文说明。
///
/// 用 [`op_error!`] 实现，每种情况只写一次 `code` 和大类。
pub trait OpError: Error + Send + Sync + 'static {
    /// 可能出现的所有 `code`，写进操作清单。
    const CODES: &'static [&'static str];
    fn code(&self) -> &'static str;
    fn kind(&self) -> ErrorKind;
}

/// 给操作的错误枚举实现 [`OpError`]：
///
/// ```ignore
/// op_error! {
///     DeleteNoteError {
///         NotFound => "not_found", NotFound;
///         Internal(_) => "internal", Internal;
///     }
/// }
/// ```
macro_rules! op_error {
    ($ty:ident { $( $variant:ident $( ( $($field:tt)* ) )? => $code:literal, $kind:ident; )* }) => {
        impl $crate::OpError for $ty {
            const CODES: &'static [&'static str] = &[$($code),*];

            fn code(&self) -> &'static str {
                match self {
                    $( Self::$variant $( ( $($field)* ) )? => $code, )*
                }
            }

            fn kind(&self) -> $crate::ErrorKind {
                match self {
                    $( Self::$variant $( ( $($field)* ) )? => $crate::ErrorKind::$kind, )*
                }
            }
        }
    };
}
pub(crate) use op_error;

/// 意料之外的错误。原始错误留给日志，对外只报 `internal`。
#[derive(Debug)]
pub struct Internal(Box<dyn Error + Send + Sync>);

impl Internal {
    pub fn new(err: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
        Self(err.into())
    }
}

impl fmt::Display for Internal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for Internal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

/// 对外的错误，各端原样交给调用方：`{ "code": "empty_title", "message": "标题不能为空" }`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
    #[serde(skip)]
    pub kind: ErrorKind,
}

impl ErrorBody {
    /// 输入解析不了：不是 JSON、少了字段、类型不对。
    pub fn invalid_input(detail: impl fmt::Display) -> Self {
        Self {
            code: "invalid_input",
            message: format!("输入不对：{detail}"),
            kind: ErrorKind::Invalid,
        }
    }

    pub fn unknown_operation(name: &str) -> Self {
        Self {
            code: "unknown_operation",
            message: format!("没有叫 {name} 的操作"),
            kind: ErrorKind::NotFound,
        }
    }

    pub fn internal() -> Self {
        Self {
            code: "internal",
            message: "出了点问题，详情在日志里".to_owned(),
            kind: ErrorKind::Internal,
        }
    }

    /// 把操作的错误变成对外的样子。意料之外的错误在这里记日志，对外只说出了问题。
    pub fn from_op<E: OpError>(op: &str, err: &E) -> Self {
        match err.kind() {
            ErrorKind::Internal => {
                tracing::error!(op, error = %Chain(err), "操作出错");
                Self::internal()
            }
            kind => Self {
                code: err.code(),
                message: err.to_string(),
                kind,
            },
        }
    }
}

impl fmt::Display for ErrorBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}（{}）", self.message, self.code)
    }
}

impl Error for ErrorBody {}

/// 把错误连同它的来源一层层打出来：`存储出错: database is locked`。
pub(crate) struct Chain<'a>(pub(crate) &'a dyn Error);

impl fmt::Display for Chain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)?;
        let mut source = self.0.source();
        while let Some(err) = source {
            write!(f, ": {err}")?;
            source = err.source();
        }
        Ok(())
    }
}
