use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use tauri::AppHandle;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadFileInfo {
    pub url: String,
    pub filename: String,
}

/// 下载单个文件
#[tauri::command]
pub async fn download_single_file(
    _app: AppHandle,
    url: String,
    save_path: String,
) -> Result<String, String> {
    // 下载文件
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("下载失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取文件内容失败: {}", e))?;

    // 保存文件
    let mut file = File::create(&save_path).map_err(|e| format!("创建文件失败: {}", e))?;

    file.write_all(&bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(format!("文件已保存到: {}", save_path))
}

/// 批量下载文件并打包成 zip
#[tauri::command]
pub async fn download_files_as_zip(
    _app: AppHandle,
    files: Vec<DownloadFileInfo>,
    save_path: String,
) -> Result<String, String> {
    if files.is_empty() {
        return Err("没有要下载的文件".to_string());
    }

    // 创建 zip 文件
    let zip_file = File::create(&save_path).map_err(|e| format!("创建 ZIP 文件失败: {}", e))?;

    let mut zip = ZipWriter::new(zip_file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let mut success_count = 0;
    let mut failed_files = Vec::new();

    // 下载并添加每个文件到 zip
    for file_info in &files {
        match download_and_add_to_zip(&mut zip, file_info, &options).await {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                failed_files.push(format!("{}: {}", file_info.filename, e));
            }
        }
    }

    // 完成 zip 文件
    zip.finish()
        .map_err(|e| format!("完成 ZIP 文件失败: {}", e))?;

    if success_count == 0 {
        return Err(format!("所有文件下载失败:\n{}", failed_files.join("\n")));
    }

    if !failed_files.is_empty() {
        Ok(format!(
            "成功打包 {} 个文件，{} 个失败\n失败文件:\n{}",
            success_count,
            failed_files.len(),
            failed_files.join("\n")
        ))
    } else {
        Ok(format!(
            "成功打包 {} 个文件到: {}",
            success_count, save_path
        ))
    }
}

/// 下载文件并添加到 zip
async fn download_and_add_to_zip(
    zip: &mut ZipWriter<File>,
    file_info: &DownloadFileInfo,
    options: &FileOptions,
) -> Result<(), String> {
    // 下载文件
    let response = reqwest::get(&file_info.url)
        .await
        .map_err(|e| format!("下载失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取内容失败: {}", e))?;

    // 添加到 zip
    zip.start_file(&file_info.filename, *options)
        .map_err(|e| format!("添加到 ZIP 失败: {}", e))?;

    zip.write_all(&bytes)
        .map_err(|e| format!("写入 ZIP 失败: {}", e))?;

    Ok(())
}
