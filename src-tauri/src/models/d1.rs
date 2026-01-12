use serde::{Deserialize, Serialize};

/// Cloudflare D1 完整配置结构（用于前端交互）
#[derive(Serialize, Deserialize, Clone)]
pub struct D1Config {
    pub account_id: String,
    pub database_id: String,
    pub api_token: String,
}

/// D1 API 响应结构
#[derive(Deserialize)]
pub struct D1Response {
    pub success: bool,
    pub errors: Vec<D1Error>,
    #[allow(dead_code)]
    pub messages: Vec<String>,
    pub result: Option<Vec<D1QueryResult>>,
}

#[derive(Deserialize)]
pub struct D1Error {
    pub code: i32,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct D1QueryResult {
    pub results: Vec<serde_json::Value>,
    pub success: bool,
    pub meta: Option<serde_json::Value>,
}
