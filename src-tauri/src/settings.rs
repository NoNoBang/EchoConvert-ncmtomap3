use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub download_dir: Option<String>,
}

pub fn load_settings() -> Result<AppSettings> {
    let path = settings_file_path()?;
    if !path.exists() {
        return Ok(AppSettings {
            download_dir: Some(default_download_dir()?.to_string_lossy().into_owned()),
        });
    }

    let content = fs::read_to_string(path)?;
    let mut settings: AppSettings = serde_json::from_str(&content)?;
    if settings.download_dir.is_none() {
        settings.download_dir = Some(default_download_dir()?.to_string_lossy().into_owned());
    }
    Ok(settings)
}

pub fn save_settings(settings: &AppSettings) -> Result<AppSettings> {
    let path = settings_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(settings)?;
    fs::write(path, content)?;
    Ok(settings.clone())
}

pub fn set_download_dir(path: String) -> Result<AppSettings> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("存储路径不能为空"));
    }

    let dir = PathBuf::from(trimmed);
    fs::create_dir_all(&dir)?;

    let settings = AppSettings {
        download_dir: Some(dir.to_string_lossy().into_owned()),
    };
    save_settings(&settings)
}

pub fn effective_download_dir() -> Result<PathBuf> {
    let settings = load_settings()?;
    let dir = settings
        .download_dir
        .map(PathBuf::from)
        .unwrap_or(default_download_dir()?);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn settings_file_path() -> Result<PathBuf> {
    Ok(app_home_dir()?.join("settings.json"))
}

fn app_home_dir() -> Result<PathBuf> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .ok_or_else(|| anyhow!("无法定位用户目录"))?;
    Ok(home.join(".ncm2mp3"))
}

fn default_download_dir() -> Result<PathBuf> {
    Ok(downloads_dir()?.join("EchoConvert"))
}

fn downloads_dir() -> Result<PathBuf> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .ok_or_else(|| anyhow!("无法定位下载目录"))?;
    Ok(home.join("Downloads"))
}
