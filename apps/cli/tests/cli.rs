//! 端到端：跑编出来的 `bedrock`，检查输出和退出码。

// 辅助函数不在 #[test] 里，clippy 的 allow-unwrap-in-tests 管不到，整个文件放开
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

struct Bedrock {
    data: tempfile::TempDir,
}

impl Bedrock {
    fn new() -> Self {
        Self {
            data: tempfile::tempdir().unwrap(),
        }
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_bedrock"));
        command
            .args(args)
            .env("BEDROCK_DATA", self.data.path())
            .env("TZ", "UTC");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    fn json(&self, args: &[&str]) -> Value {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn creates_lists_and_deletes_notes() {
    let bedrock = Bedrock::new();
    let created = bedrock.run(&["note", "create", " 买牛奶 ", "--body", "低脂的\n两盒"]);
    assert_eq!(created.status.code(), Some(0));
    assert!(
        stdout(&created).starts_with("已新建：买牛奶（编号 "),
        "{}",
        stdout(&created)
    );

    let list = bedrock.run(&["note", "list"]);
    let text = stdout(&list);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert!(lines[0].contains("  买牛奶  "), "{lines:?}");
    assert_eq!(&lines[1..], ["    低脂的", "    两盒"]);

    let notes = bedrock.json(&["--json", "note", "list"]);
    let id = notes["notes"][0]["id"].as_str().unwrap().to_owned();
    assert_eq!(notes["notes"][0]["title"], "买牛奶");

    let deleted = bedrock.run(&["note", "delete", &id]);
    assert_eq!(
        (deleted.status.code(), stdout(&deleted)),
        (Some(0), "已删除\n".to_owned())
    );
    assert_eq!(stdout(&bedrock.run(&["note", "list"])), "还没有备忘\n");

    let again = bedrock.run(&["note", "delete", &id]);
    assert_eq!(again.status.code(), Some(1));
    assert_eq!(
        stderr(&again),
        "bedrock: 这条备忘不存在，可能已经删掉了（not_found）\n"
    );
}

#[test]
fn reports_operation_errors_with_exit_code_1() {
    let bedrock = Bedrock::new();
    let empty = bedrock.run(&["note", "create", "  "]);
    assert_eq!(empty.status.code(), Some(1));
    assert_eq!(stderr(&empty), "bedrock: 标题不能为空（empty_title）\n");

    let json = bedrock.run(&["--json", "note", "create", "  "]);
    assert_eq!(json.status.code(), Some(1));
    let body: Value = serde_json::from_slice(&json.stderr).unwrap();
    assert_eq!(
        body,
        json!({ "code": "empty_title", "message": "标题不能为空" })
    );
}

#[test]
fn reports_usage_errors_with_exit_code_2() {
    let output = Bedrock::new().run(&["note", "rename"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn calls_any_operation_by_name() {
    let bedrock = Bedrock::new();
    let created = bedrock.json(&["op", "note.create", r#"{"title":"交房租"}"#]);
    assert_eq!(created["title"], "交房租");

    let mut child = bedrock
        .command(&["op", "note.list", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"{}").unwrap();
    let output = child.wait_with_output().unwrap();
    let list: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(list, json!({ "notes": [created] }));

    let bad = bedrock.run(&["op", "note.create", "不是 JSON"]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(
        stderr(&bad).contains("（invalid_input）"),
        "{}",
        stderr(&bad)
    );

    let unknown = bedrock.run(&["op", "note.rename", "{}"]);
    assert!(
        stderr(&unknown).contains("（unknown_operation）"),
        "{}",
        stderr(&unknown)
    );
}

#[test]
fn lists_operations() {
    let output = Bedrock::new().run(&["ops"]);
    assert_eq!(
        stdout(&output),
        "note.create  动作\nnote.list  查询\nnote.delete  动作\n"
    );
}

/// 不经代理，直接发一个 HTTP 请求，返回状态码和 JSON 回应。
fn post(addr: &str, path: &str, body: &str) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).unwrap();
    write!(
        stream,
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let (head, body) = response.split_once("\r\n\r\n").unwrap();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    (status, serde_json::from_str(body).unwrap())
}

#[cfg(unix)]
#[test]
fn serves_operations_over_http_until_terminated() {
    let bedrock = Bedrock::new();
    let mut child = bedrock
        .command(&["serve", "--listen", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut ready = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut ready)
        .unwrap();
    let addr = ready
        .strip_prefix("在 http://")
        .and_then(|rest| rest.strip_suffix(" 上提供服务\n"))
        .unwrap_or_else(|| panic!("{ready:?}"))
        .to_owned();

    let (status, created) = post(&addr, "/api/op/note.create", r#"{"title":"买牛奶"}"#);
    assert_eq!((status, &created["title"]), (200, &json!("买牛奶")));
    let (status, error) = post(&addr, "/api/op/note.create", r#"{"title":" "}"#);
    assert_eq!((status, &error["code"]), (400, &json!("empty_title")));

    let killed = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stderr(&output).contains("已退出"), "{}", stderr(&output));

    // 服务和命令行用同一个数据目录
    let notes = bedrock.json(&["--json", "note", "list"]);
    assert_eq!(notes, json!({ "notes": [created] }));
}
