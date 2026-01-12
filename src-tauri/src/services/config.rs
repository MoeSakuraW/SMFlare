use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::models::D1Config;

/// 缓存 D1 配置，减少重复的文件和密钥链读取
pub static D1_CONFIG_CACHE: RwLock<Option<D1Config>> = RwLock::new(None);

/// 获取配置文件路径
pub fn get_config_path() -> Result<PathBuf, String> {
    let mut path = dirs::config_dir().ok_or("无法获取配置目录")?;
    path.push("tauri-app");
    fs::create_dir_all(&path).map_err(|e| format!("创建配置目录失败: {}", e))?;
    path.push("d1_config.json");
    Ok(path)
}

/// 更新 D1 配置缓存
pub fn update_d1_config_cache(config: D1Config) {
    if let Ok(mut cache) = D1_CONFIG_CACHE.write() {
        *cache = Some(config);
    }
}

/// 清除 D1 配置缓存
pub fn clear_d1_config_cache() {
    if let Ok(mut cache) = D1_CONFIG_CACHE.write() {
        *cache = None;
    }
}
