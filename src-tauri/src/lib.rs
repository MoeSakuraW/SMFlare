mod commands;
mod models;
mod services;

use commands::{
    batch_delete_pictures, batch_update_picture_remark, delete_d1_config, delete_picture,
    download_files_as_zip, download_single_file, execute_d1_query, get_all_file_types,
    get_pictures_count, get_smms_token, get_smms_upload_history, import_all_smms_pictures,
    init_smms_pictures_table, load_d1_config, load_smms_user, query_smms_pictures, save_d1_config,
    save_smms_user, sync_smms_pictures, test_d1_connection, toggle_picture_favorite,
    update_picture_remark, upload_images,
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_smms_token,
            save_smms_user,
            load_smms_user,
            get_smms_upload_history,
            save_d1_config,
            load_d1_config,
            delete_d1_config,
            test_d1_connection,
            execute_d1_query,
            init_smms_pictures_table,
            sync_smms_pictures,
            query_smms_pictures,
            toggle_picture_favorite,
            import_all_smms_pictures,
            get_pictures_count,
            get_all_file_types,
            upload_images,
            delete_picture,
            batch_delete_pictures,
            download_single_file,
            download_files_as_zip,
            update_picture_remark,
            batch_update_picture_remark
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
