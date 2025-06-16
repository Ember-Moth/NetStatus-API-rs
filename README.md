# NetStatus API (Rust)

一个使用 **Rust + Actix-Web** 编写的高性能网络状态查询 API 服务，支持 TCP Ping 等功能，带有限流、请求超时、日志记录等中间件功能。

---

## ✨ 功能特性

- 🌐 基于 Actix-Web 框架，异步高性能
- 📦 支持配置文件（TOML 格式）
- 🔌 提供 `/v1/tcping` API 接口，检测远程 TCP 端口连通性
- ⏱️ 支持接口请求超时设置
- 🚦 请求限流（基于 [`governor`](https://crates.io/crates/governor)）
- 📋 日志输出（基于 [`tracing`](https://crates.io/crates/tracing)）
- 🛡️ 参数校验（基于 [`validator`](https://crates.io/crates/validator)）

---

## 📦 依赖环境

- Rust >= 1.75
- Cargo
- 依赖库（在 `Cargo.toml` 中）:
    - actix-web
    - governor
    - tracing
    - validator
    - config
    - serde / serde_derive

---

## 🛠️ 构建方法

```bash
# 克隆项目
git clone https://github.com/your-name/NetStatus-api-rust.git
cd NetStatus-api-rust

# 构建可执行文件
cargo build --release
```

---

## 🚀 启动方式

```bash
./NetStatus-api-rust --config ./config.toml
```

或者使用默认配置文件：

```bash
./NetStatus-api-rust
```

---

## 📄 配置文件示例（`config.toml`）

```toml
port = 8080
api_timeout = 3000
tcping_timeout = 1000
rate_limit = 60
```

- `port`: 启动服务的端口号
- `api_timeout`: HTTP 请求超时时间（毫秒）
- `tcping_timeout`: TCP 连接超时时间（毫秒）
- `rate_limit`: 每分钟允许的请求次数（全局）

---

## 📡 API 示例

### 端点：`/v1/tcping`

**请求方式：** `GET`

**参数：**

| 参数名  | 类型     | 必填 | 说明       |
|------|--------|----|----------|
| ip   | string | ✅  | 目标 IP 地址 |
| port | int    | ✅  | 目标端口     |

**示例请求：**

```bash
curl "http://localhost:8080/v1/tcping?ip=8.8.8.8&port=53"
```

**响应示例：**

```json
{
  "status": true,
  "message": "TCP connection successful"
}
```

---

## 🧪 开发测试

运行服务：

```bash
cargo run -- --config ./config.toml
```

---

## 📁 项目结构

```
NetStatus-api-rust/
├── src/
│   ├── main.rs          # 主入口，服务器初始化
│   ├── api.rs           # API 路由与处理逻辑
│   └── config.rs        # 配置加载
├── config.toml          # 配置文件示例
├── Cargo.toml           # Rust 项目元数据与依赖
└── README.md
```

---

## 📃 License
GPL-3.0 License. See [License here](./LICENSE) for details.
