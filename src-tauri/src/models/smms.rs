use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SmmsTokenResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<SmmsTokenData>,
}

#[derive(Serialize, Deserialize)]
pub struct SmmsTokenData {
    pub token: String,
}

/// SM.MS 用户信息（用于返回给前端）
#[derive(Serialize, Deserialize)]
pub struct SmmsUser {
    pub username: String,
    pub password: String,
    pub token: String,
}

/// SM.MS 上传历史单条记录
#[derive(Serialize, Deserialize, Debug)]
pub struct SmmsUploadItem {
    #[serde(default)]
    pub file_id: i64,
    pub width: i64,
    pub height: i64,
    pub filename: String,
    #[serde(rename = "storename")]
    pub store_name: String,
    pub size: i64,
    pub path: String,
    pub hash: String,
    pub created_at: String,
    pub url: String,
    #[serde(rename = "delete")]
    pub delete_url: String,
    #[serde(rename = "page")]
    pub page_url: String,
}

/// SM.MS 上传历史响应
#[derive(Deserialize, Debug)]
pub struct SmmsUploadHistoryResponse {
    pub success: bool,
    #[serde(default)]
    pub _code: String,
    #[serde(default)]
    pub message: String,
    pub data: Option<Vec<SmmsUploadItem>>,
    #[serde(rename = "RequestId")]
    #[serde(default)]
    pub _request_id: Option<String>,
}

/// 本地数据库图片记录
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmmsPicture {
    pub id: i64,
    pub file_hash: String,
    pub filename: String,
    pub store_name: String,
    pub file_type: String,
    pub width: i64,
    pub height: i64,
    pub size: i64,
    pub path: String,
    pub url: String,
    pub delete_url: String,
    pub page_url: String,
    pub is_favorite: i32,
    pub is_deleted: i32,
    pub deleted_at: Option<String>,
    pub remark: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 图片查询参数
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PictureQueryParams {
    pub file_type: Option<String>,
    pub is_favorite: Option<bool>,
    pub include_deleted: Option<bool>,
    pub order_by: Option<String>,
    pub filename: Option<String>,
    pub store_name: Option<String>,
    pub remark: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// SM.MS 上传响应数据
#[derive(Serialize, Deserialize, Debug)]
pub struct SmmsUploadData {
    #[serde(default)]
    pub file_id: i64,
    pub width: i64,
    pub height: i64,
    pub filename: String,
    #[serde(rename = "storename")]
    pub store_name: String,
    pub size: i64,
    pub path: String,
    pub hash: String,
    pub url: String,
    #[serde(rename = "delete")]
    pub delete_url: String,
    #[serde(rename = "page")]
    pub page_url: String,
}

/// SM.MS 上传响应
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SmmsUploadResponse {
    pub success: bool,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub message: String,
    pub data: Option<SmmsUploadData>,
    #[serde(rename = "RequestId")]
    #[serde(default)]
    pub request_id: Option<String>,
}

/// 上传结果（返回给前端）
#[derive(Serialize, Debug)]
pub struct UploadResult {
    pub filename: String,
    pub success: bool,
    pub message: String,
    pub url: Option<String>,
    pub remark: Option<String>,
}

/// SM.MS 删除响应
#[derive(Deserialize, Debug)]
pub struct SmmsDeleteResponse {
    pub success: bool,
    #[serde(default)]
    pub _code: String,
    #[serde(default)]
    pub message: String,
    #[serde(rename = "RequestId")]
    #[serde(default)]
    pub _request_id: Option<String>,
}

/// 同步相册图片统计结果
#[derive(Serialize, Debug)]
pub struct SyncStats {
    pub added: usize,
    pub skipped: usize,
    pub deleted: usize,
}

/// 批量删除结果
#[derive(Serialize, Debug)]
pub struct BatchDeleteResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub failed_items: Vec<String>,
}
