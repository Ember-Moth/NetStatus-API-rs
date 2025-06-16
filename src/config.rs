use config::{Config, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct ApiConfig {
    pub port: u16,
    pub api_timeout: u64,      // 单位：毫秒
    pub tcping_timeout: u64,   // 单位：毫秒
    pub rate_limit: u32,
}

pub fn load_config(config_path: &str) -> Result<ApiConfig, String> {
    let mut builder = Config::builder()
        .set_default("port", 8080)
        .map_err(|e| format!("Failed to set default port: {}", e))?
        .set_default("api_timeout", 3000)
        .map_err(|e| format!("Failed to set default api_timeout: {}", e))?
        .set_default("tcping_timeout", 1000)
        .map_err(|e| format!("Failed to set default tcping_timeout: {}", e))?
        .set_default("rate_limit", 60)
        .map_err(|e| format!("Failed to set default rate_limit: {}", e))?;

    if !config_path.is_empty() && Path::new(config_path).exists() {
        builder = builder.add_source(File::with_name(config_path));
    } else {
        builder = builder
            .add_source(File::with_name("config").required(false))
            .add_source(File::with_name("/etc/netstatus-api/config").required(false));
    }

    let config = builder
        .build()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;
    config
        .try_deserialize::<ApiConfig>()
        .map_err(|e| format!("Failed to deserialize configuration: {}", e))
}
