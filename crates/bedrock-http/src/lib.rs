//! 把核心的操作面经 HTTP 开给别的设备，命令行的 `bedrock serve` 用它。
//!
//! - `POST /api/op/<名字>`：请求体是操作的输入，响应体是输出或 `{ code, message }`
//! - `GET /api/health`：服务在不在

use std::future::Future;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use bedrock_core::{Core, Ctx, ErrorBody, ErrorKind, Ports};
use serde_json::{Value, json};
use tokio::net::TcpListener;

/// 默认只监听本机。
pub const DEFAULT_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 7400);

/// 在 `listener` 上提供服务，直到 `shutdown` 完成；之后不接新请求，等进行中的做完再返回。
pub async fn serve<P: Ports>(
    listener: TcpListener,
    core: Arc<Core<P>>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    let router = router(core, listener.local_addr()?);
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await
}

pub fn router<P: Ports>(core: Arc<Core<P>>, listen: SocketAddr) -> Router {
    let router = Router::new()
        .route("/api/health", get(health))
        .route("/api/op/{name}", post(call::<P>))
        .fallback(no_such_api)
        .with_state(core);
    // 只在本机监听时，别的网站可能借 DNS 重绑定从浏览器里调进来，只认本机的主机名
    let router = if listen.ip().is_loopback() {
        router.layer(middleware::from_fn(loopback_host_only))
    } else {
        router
    };
    router.layer(middleware::map_response(security_headers))
}

/// 回应都是 JSON，不该被浏览器当成页面加载、嵌入或猜类型。
async fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    let mut set = |name, value| {
        headers.insert(name, HeaderValue::from_static(value));
    };
    set(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; frame-ancestors 'none'",
    );
    set(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    set(header::REFERRER_POLICY, "no-referrer");
    set(header::X_FRAME_OPTIONS, "DENY");
    response
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

async fn call<P: Ports>(
    State(core): State<Arc<Core<P>>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let result = match parse_input(&headers, &body) {
        Ok(input) => core.call(&Ctx::remote(), &name, input).await,
        Err(err) => Err(err),
    };
    match result {
        Ok(output) => Json(output).into_response(),
        Err(err) => error_response(err),
    }
}

/// 只收 JSON：浏览器跨站发 JSON 要先预检，没有放行的跨站头就发不出来，挡住了跨站请求伪造。
fn parse_input(headers: &HeaderMap, body: &[u8]) -> Result<Value, ErrorBody> {
    let is_json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(';')
                .next()
                .is_some_and(|t| t.trim().eq_ignore_ascii_case("application/json"))
        });
    if !is_json {
        return Err(ErrorBody::invalid_input(
            "请求头要带 Content-Type: application/json",
        ));
    }
    if body.is_empty() {
        return Ok(Value::Object(Default::default()));
    }
    serde_json::from_slice(body).map_err(ErrorBody::invalid_input)
}

fn error_response(err: ErrorBody) -> Response {
    let status = match err.kind {
        ErrorKind::Invalid => StatusCode::BAD_REQUEST,
        ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
        ErrorKind::Forbidden => StatusCode::FORBIDDEN,
        ErrorKind::NotFound => StatusCode::NOT_FOUND,
        ErrorKind::Conflict => StatusCode::CONFLICT,
        ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, Json(err)).into_response()
}

async fn no_such_api() -> Response {
    error_response(ErrorBody {
        code: "not_found",
        message: "没有这个接口".to_owned(),
        kind: ErrorKind::NotFound,
    })
}

async fn loopback_host_only(request: Request, next: Next) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .or_else(|| request.uri().host());
    if host.is_some_and(is_loopback_host) {
        next.run(request).await
    } else {
        (StatusCode::FORBIDDEN, "只接受用本机地址访问\n").into_response()
    }
}

fn is_loopback_host(host: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(rest) => rest.split_once(']').map_or(rest, |(name, _)| name),
        None => match host.rsplit_once(':') {
            Some((name, port)) if port.bytes().all(|b| b.is_ascii_digit()) => name,
            _ => host,
        },
    };
    name.eq_ignore_ascii_case("localhost")
        || name.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_loopback_hosts() {
        for host in [
            "localhost",
            "localhost:7400",
            "127.0.0.1:7400",
            "127.0.0.2",
            "[::1]:7400",
            "LOCALHOST",
        ] {
            assert!(is_loopback_host(host), "{host}");
        }
        for host in [
            "evil.example",
            "evil.example:7400",
            "localhost.evil.example",
            "10.0.0.1:7400",
            "[::2]:7400",
            "",
        ] {
            assert!(!is_loopback_host(host), "{host}");
        }
    }
}
