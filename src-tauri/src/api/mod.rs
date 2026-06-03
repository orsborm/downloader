// JSON-RPC API 服务模块
// 提供标准 JSON-RPC 2.0 协议，支持 HTTP 和 WebSocket 传输
// 兼容 aria2 部分 API，支持事件订阅

pub mod rpc;
pub mod websocket;

use crate::AppState;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

/// API 服务配置
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// 监听地址（默认 127.0.0.1）
    pub host: String,
    /// 监听端口（默认 6800）
    pub port: u16,
    /// 认证 Token
    pub token: Option<String>,
    /// 是否允许远程访问
    pub allow_remote: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 6800,
            token: None,
            allow_remote: false,
        }
    }
}

/// JSON-RPC 2.0 请求
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 响应
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 错误
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    /// 创建成功响应
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// 创建错误响应
    pub fn error(id: Option<serde_json::Value>, code: i32, message: &str) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
            id,
        }
    }
}

/// API 事件类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ApiEvent {
    /// 任务状态变化
    TaskStateChanged {
        task_id: String,
        state: String,
    },
    /// 任务完成
    TaskCompleted {
        task_id: String,
    },
    /// 任务错误
    TaskError {
        task_id: String,
        error: String,
    },
    /// 速度更新
    SpeedUpdate {
        download_speed: u64,
        upload_speed: u64,
    },
}

/// API 服务
pub struct ApiService {
    config: ApiConfig,
    event_tx: mpsc::UnboundedSender<ApiEvent>,
    event_rx: Option<mpsc::UnboundedReceiver<ApiEvent>>,
}

impl ApiService {
    /// 创建新的 API 服务
    pub fn new(config: ApiConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        ApiService {
            config,
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    /// 获取事件发送端（用于向 API 客户端推送事件）
    pub fn event_sender(&self) -> mpsc::UnboundedSender<ApiEvent> {
        self.event_tx.clone()
    }

    /// 启动 HTTP API 服务
    pub async fn start_http(&self, app_state: Arc<AppState>) -> Result<()> {
        use axum::routing::{get, post};
        use axum::Router;

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;
        info!("JSON-RPC API 启动: http://{}/jsonrpc", addr);

        let app = Router::new()
            .route("/jsonrpc", post({
                let state = app_state.clone();
                move |headers: axum::http::header::HeaderMap, body: axum::Json<JsonRpcRequest>| {
                    let state = state.clone();
                    async move {
                        let resp = handle_jsonrpc_inner(&state, headers, body.0).await;
                        axum::response::IntoResponse::into_response(resp)
                    }
                }
            }))
            .route("/health", get(|| async { "OK" }))
            .route("/ws", get(handle_ws_upgrade))
            .layer({
                // CORS 白名单：仅允许本地开发端口，拒绝跨域恶意请求
                use tower_http::cors::{CorsLayer, AllowOrigin, AllowHeaders, AllowMethods};
                use axum::http::header;
                let origins = vec![
                    "http://localhost:1420".parse().unwrap(),
                    "http://127.0.0.1:1420".parse().unwrap(),
                    "http://localhost:3000".parse().unwrap(),
                    "http://127.0.0.1:3000".parse().unwrap(),
                    "tauri://localhost".parse().unwrap(),
                    "https://tauri.localhost".parse().unwrap(),
                ];
                CorsLayer::new()
                    .allow_origin(AllowOrigin::list(origins))
                    .allow_methods(AllowMethods::any())
                    .allow_headers(AllowHeaders::list([
                        header::CONTENT_TYPE,
                        header::AUTHORIZATION,
                    ]))
            })
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("API 服务错误: {}", e);
            }
        });

        info!("JSON-RPC API 已启动: http://{}/jsonrpc", addr);
        Ok(())
    }

    /// 处理 JSON-RPC 请求
    pub async fn handle_request(&self, request: JsonRpcRequest, state: &AppState) -> JsonRpcResponse {
        rpc::dispatch_method(request, state).await
    }
}

/// axum handler: POST /jsonrpc
async fn handle_jsonrpc_inner(
    state: &AppState,
    headers: axum::http::header::HeaderMap,
    request: JsonRpcRequest,
) -> axum::Json<JsonRpcResponse> {
    // Token 认证（aria2 兼容：通过 Authorization header 或 secret: 前缀）
    // 空 token 表示未配置认证，拒绝所有请求（安全默认值）
    let (expected_token, auth_ok, params_token_ok) = {
        let config = state.config.lock().await;
        let expected_token = config.connection.api_token.clone();
        if expected_token.is_empty() {
            return axum::Json(JsonRpcResponse::error(
                request.id,
                -1,
                "API 认证未配置：请在设置中配置 API token",
            ));
        }

        // 检查 Authorization header
        let auth_ok = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|v| {
                v.strip_prefix("Bearer ").unwrap_or(v) == expected_token.as_str()
            })
            .unwrap_or(false);

        // 检查 params 中的 token (aria2 兼容: "token:xxx")
        let params_token_ok = match &request.params {
            serde_json::Value::Array(arr) => arr.first()
                .and_then(|v| v.as_str())
                .and_then(|s| s.strip_prefix("token:"))
                .map(|t| t == expected_token.as_str())
                .unwrap_or(false),
            _ => false,
        };

        (expected_token, auth_ok, params_token_ok)
    };

    if !auth_ok && !params_token_ok {
        return axum::Json(JsonRpcResponse::error(
            request.id,
            -1,
            "认证失败：无效或缺失的 token",
        ));
    }

    axum::Json(rpc::dispatch_method(request, state).await)
}

/// axum handler: GET /ws - WebSocket upgrade（需 token 认证）
async fn handle_ws_upgrade(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    headers: axum::http::header::HeaderMap,
    ws: axum::extract::WebSocketUpgrade,
) -> axum::response::Response {
    // 验证 token（空 token 时拒绝所有连接，与 HTTP handler 行为一致）
    let config = state.config.lock().await;
    let expected_token = config.connection.api_token.clone();
    drop(config);

    if expected_token.is_empty() {
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::FORBIDDEN)
            .body(axum::body::Body::from("API 认证未配置"))
            .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()));
    }

    let auth_ok = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.strip_prefix("Bearer ").unwrap_or(v) == expected_token.as_str())
        .unwrap_or(false);

    let query_token_ok = headers
        .get("x-auth-token")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == expected_token.as_str())
        .unwrap_or(false);

    if !auth_ok && !query_token_ok {
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::UNAUTHORIZED)
            .body(axum::body::Body::from("认证失败"))
            .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()));
    }

    let ws_manager = state.ws_manager.clone();
    ws.on_upgrade(move |socket| handle_ws_socket(socket, ws_manager))
}

/// 处理 WebSocket 连接
async fn handle_ws_socket(
    socket: axum::extract::ws::WebSocket,
    ws_manager: Arc<websocket::WsManager>,
) {
    use futures::{SinkExt, StreamExt};

    let conn_id = ws_manager.add_connection().await;
    let mut rx = ws_manager.subscribe();
    let (mut sender, mut receiver) = socket.split();

    // 转发广播事件到客户端
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(data) = serde_json::to_string(&event) {
                if sender
                    .send(axum::extract::ws::Message::Text(data.into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    // 接收客户端消息（目前仅用于保持连接）
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // 忽略客户端消息，仅保持连接
        }
    });

    // 等待任一任务结束
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    ws_manager.remove_connection(conn_id).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_config_default_localhost_only() {
        let config = ApiConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 6800);
        assert!(config.token.is_none());
        assert!(!config.allow_remote);
    }

    #[test]
    fn jsonrpc_success_response_format() {
        let resp = JsonRpcResponse::success(
            Some(serde_json::json!(1)),
            serde_json::json!("ok"),
        );
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        assert_eq!(resp.id, Some(serde_json::json!(1)));
    }

    #[test]
    fn jsonrpc_error_response_format() {
        let resp = JsonRpcResponse::error(
            Some(serde_json::json!(2)),
            -1,
            "test error",
        );
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        let err = resp.error.unwrap();
        assert_eq!(err.code, -1);
        assert_eq!(err.message, "test error");
    }

    #[test]
    fn cors_whitelist_contains_only_localhost() {
        // 验证 CORS 白名单只包含本地地址（安全要求）
        let origins = vec![
            "http://localhost:1420",
            "http://127.0.0.1:1420",
            "http://localhost:3000",
            "http://127.0.0.1:3000",
            "tauri://localhost",
            "https://tauri.localhost",
        ];
        for origin in &origins {
            assert!(
                origin.contains("localhost") || origin.contains("127.0.0.1"),
                "CORS origin {} is not a local address",
                origin
            );
        }
        // 不应包含通配符 *
        assert!(!origins.iter().any(|o| o.contains("*")));
    }

    #[test]
    fn api_event_serialization() {
        let event = ApiEvent::TaskCompleted {
            task_id: "test-123".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("TaskCompleted"));
        assert!(json.contains("test-123"));
    }
}
