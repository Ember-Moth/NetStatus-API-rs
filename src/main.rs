mod config;
mod api;

use actix_web::{web, App, HttpServer, middleware};
use governor::{Quota, RateLimiter};
use governor::state::InMemoryState;
use governor::clock::DefaultClock;
use governor::state::NotKeyed;
use std::sync::{Arc, Mutex};
use std::num::NonZeroU32;
use std::time::Duration;
use tracing::{info, error};
use tokio::time::timeout;
use actix_web::dev::Service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // 解析配置路径参数
    let args: Vec<String> = std::env::args().collect();
    let config_path = args
        .iter()
        .position(|arg| arg == "--config")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| String::from("default_config.toml"));

    // 加载配置
    let settings = match config::load_config(&config_path) {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            Arc::new(cfg)  // 注意这里是 Arc<ApiConfig>
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
        }
    };

    let port = settings.port;
    let api_timeout = settings.api_timeout;
    let rate_limit = settings.rate_limit;

    // 构建限速器（不使用键）
    let non_zero_rate_limit = NonZeroU32::new(rate_limit).unwrap_or_else(|| NonZeroU32::new(60).unwrap());
    let limiter: Arc<Mutex<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>> =
        Arc::new(Mutex::new(RateLimiter::direct(Quota::per_minute(non_zero_rate_limit))));

    // 构建 HTTP Server
    let server = HttpServer::new(move || {
        let limiter = limiter.clone();
        let settings = settings.clone();

        App::new()
            .app_data(web::Data::new(settings.clone()))  // 传入 Data<Arc<ApiConfig>>
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new().add(("Content-Type", "application/json")))
            .wrap(middleware::Compress::default())
            .wrap_fn(move |req, srv| {
                let limiter = limiter.clone();
                let fut = srv.call(req);

                async move {
                    // 限速检查
                    let allow = {
                        let limiter = limiter.lock().unwrap();
                        limiter.check().is_ok()
                    };

                    if !allow {
                        return Err(actix_web::error::ErrorTooManyRequests("Rate limit exceeded"));
                    }

                    // 包裹请求处理为超时
                    timeout(Duration::from_millis(api_timeout), fut)
                        .await
                        .unwrap_or_else(|_| Err(actix_web::error::ErrorRequestTimeout("Request timed out")))
                }
            })
            .service(web::scope("/v1").route("/tcping", web::get().to(api::tcping_v1)))
    });

    let server = server.bind(("0.0.0.0", port))?.run();
    let server_handle = server.handle();

    // 信号处理（Unix）
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let server_handle = server_handle.clone();
        tokio::spawn(async move {
            let mut sigint = signal(SignalKind::interrupt()).unwrap();
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            tokio::select! {
                _ = sigint.recv() => info!("Received SIGINT, shutting down"),
                _ = sigterm.recv() => info!("Received SIGTERM, shutting down"),
            }
            server_handle.stop(true).await;
        });
    }

    // 信号处理（Windows）
    #[cfg(windows)]
    {
        let server_handle = server_handle.clone();
        tokio::spawn(async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                error!("Ctrl+C signal error: {}", e);
                return;
            }
            info!("Received Ctrl+C, shutting down");
            server_handle.stop(true).await;
        });
    }

    info!("Server running on port {}", port);
    server.await
}
