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
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let config_path = args
        .iter()
        .position(|arg| arg == "--config")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| String::from("default_config.toml"));

    let settings = match config::load_config(&config_path) {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            Arc::new(cfg)  // Arc<ApiConfig>
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
        }
    };

    let listen_addr = settings.listen.clone();
    let api_timeout = settings.api_timeout;
    let rate_limit = settings.rate_limit;

    let non_zero_rate_limit = NonZeroU32::new(rate_limit).unwrap_or_else(|| NonZeroU32::new(60).unwrap());
    let limiter: Arc<Mutex<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>> =
        Arc::new(Mutex::new(RateLimiter::direct(Quota::per_minute(non_zero_rate_limit))));

    let server = HttpServer::new(move || {
        let limiter = limiter.clone();
        let settings = settings.clone();

        App::new()
            .app_data(web::Data::new(settings.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new().add(("Content-Type", "application/json")))
            .wrap(middleware::Compress::default())
            .wrap_fn(move |req, srv| {
                let limiter = limiter.clone();
                let fut = srv.call(req);

                async move {
                    let allow = {
                        let limiter = limiter.lock().unwrap();
                        limiter.check().is_ok()
                    };

                    if !allow {
                        return Err(actix_web::error::ErrorTooManyRequests("Rate limit exceeded"));
                    }

                    timeout(Duration::from_millis(api_timeout), fut)
                        .await
                        .unwrap_or_else(|_| Err(actix_web::error::ErrorRequestTimeout("Request timed out")))
                }
            })
            .service(web::scope("/v1").route("/tcping", web::get().to(api::tcping_v1)))
    });

    let server = server.bind(&listen_addr)?.run();
    let server_handle = server.handle();

    // Unix 平台信号处理
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

    // Windows 平台信号处理
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

    info!("Server running on {}", listen_addr);
    server.await
}
