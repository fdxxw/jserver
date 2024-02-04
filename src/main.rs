use std::iter::Map;

use clap::{builder::Str, Parser};
use salvo::prelude::*;
use tracing_subscriber::{
    fmt::{self, time::FormatTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = ("127.0.0.1:5800".to_string()))]
    bind: String,
}

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

#[handler]
async fn json(req: &mut Request) -> &'static str {
    let body = req.parse_json::<serde_json::Value>().await.unwrap();
    tracing::info!("path = {}, json = {}", req.uri(), body);
    ""
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 输出到控制台中
    let formatting_layer = fmt::layer()
        .pretty()
        .with_writer(std::io::stdout)
        .with_timer(LocalTimer);

    // 输出到文件中
    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_line_number(true)
        .with_timer(LocalTimer)
        .with_writer(non_blocking_appender);
    // 注册
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        // ErrorLayer 可以让 color-eyre 获取到 span 的信息
        .with(tracing_error::ErrorLayer::default())
        .with(formatting_layer)
        .with(file_layer)
        .init();

    let router = Router::new()
        .push(Router::with_path("/ping").get(hello))
        .push(Router::with_path("/json/<**+*rest_path>").post(json));
    let acceptor = TcpListener::new(args.bind).bind().await;
    Server::new(acceptor).serve(router).await;
}

struct LocalTimer;
impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("%FT%T%.3f"))
    }
}
