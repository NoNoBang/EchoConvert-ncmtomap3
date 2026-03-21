// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ncm;

use tauri::Manager;
use log::LevelFilter;

fn main() {
    // Initialize logging
    #[cfg(debug_assertions)]
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();
    
    #[cfg(not(debug_assertions))]
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();
    
    log::info!("Starting NCM to MP3 converter");
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![convert_ncm_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn convert_ncm_files(files: Vec<String>) -> Result<String, String> {
    log::info!("Converting {} files", files.len());
    
    match ncm::convert(&files) {
        Ok(count) => {
            let message = format!("Successfully converted {} files", count);
            log::info!("{}", message);
            Ok(message)
        }
        Err(e) => {
            let error_msg = format!("Conversion error: {}", e);
            log::error!("{}", error_msg);
            Err(error_msg)
        }
    }
}
