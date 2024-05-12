use std::net::SocketAddr;

use axum::Router;
use hyper::Method;
use segment::Segment;
use tower_http::{cors::{Any, CorsLayer}, services::ServeDir};
use tracing::info;

use crate::{asset::index_handler, config::Config, route::routes, state::AppState};



pub async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("server listening on {}, see: http://{}", addr, addr);
    axum::serve(listener, app)
        // .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

pub async fn start(conf: Config) {

    info!("{:?}",&conf);
    
    init_app().await;

    let model_file = "./mobile_sam-tiny-vitt.safetensors";

    download_model(model_file).await;

    let device = segment::Device::new_cuda(0).unwrap();
    let model = Segment::new(model_file, device).unwrap();

    let port = conf.server_port;
    let base_path = conf.base_dir.as_str();

    let shared_state = AppState {
        config: conf.clone(),
        model,
    };


    let serve_dir = ServeDir::new(base_path);

    let app = Router::new()
        .nest("/api", routes())
        .nest_service("/data", serve_dir)
        .layer(
            CorsLayer::permissive()
        )
        .with_state(shared_state)
        .fallback(index_handler);

    serve(app, port).await;
}

async fn init_app() {
    
}

async fn download_model(file:&str) {
let url = "https://hf-mirror.com/lmz/candle-sam/resolve/main/mobile_sam-tiny-vitt.safetensors?download=true";
        if tokio::fs::metadata(file).await.is_err() {
        info!("正在下载文件中:{}", file);
        let data = reqwest::get(url).await.expect("下载失败，请重试").bytes().await.expect("下载完成，转换 bytes 失败");
        tokio::fs::write(file, data)
            .await
            .unwrap_or_else(|_| panic!("保存{}失败", file));
    }
info!("下载 模型完成")
}