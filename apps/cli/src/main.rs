//! bedrock 的命令行：在本机直接用核心，`serve` 把操作面开给别的设备。

use std::io::{self, Read};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Context;
use bedrock_core::note::{
    CreateNote, CreateNoteInput, DeleteNote, DeleteNoteInput, ListNotes, ListNotesInput, NoteView,
};
use bedrock_core::{Core, Ctx, ErrorBody, Kind, Operation, Ports};
use bedrock_local::LocalPorts;
use clap::{Parser, Subcommand};
use jiff::tz::TimeZone;
use serde_json::Value;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "bedrock",
    version,
    about = "bedrock 的命令行：管理备忘，也能开服务给别的设备用"
)]
struct Cli {
    /// 数据目录，默认在系统的应用数据目录下
    #[arg(long, global = true, env = "BEDROCK_DATA", value_name = "目录")]
    data: Option<PathBuf>,
    /// 输出操作的原始结果（JSON），方便脚本处理
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 备忘
    #[command(subcommand)]
    Note(NoteCommand),
    /// 按名字直接调一个操作，输出 JSON
    Op {
        /// 操作名，比如 note.create
        #[arg(value_name = "名字")]
        name: String,
        /// 输入的 JSON，写 - 时从标准输入读；不写就是 {}
        #[arg(value_name = "输入")]
        input: Option<String>,
    },
    /// 列出所有操作；加 --json 输出完整的操作清单
    Ops,
    /// 开 HTTP 服务，让别的设备按名字调操作，Ctrl+C 退出
    Serve {
        /// 监听的地址和端口，默认只监听本机
        #[arg(long, env = "BEDROCK_LISTEN", value_name = "地址:端口", default_value_t = bedrock_http::DEFAULT_ADDR)]
        listen: SocketAddr,
    },
}

#[derive(Subcommand)]
enum NoteCommand {
    /// 新建一条备忘
    Create {
        #[arg(value_name = "标题")]
        title: String,
        /// 正文
        #[arg(long, value_name = "正文")]
        body: Option<String>,
    },
    /// 列出所有备忘，新建的在前
    List,
    /// 删掉一条备忘
    Delete {
        #[arg(value_name = "编号")]
        id: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let serving = matches!(cli.command, Command::Serve { .. });

    // 日志打到标准错误：平时只有意料之外的错误，开服务时多记启停
    let level = if serving { "info" } else { "warn" };
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            EnvFilter::try_from_env("BEDROCK_LOG").unwrap_or_else(|_| EnvFilter::new(level)),
        )
        .init();

    // 服务要同时接多个请求，用多线程；别的命令做完一件事就退出，单线程够了
    let mut runtime = if serving {
        tokio::runtime::Builder::new_multi_thread()
    } else {
        tokio::runtime::Builder::new_current_thread()
    };
    let runtime = match runtime.enable_all().build() {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("bedrock: 起不来异步运行时：{err}");
            return ExitCode::FAILURE;
        }
    };

    let json = cli.json;
    match runtime.block_on(run(cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            match err.downcast_ref::<ErrorBody>() {
                Some(body) if json => eprintln!("{}", pretty(body)),
                _ => eprintln!("bedrock: {err:#}"),
            }
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Ops => print_ops(cli.json),
        Command::Note(command) => note(&open(cli.data).await?, command, cli.json).await?,
        Command::Op { name, input } => {
            let input = read_input(input)?;
            let output = open(cli.data)
                .await?
                .call(&Ctx::local(), &name, input)
                .await?;
            // 这一层是给脚本用的，固定输出 JSON
            println!("{}", pretty(&output));
        }
        Command::Serve { listen } => serve(open(cli.data).await?, listen).await?,
    }
    Ok(())
}

async fn open(data: Option<PathBuf>) -> anyhow::Result<Core<LocalPorts>> {
    let data = match data {
        Some(data) => data,
        None => bedrock_local::default_data_dir()
            .context("找不到应用数据目录，请用 --data 或 BEDROCK_DATA 指定")?,
    };
    Ok(bedrock_local::open(&data).await?)
}

async fn note(core: &Core<LocalPorts>, command: NoteCommand, json: bool) -> anyhow::Result<()> {
    match command {
        NoteCommand::Create { title, body } => {
            let note = op::<CreateNote, _>(core, CreateNoteInput { title, body }).await?;
            if json {
                println!("{}", pretty(&note));
            } else {
                println!("已新建：{}（编号 {}）", note.title, note.id);
            }
        }
        NoteCommand::List => {
            let list = op::<ListNotes, _>(core, ListNotesInput {}).await?;
            if json {
                println!("{}", pretty(&list));
            } else if list.notes.is_empty() {
                println!("还没有备忘");
            } else {
                for note in &list.notes {
                    print_note(note);
                }
            }
        }
        NoteCommand::Delete { id } => {
            let output = op::<DeleteNote, _>(core, DeleteNoteInput { id }).await?;
            if json {
                println!("{}", pretty(&output));
            } else {
                println!("已删除");
            }
        }
    }
    Ok(())
}

async fn op<O: Operation, P: Ports>(
    core: &Core<P>,
    input: O::Input,
) -> Result<O::Output, ErrorBody> {
    core.run::<O>(&Ctx::local(), input)
        .await
        .map_err(|err| ErrorBody::from_op(O::NAME, &err))
}

async fn serve(core: Core<LocalPorts>, listen: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("监听不了 {listen}"))?;
    if !listen.ip().is_loopback() {
        tracing::warn!("监听的不是本机地址，还没有认证，同一网络里谁都能调");
    }
    // 这一行给人和脚本看服务开在哪；端口写 0 时由系统分配
    println!("在 http://{} 上提供服务", listener.local_addr()?);
    bedrock_http::serve(listener, Arc::new(core), shutdown_signal()).await?;
    tracing::info!("已退出");
    Ok(())
}

/// Ctrl+C 或 SIGTERM。
async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
    tracing::info!("收到退出信号，等进行中的请求做完");
}

fn print_note(note: &NoteView) {
    let time = note
        .created_at
        .to_zoned(TimeZone::system())
        .strftime("%Y-%m-%d %H:%M");
    println!("{time}  {}  {}", note.title, note.id);
    if let Some(body) = &note.body {
        for line in body.lines() {
            println!("    {line}");
        }
    }
}

fn print_ops(json: bool) {
    let manifest = bedrock_core::manifest();
    if json {
        println!("{}", pretty(&manifest));
        return;
    }
    for op in &manifest.operations {
        let kind = match op.kind {
            Kind::Query => "查询",
            Kind::Action => "动作",
        };
        println!("{}  {kind}", op.name);
    }
}

fn read_input(input: Option<String>) -> anyhow::Result<Value> {
    let text = match input.as_deref() {
        None => return Ok(Value::Object(Default::default())),
        Some("-") => {
            let mut text = String::new();
            io::stdin()
                .read_to_string(&mut text)
                .context("读不了标准输入")?;
            text
        }
        Some(text) => text.to_owned(),
    };
    Ok(serde_json::from_str(&text).map_err(ErrorBody::invalid_input)?)
}

fn pretty(value: &impl serde::Serialize) -> String {
    // 这些类型都是普通的结构体和字符串，序列化不会失败
    serde_json::to_string_pretty(value).unwrap_or_default()
}
