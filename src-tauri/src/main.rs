#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ncm;
mod online;
mod settings;

use log::LevelFilter;
use online::{DownloadBatchResult, DownloadTrackRequest, PlaylistAnalysisResult};
use settings::AppSettings;

fn main() {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();

    #[cfg(not(debug_assertions))]
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            analyze_online_playlist,
            download_online_tracks,
            convert_ncm_files,
            get_app_settings,
            set_download_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn analyze_online_playlist(query: String) -> Result<PlaylistAnalysisResult, String> {
    online::analyze_playlist(&query)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn download_online_tracks(
    tracks: Vec<DownloadTrackRequest>,
) -> Result<DownloadBatchResult, String> {
    online::download_tracks(tracks)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn convert_ncm_files(files: Vec<String>) -> Result<ncm::ConversionBatchResult, String> {
    Ok(ncm::convert(&files))
}

#[tauri::command]
async fn get_app_settings() -> Result<AppSettings, String> {
    settings::load_settings().map_err(|error| error.to_string())
}

#[tauri::command]
async fn set_download_directory(path: String) -> Result<AppSettings, String> {
    settings::set_download_dir(path).map_err(|error| error.to_string())
}
