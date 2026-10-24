//! 操作面：核心对外的每件事都是一个带名字的操作，各端按名字调用。

use std::future::Future;

use schemars::generate::{SchemaGenerator, SchemaSettings};
use schemars::transform::RecursiveTransform;
use schemars::{JsonSchema, Schema};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::error::Chain;
use crate::{Core, ErrorBody, OpError, Ports};

/// 操作的种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// 只读，不改任何状态：可以重试、可以缓存
    Query,
    /// 会改状态：失败后不自动重试
    Action,
}

/// 谁在调用。各端按自己的认证方式填好，权限在操作里判断。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Caller {
    /// 本机直接调用：命令行、桌面和手机应用、原生接口
    Local,
    /// 经网络进来的请求
    Remote,
}

/// 一次调用的上下文，由端填好传进来。
#[derive(Debug, Clone)]
pub struct Ctx {
    pub caller: Caller,
}

impl Ctx {
    pub fn local() -> Self {
        Self {
            caller: Caller::Local,
        }
    }

    pub fn remote() -> Self {
        Self {
            caller: Caller::Remote,
        }
    }
}

/// 一个操作。写好后在 `lib.rs` 的 `operations!` 里登记，各端和生成的类型就都有了。
pub trait Operation {
    /// `<业务>.<动作>`，发布后不能改。
    const NAME: &'static str;
    const KIND: Kind;
    type Input: DeserializeOwned + JsonSchema + Send;
    type Output: Serialize + JsonSchema;
    type Error: OpError;

    fn run<P: Ports>(
        core: &Core<P>,
        ctx: &Ctx,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

impl<P: Ports> Core<P> {
    /// 直接调一个操作，输入输出都是 Rust 类型。给 Rust 写的端用，比如命令行的子命令。
    pub async fn run<O: Operation>(
        &self,
        ctx: &Ctx,
        input: O::Input,
    ) -> Result<O::Output, O::Error> {
        O::run(self, ctx, input).await
    }

    pub(crate) async fn run_json<O: Operation>(
        &self,
        ctx: &Ctx,
        input: Value,
    ) -> Result<Value, ErrorBody> {
        let input = serde_json::from_value(input).map_err(ErrorBody::invalid_input)?;
        let output = O::run(self, ctx, input)
            .await
            .map_err(|err| ErrorBody::from_op(O::NAME, &err))?;
        serde_json::to_value(output).map_err(|err| {
            tracing::error!(op = O::NAME, error = %Chain(&err), "输出转不成 JSON");
            ErrorBody::internal()
        })
    }
}

/// 登记操作，生成 `Core::call` 和 `manifest()`。
macro_rules! operations {
    ($($op:path),* $(,)?) => {
        impl<P: $crate::Ports> $crate::Core<P> {
            /// 按名字调用一个操作，输入、输出都是 JSON。
            ///
            /// 名字不存在返回 `unknown_operation`，输入解析不了返回 `invalid_input`，
            /// 其余是操作自己的错误。
            pub async fn call(
                &self,
                ctx: &$crate::Ctx,
                name: &str,
                input: ::serde_json::Value,
            ) -> Result<::serde_json::Value, $crate::ErrorBody> {
                $(
                    if name == <$op as $crate::Operation>::NAME {
                        return self.run_json::<$op>(ctx, input).await;
                    }
                )*
                Err($crate::ErrorBody::unknown_operation(name))
            }
        }

        /// 操作清单：所有登记了的操作，加上输入输出的 JSON Schema。`just gen` 把它导出到
        /// `schema/ops.json`。
        pub fn manifest() -> $crate::Manifest {
            let mut builder = $crate::ops::ManifestBuilder::new();
            $( builder.add::<$op>(); )*
            builder.finish()
        }
    };
}
pub(crate) use operations;

/// 操作清单。
#[derive(Debug, Serialize)]
pub struct Manifest {
    /// 核心的版本
    pub version: &'static str,
    pub operations: Vec<OperationInfo>,
    /// 输入输出用到的所有类型
    #[serde(rename = "$defs")]
    pub defs: Map<String, Value>,
}

#[derive(Debug, Serialize)]
pub struct OperationInfo {
    pub name: &'static str,
    pub kind: Kind,
    pub input: Schema,
    pub output: Schema,
    /// 可能返回的错误码，不含各端共有的 `invalid_input`、`unknown_operation`
    pub errors: &'static [&'static str],
}

pub(crate) struct ManifestBuilder {
    generator: SchemaGenerator,
    operations: Vec<OperationInfo>,
}

impl ManifestBuilder {
    pub(crate) fn new() -> Self {
        let mut settings = SchemaSettings::draft2020_12();
        settings
            .transforms
            .push(Box::new(RecursiveTransform(drop_optional_null)));
        Self {
            generator: settings.into_generator(),
            operations: Vec::new(),
        }
    }

    pub(crate) fn add<O: Operation>(&mut self) {
        let input = self.generator.subschema_for::<O::Input>();
        let output = self.generator.subschema_for::<O::Output>();
        self.operations.push(OperationInfo {
            name: O::NAME,
            kind: O::KIND,
            input,
            output,
            errors: <O::Error as OpError>::CODES,
        });
    }

    pub(crate) fn finish(mut self) -> Manifest {
        Manifest {
            version: env!("CARGO_PKG_VERSION"),
            operations: self.operations,
            defs: self.generator.take_definitions(true),
        }
    }
}

/// 可选字段缺省就是没有，线上不写 `null`。schemars 给 `Option` 字段的类型加了 `null`，这里去掉，
/// 生成的 TS 类型才是 `body?: string` 而不是 `body?: string | null`。
fn drop_optional_null(schema: &mut Schema) {
    let required: Vec<String> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|names| {
            names
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) else {
        return;
    };
    for (name, property) in properties.iter_mut() {
        if required.contains(name) {
            continue;
        }
        let Some(property) = property.as_object_mut() else {
            continue;
        };
        // 简单类型：`"type": ["string", "null"]`
        if let Some(Value::Array(types)) = property.get_mut("type") {
            types.retain(|t| t != "null");
            if let [only] = types.as_slice() {
                let only = only.clone();
                property.insert("type".to_owned(), only);
            }
        }
        // 引用别的类型：`"anyOf": [{ "$ref": … }, { "type": "null" }]`
        if let Some(Value::Array(variants)) = property.get("anyOf")
            && let [variant, null] = variants.as_slice()
            && null.get("type").and_then(Value::as_str) == Some("null")
            && let Some(variant) = variant.as_object()
        {
            let variant = variant.clone();
            property.remove("anyOf");
            property.extend(variant);
        }
    }
}
