use std::env::args;

use server::{config::Config, server::start};
use tracing::Level;
use tracing_subscriber::fmt::format;

#[tokio::main]
async fn main() {
    let mut conf = envy::from_env::<Config>().expect("环境配置解析错误");
    let args: Vec<String> = args().collect();
    if args.len() >= 2 {
        conf.base_dir = args.get(1).unwrap().to_string();
    }

    

    tracing_subscriber::fmt()
        .event_format(format().compact())
        .with_max_level(Level::INFO)
        .init();
    tokio::join!(start(conf));
}
