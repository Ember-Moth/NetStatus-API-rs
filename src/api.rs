use actix_web::{http::StatusCode, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use tracing::{error, info};
use validator::Validate;

use crate::config::ApiConfig;
use std::sync::Arc;

#[derive(Serialize)]
pub struct TcpingRes {
    status: bool,
    message: String,
}

#[derive(Validate, Deserialize)]
pub struct TcpingParams {
    #[validate(ip)]
    ip: String,
    #[validate(range(min = 1, max = 65535))]
    port: u16,
}

pub async fn tcping_v1(
    params: web::Query<TcpingParams>,
    config: web::Data<Arc<ApiConfig>>,  // 这里改成 Data<Arc<ApiConfig>>
) -> HttpResponse {
    // 参数校验
    if let Err(e) = params.validate() {
        return HttpResponse::BadRequest().json(TcpingRes {
            status: false,
            message: format!("Invalid parameters: {}", e),
        });
    }

    let tcping_timeout = config.tcping_timeout;

    let (status, message) = ping(&params.ip, params.port, tcping_timeout);
    info!(
        "TCP ping to {}:{} - Status: {}, Message: {}",
        params.ip, params.port, status, message
    );

    HttpResponse::build(if status {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    })
        .json(TcpingRes { status, message })
}

fn ping(ip: &str, port: u16, timeout_ms: u64) -> (bool, String) {
    let addr = format!("{}:{}", ip, port);
    info!("Attempting TCP connection to {}", addr);

    let socket_addr: SocketAddr = match addr.parse() {
        Ok(sa) => sa,
        Err(e) => {
            error!("Invalid socket address {}: {}", addr, e);
            return (false, format!("Invalid socket address: {}", e));
        }
    };

    match TcpStream::connect_timeout(&socket_addr, Duration::from_millis(timeout_ms)) {
        Ok(_) => {
            info!("TCP connection to {} successful", addr);
            (true, "TCP connection successful".to_string())
        }
        Err(e) => {
            error!("TCP connection to {} failed: {}", addr, e);
            (false, format!("TCP connection failed: {}", e))
        }
    }
}
