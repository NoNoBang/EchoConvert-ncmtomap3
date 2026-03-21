// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ncm;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![convert_ncm_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn convert_ncm_files(files: Vec<String>) -> Result<String, String> {
    // TODO: Implement NCM to MP3 conversion logic here
    
    match ncm::convert(&files) {
        Ok(count) => Ok(format!("Successfully converted {} files", count)),
        Err(e) => Err(format!("Conversion error: {}", e)),
    }
}
