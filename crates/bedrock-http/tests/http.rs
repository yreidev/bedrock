//! 端到端：起服务，用 HTTP 调操作。

// 辅助函数不在 #[test] 里，clippy 的 allow-unwrap-in-tests 管不到，整个文件放开
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use reqwest::StatusCode;
use reqwest::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, HOST, X_FRAME_OPTIONS};
use serde_json::{Value, json};
use tokio::net::TcpListener;

struct Server {
    base: String,
    http: reqwest::Client,
    _data: tempfile::TempDir,
}

impl Server {
    async fn start() -> Self {
        let data = tempfile::tempdir().unwrap();
        let core = Arc::new(bedrock_local::open(data.path()).await.unwrap());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(bedrock_http::serve(listener, core, std::future::pending()));
        Self {
            base,
            http: reqwest::Client::builder().no_proxy().build().unwrap(),
            _data: data,
        }
    }

    async fn op(&self, name: &str, input: Value) -> (StatusCode, Value) {
        let response = self
            .http
            .post(format!("{}/api/op/{name}", self.base))
            .json(&input)
            .send()
            .await
            .unwrap();
        (response.status(), response.json().await.unwrap())
    }
}

#[tokio::test]
async fn creates_lists_and_deletes_notes() {
    let server = Server::start().await;
    let (status, created) = server.op("note.create", json!({ "title": "买牛奶" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(created["title"], "买牛奶");

    let (_, list) = server.op("note.list", json!({})).await;
    assert_eq!(list, json!({ "notes": [created.clone()] }));

    let (status, deleted) = server
        .op("note.delete", json!({ "id": created["id"] }))
        .await;
    assert_eq!((status, deleted), (StatusCode::OK, json!({})));
    let (status, again) = server
        .op("note.delete", json!({ "id": created["id"] }))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(again["code"], "not_found");
}

#[tokio::test]
async fn maps_error_codes_to_status_codes() {
    let server = Server::start().await;
    let (status, body) = server.op("note.create", json!({ "title": " " })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body,
        json!({ "code": "empty_title", "message": "标题不能为空" })
    );

    let (status, body) = server.op("note.rename", json!({})).await;
    assert_eq!(
        (status, body["code"].as_str()),
        (StatusCode::NOT_FOUND, Some("unknown_operation"))
    );

    let (status, body) = server
        .op("note.create", json!({ "name": "少了标题" }))
        .await;
    assert_eq!(
        (status, body["code"].as_str()),
        (StatusCode::BAD_REQUEST, Some("invalid_input"))
    );
}

#[tokio::test]
async fn accepts_only_json_requests() {
    let server = Server::start().await;
    let url = format!("{}/api/op/note.list", server.base);

    let form = server
        .http
        .post(&url)
        .body("{}")
        .header(CONTENT_TYPE, "text/plain")
        .send()
        .await
        .unwrap();
    assert_eq!(form.status(), StatusCode::BAD_REQUEST);

    let empty = server
        .http
        .post(&url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();
    assert_eq!(empty.status(), StatusCode::OK, "空的请求体当作 {{}}");

    let get = server.http.get(&url).send().await.unwrap();
    assert_eq!(get.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn answers_only_to_loopback_host_names() {
    let server = Server::start().await;
    let url = format!("{}/api/health", server.base);
    let rebound = server
        .http
        .get(&url)
        .header(HOST, "evil.example")
        .send()
        .await
        .unwrap();
    assert_eq!(rebound.status(), StatusCode::FORBIDDEN);

    let local = server
        .http
        .get(&url)
        .header(HOST, "localhost")
        .send()
        .await
        .unwrap();
    assert_eq!(local.status(), StatusCode::OK);
}

#[tokio::test]
async fn every_response_carries_security_headers() {
    let server = Server::start().await;
    for path in ["/api/health", "/api/nothing", "/"] {
        let response = server
            .http
            .get(format!("{}{path}", server.base))
            .send()
            .await
            .unwrap();
        let headers = response.headers();
        assert_eq!(
            headers[CONTENT_SECURITY_POLICY], "default-src 'none'; frame-ancestors 'none'",
            "{path}"
        );
        assert_eq!(headers[X_FRAME_OPTIONS], "DENY", "{path}");
    }
}

#[tokio::test]
async fn answers_unknown_paths_with_json() {
    let server = Server::start().await;
    for path in ["/api/nothing", "/"] {
        let missing = server
            .http
            .get(format!("{}{path}", server.base))
            .send()
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND, "{path}");
        assert_eq!(
            missing.json::<Value>().await.unwrap()["code"],
            "not_found",
            "{path}"
        );
    }
}
