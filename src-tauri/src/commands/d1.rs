use std::fs;

use crate::models::{D1Config, D1Response};
use crate::services::config::{
    clear_d1_config_cache, get_config_path, update_d1_config_cache, D1_CONFIG_CACHE,
};

/// 保存 D1 配置
#[tauri::command]
pub async fn save_d1_config(config: D1Config) -> Result<String, String> {
    let config_path = get_config_path()?;
    let json =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(&config_path, json).map_err(|e| format!("写入配置文件失败: {}", e))?;

    update_d1_config_cache(config);

    Ok("配置保存成功".to_string())
}

/// 读取 D1 配置
#[tauri::command]
pub async fn load_d1_config() -> Result<D1Config, String> {
    if let Ok(cache) = D1_CONFIG_CACHE.read() {
        if let Some(config) = cache.as_ref() {
            return Ok(config.clone());
        }
    }

    let config_path = get_config_path()?;
    if !config_path.exists() {
        return Err("配置文件不存在".to_string());
    }

    let json = fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;
    let config: D1Config =
        serde_json::from_str(&json).map_err(|e| format!("解析配置文件失败: {}", e))?;

    update_d1_config_cache(config.clone());

    Ok(config)
}

/// 删除 D1 配置
#[tauri::command]
pub async fn delete_d1_config() -> Result<String, String> {
    let config_path = get_config_path()?;
    if config_path.exists() {
        fs::remove_file(&config_path).map_err(|e| format!("删除配置文件失败: {}", e))?;
    }

    clear_d1_config_cache();

    Ok("配置已完全删除".to_string())
}

/// 测试 D1 连接
#[tauri::command]
pub async fn test_d1_connection(config: D1Config) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/query",
        config.account_id, config.database_id
    );

    let body = serde_json::json!({ "sql": "SELECT 1 as test" });

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    let result: D1Response = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.success {
        Ok("连接成功".to_string())
    } else {
        let error_msg = result
            .errors
            .iter()
            .map(|e| format!("[{}] {}", e.code, e.message))
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!("连接失败: {}", error_msg))
    }
}

/// 执行 D1 SQL 查询
#[tauri::command]
pub async fn execute_d1_query(sql: String) -> Result<Vec<serde_json::Value>, String> {
    let config = load_d1_config().await?;
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/query",
        config.account_id, config.database_id
    );

    let body = serde_json::json!({ "sql": sql });

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let result: D1Response = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.success {
        if let Some(query_results) = result.result {
            if let Some(first_result) = query_results.first() {
                return Ok(first_result.results.clone());
            }
        }
        Ok(vec![])
    } else {
        let error_msg = result
            .errors
            .iter()
            .map(|e| format!("[{}] {}", e.code, e.message))
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!("查询失败: {}", error_msg))
    }
}

/// 批量执行 D1 SQL 语句（性能优化）
pub async fn execute_d1_batch(sqls: Vec<String>) -> Result<(), String> {
    if sqls.is_empty() {
        return Ok(());
    }

    let config = load_d1_config().await?;
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/query",
        config.account_id, config.database_id
    );

    // 使用分号连接的批量SQL（D1会自动作为事务执行）
    let batch_sql = sqls.join("; ");

    let body = serde_json::json!({ "sql": batch_sql });

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("批量请求失败: {}", e))?;

    let result: D1Response = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.success {
        Ok(())
    } else {
        let error_msg = result
            .errors
            .iter()
            .map(|e| format!("[{}] {}", e.code, e.message))
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!("批量执行失败: {}", error_msg))
    }
}
