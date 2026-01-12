use crate::commands::d1::{execute_d1_batch, execute_d1_query};
use crate::models::{
    PictureQueryParams, SmmsPicture, SmmsTokenResponse, SmmsUploadHistoryResponse, SmmsUploadItem,
    SmmsUser, SyncStats,
};
use crate::services::crypto::{decrypt_password, encrypt_password};
use std::collections::HashSet;

/// 获取 SM.MS Token
#[tauri::command]
pub async fn get_smms_token(username: String, password: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = "https://sm.ms/api/v2/token";

    let params = [
        ("username", username.as_str()),
        ("password", password.as_str()),
    ];

    let response = client
        .post(url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let result: SmmsTokenResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if result.success {
        result
            .data
            .map(|d| d.token)
            .ok_or("响应中没有 token 数据".to_string())
    } else {
        Err(format!("获取 token 失败: {}", result.message))
    }
}

/// 保存 SM.MS 凭证到 D1 数据库
#[tauri::command]
pub async fn save_smms_user(
    username: String,
    password: String,
    token: String,
) -> Result<String, String> {
    println!("=== save_smms_user 调试信息 ===");
    println!("username: {}", username);
    println!("password length: {}", password.len());
    println!("token length: {}", token.len());

    // 加载 D1 配置（用于派生加密密钥）
    let d1_config = crate::commands::d1::load_d1_config().await?;

    // 确保表存在
    let create_table_sql = "CREATE TABLE IF NOT EXISTS smms_user (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL UNIQUE,
        encrypted_password TEXT NOT NULL,
        encrypted_api_token TEXT,
        created_at DATETIME DEFAULT (datetime('now')),
        updated_at DATETIME DEFAULT (datetime('now'))
    )";

    execute_d1_query(create_table_sql.to_string()).await?;

    // 加密密码和 token（使用 D1 配置派生密钥）
    let encrypted_password =
        encrypt_password(&password, &d1_config.account_id, &d1_config.database_id)?;
    let encrypted_token = encrypt_password(&token, &d1_config.account_id, &d1_config.database_id)?;

    // UPSERT SQL
    let sql = format!(
        "INSERT INTO smms_user (username, encrypted_password, encrypted_api_token, updated_at) \
         VALUES ('{}', '{}', '{}', datetime('now')) \
         ON CONFLICT(username) DO UPDATE SET \
         encrypted_password = excluded.encrypted_password, \
         encrypted_api_token = excluded.encrypted_api_token, \
         updated_at = excluded.updated_at",
        username.replace("'", "''"),
        encrypted_password.replace("'", "''"),
        encrypted_token.replace("'", "''")
    );

    execute_d1_query(sql).await?;

    Ok("SM.MS 凭证已安全保存到数据库".to_string())
}

/// 从 D1 数据库加载 SM.MS 凭证
#[tauri::command]
pub async fn load_smms_user(username: Option<String>) -> Result<SmmsUser, String> {
    // 加载 D1 配置（用于派生解密密钥）
    let d1_config = crate::commands::d1::load_d1_config().await?;

    let sql = if let Some(user) = username.filter(|u| !u.trim().is_empty()) {
        format!(
            "SELECT username, encrypted_password, encrypted_api_token \
             FROM smms_user WHERE username = '{}' LIMIT 1",
            user.replace("'", "''")
        )
    } else {
        "SELECT username, encrypted_password, encrypted_api_token \
         FROM smms_user ORDER BY id ASC LIMIT 1"
            .to_string()
    };

    let results = execute_d1_query(sql).await?;

    if results.is_empty() {
        return Err("未找到 SM.MS 凭证".to_string());
    }

    let record = &results[0];
    let username = record
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or("用户名字段缺失")?
        .to_string();

    let encrypted_password = record
        .get("encrypted_password")
        .and_then(|v| v.as_str())
        .ok_or("密码字段缺失")?
        .to_string();

    let encrypted_token = record
        .get("encrypted_api_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // 解密密码和 token（使用 D1 配置派生密钥）
    let password = decrypt_password(
        &encrypted_password,
        &d1_config.account_id,
        &d1_config.database_id,
    )?;

    let token = if encrypted_token.is_empty() {
        String::new()
    } else {
        decrypt_password(
            &encrypted_token,
            &d1_config.account_id,
            &d1_config.database_id,
        )?
    };

    Ok(SmmsUser {
        username,
        password,
        token,
    })
}

/// 获取 SM.MS 上传历史
#[tauri::command]
pub async fn get_smms_upload_history(page: Option<i32>) -> Result<Vec<SmmsUploadItem>, String> {
    let user = load_smms_user(None).await?;

    if user.token.is_empty() {
        return Err("请先登录 SM.MS 获取 token".to_string());
    }

    let client = reqwest::Client::new();
    let url = "https://sm.ms/api/v2/upload_history";
    let page_num = page.unwrap_or(1);

    let response = client
        .get(url)
        .header("Authorization", user.token)
        .query(&[("page", page_num.to_string())])
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    // 获取响应文本
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    // 解析 JSON
    let result: SmmsUploadHistoryResponse =
        serde_json::from_str(&response_text).map_err(|e| format!("解析响应失败: {}", e))?;

    if result.success {
        Ok(result.data.unwrap_or_default())
    } else {
        Err(format!("获取上传历史失败: {}", result.message))
    }
}

/// 初始化 smms_pictures 表和索引
#[tauri::command]
pub async fn init_smms_pictures_table() -> Result<String, String> {
    // 直接创建表（如果不存在）
    let create_table_sql = "CREATE TABLE IF NOT EXISTS smms_pictures (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        file_hash TEXT NOT NULL UNIQUE,
        filename TEXT NOT NULL,
        store_name TEXT NOT NULL,
        file_type TEXT NOT NULL,
        width INTEGER NOT NULL,
        height INTEGER NOT NULL,
        size INTEGER NOT NULL,
        path TEXT NOT NULL,
        url TEXT NOT NULL,
        delete_url TEXT NOT NULL,
        page_url TEXT NOT NULL,
        is_favorite INTEGER DEFAULT 0,
        is_deleted INTEGER DEFAULT 0,
        deleted_at DATETIME,
        remark TEXT,
        created_at DATETIME NOT NULL,
        updated_at DATETIME DEFAULT (datetime('now'))
    )";

    execute_d1_query(create_table_sql.to_string()).await?;

    // 为已存在的表添加新字段（兼容旧数据）
    let alter_sqls = vec![
        "ALTER TABLE smms_pictures ADD COLUMN is_deleted INTEGER DEFAULT 0".to_string(),
        "ALTER TABLE smms_pictures ADD COLUMN deleted_at DATETIME".to_string(),
        "ALTER TABLE smms_pictures ADD COLUMN remark TEXT".to_string(),
    ];

    for sql in alter_sqls {
        let _ = execute_d1_query(sql).await; // 忽略字段已存在的错误
    }

    // 批量创建索引
    let indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_smms_pictures_hash ON smms_pictures(file_hash)".to_string(),
        "CREATE INDEX IF NOT EXISTS idx_smms_pictures_created_at ON smms_pictures(created_at DESC)".to_string(),
        "CREATE INDEX IF NOT EXISTS idx_smms_pictures_updated_at ON smms_pictures(updated_at DESC)".to_string(),
        "CREATE INDEX IF NOT EXISTS idx_smms_pictures_favorite ON smms_pictures(is_favorite) WHERE is_favorite = 1".to_string(),
        "CREATE INDEX IF NOT EXISTS idx_smms_pictures_type ON smms_pictures(file_type)".to_string(),
        "CREATE INDEX IF NOT EXISTS idx_smms_pictures_type_created ON smms_pictures(file_type, created_at DESC)".to_string(),
        "CREATE INDEX IF NOT EXISTS idx_smms_pictures_deleted ON smms_pictures(is_deleted)".to_string(),
    ];

    execute_d1_batch(indexes).await?;

    Ok("smms_pictures 表和索引初始化成功".to_string())
}

/// 同步上传历史到本地数据库
#[tauri::command]
pub async fn sync_smms_pictures(page: Option<i32>) -> Result<String, String> {
    // 确保表存在
    init_smms_pictures_table().await?;

    // 获取上传历史
    let items = get_smms_upload_history(page).await?;

    if items.is_empty() {
        return Ok("没有新的图片需要同步".to_string());
    }

    // 批量收集SQL语句
    let batch_sqls: Vec<String> = items
        .iter()
        .map(|item| {
            let file_type = item
                .filename
                .split('.')
                .last()
                .unwrap_or("unknown")
                .to_lowercase();

            format!(
                "INSERT INTO smms_pictures (file_hash, filename, store_name, file_type, width, height, size, path, url, delete_url, page_url, created_at, updated_at) \
                 VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, '{}', '{}', '{}', '{}', '{}', datetime('now')) \
                 ON CONFLICT(file_hash) DO UPDATE SET \
                 filename = excluded.filename, \
                 store_name = excluded.store_name, \
                 width = excluded.width, \
                 height = excluded.height, \
                 size = excluded.size, \
                 path = excluded.path, \
                 url = excluded.url, \
                 delete_url = excluded.delete_url, \
                 page_url = excluded.page_url, \
                 updated_at = excluded.updated_at",
                item.hash.replace("'", "''"),
                item.filename.replace("'", "''"),
                item.store_name.replace("'", "''"),
                file_type.replace("'", "''"),
                item.width,
                item.height,
                item.size,
                item.path.replace("'", "''"),
                item.url.replace("'", "''"),
                item.delete_url.replace("'", "''"),
                item.page_url.replace("'", "''"),
                item.created_at.replace("'", "''")
            )
        })
        .collect();

    let count = batch_sqls.len();
    execute_d1_batch(batch_sqls).await?;

    Ok(format!("成功同步 {} 张图片到本地数据库", count))
}

/// 获取所有文件类型
#[tauri::command]
pub async fn get_all_file_types() -> Result<Vec<String>, String> {
    // 确保表存在
    init_smms_pictures_table().await?;

    // 只查询未删除图片的文件类型
    let sql =
        "SELECT DISTINCT file_type FROM smms_pictures WHERE is_deleted = 0 ORDER BY file_type"
            .to_string();
    let results = execute_d1_query(sql).await?;

    let types: Vec<String> = results
        .iter()
        .filter_map(|row| row.get("file_type")?.as_str().map(|s| s.to_string()))
        .collect();

    Ok(types)
}

/// 获取图片总数（支持筛选）
#[tauri::command]
pub async fn get_pictures_count(params: PictureQueryParams) -> Result<i64, String> {
    // 确保表存在
    init_smms_pictures_table().await?;

    let mut sql = "SELECT COUNT(*) as count FROM smms_pictures WHERE 1=1".to_string();

    // 删除状态筛选：None=全部, Some(false)=仅未删除, Some(true)=仅已删除
    match params.include_deleted {
        Some(false) => sql.push_str(" AND is_deleted = 0"),
        Some(true) => sql.push_str(" AND is_deleted = 1"),
        None => {} // 全部显示，不添加条件
    }

    // 追加筛选条件
    if let Some(ref ft) = params.file_type {
        if !ft.is_empty() {
            sql.push_str(&format!(" AND file_type = '{}'", ft.replace("'", "''")));
        }
    }
    if let Some(fav) = params.is_favorite {
        sql.push_str(&format!(" AND is_favorite = {}", i32::from(fav)));
    }

    // 文件名模糊搜索
    if let Some(ref fname) = params.filename {
        if !fname.is_empty() {
            sql.push_str(&format!(
                " AND filename LIKE '%{}%'",
                fname.replace("'", "''")
            ));
        }
    }

    // 存储名模糊搜索
    if let Some(ref sname) = params.store_name {
        if !sname.is_empty() {
            sql.push_str(&format!(
                " AND store_name LIKE '%{}%'",
                sname.replace("'", "''")
            ));
        }
    }

    // 备注模糊搜索
    if let Some(ref rmk) = params.remark {
        if !rmk.is_empty() {
            sql.push_str(&format!(" AND remark LIKE '%{}%'", rmk.replace("'", "''")));
        }
    }

    let results = execute_d1_query(sql).await?;

    if let Some(row) = results.first() {
        if let Some(count) = row.get("count").and_then(|v| v.as_i64()) {
            return Ok(count);
        }
    }

    Ok(0)
}

/// 查询图片列表（支持筛选、排序、分页）
#[tauri::command]
pub async fn query_smms_pictures(params: PictureQueryParams) -> Result<Vec<SmmsPicture>, String> {
    // 确保表存在
    init_smms_pictures_table().await?;

    let mut sql = "SELECT * FROM smms_pictures WHERE 1=1".to_string();

    // 删除状态筛选：None=全部, Some(false)=仅未删除, Some(true)=仅已删除
    match params.include_deleted {
        Some(false) => sql.push_str(" AND is_deleted = 0"),
        Some(true) => sql.push_str(" AND is_deleted = 1"),
        None => {} // 全部显示，不添加条件
    }

    // 追加筛选条件
    if let Some(ref ft) = params.file_type {
        if !ft.is_empty() {
            sql.push_str(&format!(" AND file_type = '{}'", ft.replace("'", "''")));
        }
    }
    if let Some(fav) = params.is_favorite {
        sql.push_str(&format!(" AND is_favorite = {}", i32::from(fav)));
    }

    // 文件名模糊搜索
    if let Some(ref fname) = params.filename {
        if !fname.is_empty() {
            sql.push_str(&format!(
                " AND filename LIKE '%{}%'",
                fname.replace("'", "''")
            ));
        }
    }

    // 存储名模糊搜索
    if let Some(ref sname) = params.store_name {
        if !sname.is_empty() {
            sql.push_str(&format!(
                " AND store_name LIKE '%{}%'",
                sname.replace("'", "''")
            ));
        }
    }

    // 备注模糊搜索
    if let Some(ref rmk) = params.remark {
        if !rmk.is_empty() {
            sql.push_str(&format!(" AND remark LIKE '%{}%'", rmk.replace("'", "''")));
        }
    }

    // 排序
    let order = params.order_by.as_deref().unwrap_or("created_at_desc");
    let order_clause = match order {
        "created_at_asc" => "created_at ASC",
        "created_at_desc" => "created_at DESC",
        "updated_at_asc" => "updated_at ASC",
        "updated_at_desc" => "updated_at DESC",
        "size_asc" => "size ASC",
        "size_desc" => "size DESC",
        _ => "created_at DESC",
    };
    sql.push_str(&format!(" ORDER BY {}", order_clause));

    // 分页
    if let Some(lim) = params.limit {
        sql.push_str(&format!(" LIMIT {}", lim));
    }
    if let Some(off) = params.offset {
        sql.push_str(&format!(" OFFSET {}", off));
    }

    let results = execute_d1_query(sql).await?;

    // 转换为 SmmsPicture 结构体，明确处理转换错误
    let mut pictures = Vec::new();
    for (index, row) in results.iter().enumerate() {
        let picture = SmmsPicture {
            id: row
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 id 字段", index))?,
            file_hash: row
                .get("file_hash")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 file_hash 字段", index))?
                .to_string(),
            filename: row
                .get("filename")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 filename 字段", index))?
                .to_string(),
            store_name: row
                .get("store_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 store_name 字段", index))?
                .to_string(),
            file_type: row
                .get("file_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 file_type 字段", index))?
                .to_string(),
            width: row
                .get("width")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 width 字段", index))?,
            height: row
                .get("height")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 height 字段", index))?,
            size: row
                .get("size")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 size 字段", index))?,
            path: row
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 path 字段", index))?
                .to_string(),
            url: row
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 url 字段", index))?
                .to_string(),
            delete_url: row
                .get("delete_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 delete_url 字段", index))?
                .to_string(),
            page_url: row
                .get("page_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 page_url 字段", index))?
                .to_string(),
            is_favorite: row
                .get("is_favorite")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 is_favorite 字段", index))?
                as i32,
            is_deleted: row.get("is_deleted").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            deleted_at: row
                .get("deleted_at")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            remark: row
                .get("remark")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: row
                .get("created_at")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 created_at 字段", index))?
                .to_string(),
            updated_at: row
                .get("updated_at")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("记录 {} 缺失或无效的 updated_at 字段", index))?
                .to_string(),
        };
        pictures.push(picture);
    }

    Ok(pictures)
}

/// 更新图片收藏状态
#[tauri::command]
pub async fn toggle_picture_favorite(id: i64, is_favorite: bool) -> Result<String, String> {
    let sql = format!(
        "UPDATE smms_pictures SET is_favorite = {}, updated_at = datetime('now') WHERE id = {}",
        if is_favorite { 1 } else { 0 },
        id
    );

    execute_d1_query(sql).await?;

    Ok(format!(
        "图片 {} 已{}收藏",
        id,
        if is_favorite { "" } else { "取消" }
    ))
}

/// 导入所有相册图片到数据库
#[tauri::command]
pub async fn import_all_smms_pictures() -> Result<SyncStats, String> {
    // 确保表存在
    init_smms_pictures_table().await?;

    // 获取数据库中所有已存在的hash集合
    let db_sql = "SELECT file_hash FROM smms_pictures WHERE is_deleted = 0".to_string();
    let db_results = execute_d1_query(db_sql).await?;
    let mut existing_hashes = HashSet::new();
    for row in db_results {
        if let Some(hash) = row.get("file_hash").and_then(|v| v.as_str()) {
            existing_hashes.insert(hash.to_string());
        }
    }

    let mut added_count = 0;
    let mut skipped_count = 0;
    let mut current_page = 1;
    let max_pages = 100;
    let mut consecutive_failures = 0;
    let max_consecutive_failures = 3;
    let batch_size = 50; // 每批处理50条记录

    // 收集所有从 API 获取到的 hash
    let mut api_hashes = HashSet::new();

    loop {
        // 获取当前页的上传历史
        let items = match get_smms_upload_history(Some(current_page)).await {
            Ok(items) => {
                consecutive_failures = 0;
                items
            }
            Err(e) => {
                consecutive_failures += 1;
                if consecutive_failures >= max_consecutive_failures {
                    if added_count + skipped_count > 0 {
                        break;
                    } else {
                        return Err(format!(
                            "连续 {} 次获取失败: {}",
                            max_consecutive_failures, e
                        ));
                    }
                }
                current_page += 1;
                continue;
            }
        };

        if items.is_empty() {
            break;
        }

        // 批量收集SQL语句
        let mut batch_sqls = Vec::new();

        for item in items {
            // 收集 API 返回的 hash
            api_hashes.insert(item.hash.clone());

            // 判断是新增还是跳过
            if existing_hashes.contains(&item.hash) {
                skipped_count += 1;
            } else {
                added_count += 1;
                existing_hashes.insert(item.hash.clone());
            }

            let file_type = item
                .filename
                .split('.')
                .last()
                .unwrap_or("unknown")
                .to_lowercase();

            let sql = format!(
                "INSERT INTO smms_pictures (file_hash, filename, store_name, file_type, width, height, size, path, url, delete_url, page_url, is_deleted, created_at, updated_at) \
                 VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, '{}', '{}', '{}', '{}', 0, '{}', datetime('now')) \
                 ON CONFLICT(file_hash) DO UPDATE SET \
                 filename = excluded.filename, \
                 store_name = excluded.store_name, \
                 width = excluded.width, \
                 height = excluded.height, \
                 size = excluded.size, \
                 path = excluded.path, \
                 url = excluded.url, \
                 delete_url = excluded.delete_url, \
                 page_url = excluded.page_url, \
                 is_deleted = 0, \
                 deleted_at = NULL, \
                 updated_at = excluded.updated_at",
                item.hash.replace("'", "''"),
                item.filename.replace("'", "''"),
                item.store_name.replace("'", "''"),
                file_type.replace("'", "''"),
                item.width,
                item.height,
                item.size,
                item.path.replace("'", "''"),
                item.url.replace("'", "''"),
                item.delete_url.replace("'", "''"),
                item.page_url.replace("'", "''"),
                item.created_at.replace("'", "''")
            );

            batch_sqls.push(sql);

            // 达到批量大小时执行
            if batch_sqls.len() >= batch_size {
                let batch = std::mem::take(&mut batch_sqls);
                execute_d1_batch(batch).await?;
            }
        }

        // 执行剩余的SQL
        if !batch_sqls.is_empty() {
            execute_d1_batch(batch_sqls).await?;
        }

        current_page += 1;

        if current_page > max_pages {
            break;
        }
    }

    // 清理逻辑：标记数据库中存在但 API 未返回的图片为已删除
    let mut deleted_count = 0;
    if !api_hashes.is_empty() {
        // 获取数据库中所有未删除的图片 hash
        let db_sql = "SELECT file_hash FROM smms_pictures WHERE is_deleted = 0".to_string();
        let db_results = execute_d1_query(db_sql).await?;

        let mut delete_sqls = Vec::new();
        for row in db_results {
            if let Some(hash) = row.get("file_hash").and_then(|v| v.as_str()) {
                if !api_hashes.contains(hash) {
                    // 数据库有但 API 没有，标记为已删除
                    let delete_sql = format!(
                        "UPDATE smms_pictures SET is_deleted = 1, deleted_at = datetime('now'), updated_at = datetime('now') WHERE file_hash = '{}'",
                        hash.replace("'", "''")
                    );
                    delete_sqls.push(delete_sql);
                    deleted_count += 1;
                }
            }
        }

        if !delete_sqls.is_empty() {
            execute_d1_batch(delete_sqls).await?;
        }
    }

    Ok(SyncStats {
        added: added_count,
        skipped: skipped_count,
        deleted: deleted_count,
    })
}

/// 上传图片到 SM.MS
#[tauri::command]
pub async fn upload_images(
    file_paths: Vec<String>,
    remark: Option<String>,
) -> Result<Vec<crate::models::UploadResult>, String> {
    use crate::models::{SmmsUploadResponse, UploadResult};

    // 加载用户凭证获取 token
    let user = load_smms_user(None).await?;

    if user.token.is_empty() {
        return Err("请先登录 SM.MS 获取 token".to_string());
    }

    // 确保表存在
    init_smms_pictures_table().await?;

    let client = reqwest::Client::new();
    let upload_url = "https://sm.ms/api/v2/upload";
    let mut results = Vec::new();

    // 逐个上传图片
    for file_path in file_paths {
        let filename = std::path::Path::new(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 读取文件
        let file_data = match tokio::fs::read(&file_path).await {
            Ok(data) => data,
            Err(e) => {
                results.push(UploadResult {
                    filename: filename.clone(),
                    success: false,
                    message: format!("读取文件失败: {}", e),
                    url: None,
                    remark: remark.clone(),
                });
                continue;
            }
        };

        // 构建 multipart form
        let part = reqwest::multipart::Part::bytes(file_data).file_name(filename.clone());

        let form = reqwest::multipart::Form::new().part("smfile", part);

        // 发送上传请求
        let response: reqwest::Response = match client
            .post(upload_url)
            .header("Authorization", &user.token)
            .multipart(form)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                results.push(UploadResult {
                    filename: filename.clone(),
                    success: false,
                    message: format!("上传请求失败: {}", e),
                    url: None,
                    remark: remark.clone(),
                });
                continue;
            }
        };

        // 解析响应
        let upload_response: SmmsUploadResponse = match response.json().await {
            Ok(resp) => resp,
            Err(e) => {
                results.push(UploadResult {
                    filename: filename.clone(),
                    success: false,
                    message: format!("解析响应失败: {}", e),
                    url: None,
                    remark: remark.clone(),
                });
                continue;
            }
        };

        // 处理上传结果
        if upload_response.success {
            if let Some(data) = upload_response.data {
                // 上传成功，插入数据库
                let file_type = data
                    .filename
                    .split('.')
                    .last()
                    .unwrap_or("unknown")
                    .to_lowercase();

                // 处理备注字段
                let remark_value = remark
                    .as_ref()
                    .map(|r| format!("'{}'", r.replace("'", "''")))
                    .unwrap_or_else(|| "NULL".to_string());

                let sql = format!(
                    "INSERT INTO smms_pictures (file_hash, filename, store_name, file_type, width, height, size, path, url, delete_url, page_url, remark, created_at, updated_at) \
                     VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, '{}', '{}', '{}', '{}', {}, datetime('now'), datetime('now')) \
                     ON CONFLICT(file_hash) DO UPDATE SET \
                     filename = excluded.filename, \
                     store_name = excluded.store_name, \
                     width = excluded.width, \
                     height = excluded.height, \
                     size = excluded.size, \
                     path = excluded.path, \
                     url = excluded.url, \
                     delete_url = excluded.delete_url, \
                     page_url = excluded.page_url, \
                     remark = excluded.remark, \
                     updated_at = excluded.updated_at",
                    data.hash.replace("'", "''"),
                    data.filename.replace("'", "''"),
                    data.store_name.replace("'", "''"),
                    file_type.replace("'", "''"),
                    data.width,
                    data.height,
                    data.size,
                    data.path.replace("'", "''"),
                    data.url.replace("'", "''"),
                    data.delete_url.replace("'", "''"),
                    data.page_url.replace("'", "''"),
                    remark_value
                );

                // 执行数据库插入
                match execute_d1_query(sql).await {
                    Ok(_) => {
                        results.push(UploadResult {
                            filename: filename.clone(),
                            success: true,
                            message: "上传成功".to_string(),
                            url: Some(data.url.clone()),
                            remark: remark.clone(),
                        });
                    }
                    Err(e) => {
                        results.push(UploadResult {
                            filename: filename.clone(),
                            success: false,
                            message: format!("上传成功但数据库插入失败: {}", e),
                            url: Some(data.url),
                            remark: remark.clone(),
                        });
                    }
                }
            } else {
                results.push(UploadResult {
                    filename: filename.clone(),
                    success: false,
                    message: "上传响应中没有数据".to_string(),
                    url: None,
                    remark: remark.clone(),
                });
            }
        } else {
            results.push(UploadResult {
                filename: filename.clone(),
                success: false,
                message: format!("上传失败: {}", upload_response.message),
                url: None,
                remark: remark.clone(),
            });
        }
    }

    Ok(results)
}

/// 删除图片（先调用 SM.MS API，成功后删除数据库记录）
#[tauri::command]
pub async fn delete_picture(id: i64) -> Result<String, String> {
    use crate::models::SmmsDeleteResponse;

    // 1. 从数据库查询图片信息
    let sql = format!(
        "SELECT delete_url, filename FROM smms_pictures WHERE id = {}",
        id
    );
    let results = execute_d1_query(sql).await?;

    if results.is_empty() {
        return Err("图片不存在".to_string());
    }

    let delete_url = results[0]["delete_url"]
        .as_str()
        .ok_or("delete_url 字段无效")?
        .to_string();

    let filename = results[0]["filename"]
        .as_str()
        .ok_or("filename 字段无效")?
        .to_string();

    // 2. 从 delete_url 中提取 hash
    // delete_url 格式: https://sm.ms/delete/HASH
    let hash = delete_url
        .split('/')
        .last()
        .ok_or("无法从 delete_url 中提取 hash")?;

    // 3. 加载用户凭证获取 token
    let user = load_smms_user(None).await?;

    // 4. 调用 SM.MS 删除 API
    let api_url = format!("https://sm.ms/api/v2/delete/{}", hash);
    let client = reqwest::Client::new();
    let response = client
        .get(&api_url)
        .header("Authorization", &user.token)
        .send()
        .await
        .map_err(|e| format!("删除请求失败: {}", e))?;

    // 5. 解析响应
    let delete_response: SmmsDeleteResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 6. 处理删除结果
    let should_delete = delete_response.success
        || delete_response
            .message
            .to_lowercase()
            .contains("already deleted");

    if should_delete {
        // 使用软删除，保留历史记录
        let delete_sql = format!(
            "UPDATE smms_pictures SET is_deleted = 1, deleted_at = datetime('now'), updated_at = datetime('now') WHERE id = {}",
            id
        );
        execute_d1_query(delete_sql).await?;

        if delete_response.success {
            Ok(format!("图片 {} 删除成功", filename))
        } else {
            Ok(format!(
                "图片 {} 已标记为删除（服务器上已不存在）",
                filename
            ))
        }
    } else {
        Err(format!("删除失败: {}", delete_response.message))
    }
}

/// 批量删除图片
#[tauri::command]
pub async fn batch_delete_pictures(
    ids: Vec<i64>,
) -> Result<crate::models::BatchDeleteResult, String> {
    use crate::models::{BatchDeleteResult, SmmsDeleteResponse};

    if ids.is_empty() {
        return Err("未选择要删除的图片".to_string());
    }

    // 加载用户凭证
    let user = load_smms_user(None).await?;
    let client = reqwest::Client::new();

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut failed_items = Vec::new();

    for id in ids {
        // 查询图片信息
        let sql = format!(
            "SELECT delete_url, filename FROM smms_pictures WHERE id = {}",
            id
        );
        let results = match execute_d1_query(sql).await {
            Ok(r) => r,
            Err(e) => {
                failed_count += 1;
                failed_items.push(format!("ID {}: {}", id, e));
                continue;
            }
        };

        if results.is_empty() {
            failed_count += 1;
            failed_items.push(format!("ID {}: 图片不存在", id));
            continue;
        }

        let delete_url = match results[0]["delete_url"].as_str() {
            Some(url) => url.to_string(),
            None => {
                failed_count += 1;
                failed_items.push(format!("ID {}: delete_url 字段无效", id));
                continue;
            }
        };

        let filename = results[0]["filename"]
            .as_str()
            .unwrap_or("未知文件")
            .to_string();

        // 提取 hash
        let hash = match delete_url.split('/').last() {
            Some(h) => h,
            None => {
                failed_count += 1;
                failed_items.push(format!("{}: 无法提取 hash", filename));
                continue;
            }
        };

        // 调用删除 API
        let api_url = format!("https://sm.ms/api/v2/delete/{}", hash);
        let response = match client
            .get(&api_url)
            .header("Authorization", &user.token)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                failed_count += 1;
                failed_items.push(format!("{}: 删除请求失败 - {}", filename, e));
                continue;
            }
        };

        // 解析响应
        let delete_response: SmmsDeleteResponse = match response.json().await {
            Ok(r) => r,
            Err(e) => {
                failed_count += 1;
                failed_items.push(format!("{}: 解析响应失败 - {}", filename, e));
                continue;
            }
        };

        // 处理删除结果
        let should_delete = delete_response.success
            || delete_response
                .message
                .to_lowercase()
                .contains("already deleted");

        if should_delete {
            // 软删除
            let delete_sql = format!(
                "UPDATE smms_pictures SET is_deleted = 1, deleted_at = datetime('now'), updated_at = datetime('now') WHERE id = {}",
                id
            );
            match execute_d1_query(delete_sql).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    failed_count += 1;
                    failed_items.push(format!("{}: 数据库更新失败 - {}", filename, e));
                }
            }
        } else {
            failed_count += 1;
            failed_items.push(format!("{}: {}", filename, delete_response.message));
        }
    }

    Ok(BatchDeleteResult {
        success_count,
        failed_count,
        failed_items,
    })
}

/// 格式化备注值为 SQL 字符串
fn format_remark_sql(remark: &Option<String>) -> String {
    remark
        .as_ref()
        .map(|r| format!("'{}'", r.replace("'", "''")))
        .unwrap_or_else(|| "NULL".to_string())
}

/// 更新图片备注
#[tauri::command]
pub async fn update_picture_remark(id: i64, remark: Option<String>) -> Result<String, String> {
    let sql = format!(
        "UPDATE smms_pictures SET remark = {}, updated_at = datetime('now') WHERE id = {}",
        format_remark_sql(&remark),
        id
    );
    execute_d1_query(sql).await?;
    Ok("备注更新成功".to_string())
}

/// 批量更新图片备注
#[tauri::command]
pub async fn batch_update_picture_remark(
    ids: Vec<i64>,
    remark: Option<String>,
) -> Result<String, String> {
    if ids.is_empty() {
        return Err("未选择要更新的图片".to_string());
    }

    let ids_str = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let sql = format!(
        "UPDATE smms_pictures SET remark = {}, updated_at = datetime('now') WHERE id IN ({})",
        format_remark_sql(&remark),
        ids_str
    );
    execute_d1_query(sql).await?;
    Ok(format!("成功更新 {} 张图片的备注", ids.len()))
}
