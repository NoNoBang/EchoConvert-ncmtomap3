use std::path::PathBuf;

use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::settings;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistAnalysisResult {
    pub playlist_id: u64,
    pub title: String,
    pub creator: String,
    pub cover_url: Option<String>,
    pub track_count: usize,
    pub tracks: Vec<PlaylistTrack>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistTrack {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover_url: Option<String>,
    pub duration_ms: u64,
    pub duration_label: String,
    pub size_bytes: u64,
    pub size_label: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTrackRequest {
    pub id: u64,
    pub title: String,
    pub artist: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItemResult {
    pub id: u64,
    pub title: String,
    pub saved_path: Option<String>,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadBatchResult {
    pub total: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub save_dir: String,
    pub items: Vec<DownloadItemResult>,
}

pub async fn analyze_playlist(query: &str) -> Result<PlaylistAnalysisResult> {
    let playlist_id = parse_playlist_id(query)?;
    let url = format!(
        "https://music.163.com/api/v6/playlist/detail?id={}&n=1000",
        playlist_id
    );

    let response = client()
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let playlist = response
        .get("playlist")
        .or_else(|| response.get("result"))
        .ok_or_else(|| anyhow!("未获取到歌单信息"))?;

    let title = playlist
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or("未命名歌单")
        .to_string();

    let creator = playlist
        .get("creator")
        .and_then(|value| value.get("nickname"))
        .and_then(|value| value.as_str())
        .unwrap_or("未知用户")
        .to_string();

    let cover_url = playlist
        .get("coverImgUrl")
        .and_then(|value| value.as_str())
        .map(str::to_string);

    let track_ids = playlist
        .get("trackIds")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(|value| value.as_u64()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let tracks = if track_ids.is_empty() {
        playlist
            .get("tracks")
            .and_then(|value| value.as_array())
            .map(|items| items.iter().map(map_track).collect::<Vec<_>>())
            .unwrap_or_default()
    } else {
        fetch_tracks_by_ids(&track_ids).await?
    };

    Ok(PlaylistAnalysisResult {
        playlist_id,
        title,
        creator,
        cover_url,
        track_count: tracks.len(),
        tracks,
    })
}

async fn fetch_tracks_by_ids(track_ids: &[u64]) -> Result<Vec<PlaylistTrack>> {
    let mut tracks = Vec::with_capacity(track_ids.len());

    for chunk in track_ids.chunks(200) {
        let ids = chunk
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");

        let url = format!("https://music.163.com/api/song/detail?ids=[{}]", ids);
        let response = client()
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        let songs = response
            .get("songs")
            .and_then(|value| value.as_array())
            .ok_or_else(|| anyhow!("未获取到歌曲详情"))?;

        tracks.extend(songs.iter().map(map_track));
    }

    Ok(tracks)
}

pub async fn download_tracks(tracks: Vec<DownloadTrackRequest>) -> Result<DownloadBatchResult> {
    if tracks.is_empty() {
        return Err(anyhow!("没有可下载的歌曲"));
    }

    let save_dir = settings::effective_download_dir()?;
    let save_dir_string = save_dir.to_string_lossy().into_owned();
    let mut items = Vec::with_capacity(tracks.len());

    for track in tracks {
        match download_single_track(&save_dir, &track).await {
            Ok(saved_path) => items.push(DownloadItemResult {
                id: track.id,
                title: track.title.clone(),
                saved_path: Some(saved_path),
                status: "done".into(),
                message: "下载完成".into(),
            }),
            Err(error) => items.push(DownloadItemResult {
                id: track.id,
                title: track.title.clone(),
                saved_path: None,
                status: "error".into(),
                message: error.to_string(),
            }),
        }
    }

    let success_count = items.iter().filter(|item| item.status == "done").count();
    let failure_count = items.len().saturating_sub(success_count);

    Ok(DownloadBatchResult {
        total: items.len(),
        success_count,
        failure_count,
        save_dir: save_dir_string,
        items,
    })
}

async fn download_single_track(save_dir: &PathBuf, track: &DownloadTrackRequest) -> Result<String> {
    let url = format!(
        "https://music.163.com/song/media/outer/url?id={}.mp3",
        track.id
    );
    let response = client().get(url).send().await?.error_for_status()?;
    let final_url = response.url().to_string();
    let bytes = response.bytes().await?;

    if bytes.is_empty() {
        return Err(anyhow!("下载结果为空"));
    }

    let extension = infer_extension(&final_url);
    let file_name = format!(
        "{} - {}.{}",
        sanitize_filename(&track.artist),
        sanitize_filename(&track.title),
        extension
    );
    let output_path = save_dir.join(file_name);
    fs::write(&output_path, bytes).await?;

    Ok(output_path.to_string_lossy().into_owned())
}

fn client() -> reqwest::Client {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36"),
    );
    headers.insert(REFERER, HeaderValue::from_static("https://music.163.com/"));

    reqwest::Client::builder()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .expect("http client")
}

fn parse_playlist_id(input: &str) -> Result<u64> {
    let number_regex = Regex::new(r"(?P<id>\d{5,})").expect("playlist regex");
    let captures = number_regex
        .captures(input)
        .ok_or_else(|| anyhow!("未识别到歌单 ID"))?;

    captures["id"]
        .parse::<u64>()
        .map_err(|_| anyhow!("歌单 ID 解析失败"))
}

fn infer_extension(url: &str) -> &'static str {
    if url.contains(".flac") {
        "flac"
    } else if url.contains(".m4a") {
        "m4a"
    } else {
        "mp3"
    }
}

fn sanitize_filename(value: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let sanitized = value
        .chars()
        .map(|ch| {
            if invalid.contains(&ch) || ch.is_control() {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();

    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        "untitled".into()
    } else {
        trimmed.into()
    }
}

fn format_duration(duration_ms: u64) -> String {
    let total_seconds = duration_ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

fn map_track(item: &serde_json::Value) -> PlaylistTrack {
    let duration_ms = item.get("dt").and_then(|value| value.as_u64()).unwrap_or(0);
    let size_bytes = extract_track_size(item);

    PlaylistTrack {
        id: item
            .get("id")
            .and_then(|value| value.as_u64())
            .unwrap_or_default(),
        title: item
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("未知歌曲")
            .to_string(),
        artist: extract_artist(item),
        album: extract_album(item),
        cover_url: extract_cover_url(item),
        duration_ms,
        duration_label: format_duration(duration_ms),
        size_bytes,
        size_label: format_size(size_bytes),
    }
}

fn extract_artist(item: &serde_json::Value) -> String {
    item.get("ar")
        .or_else(|| item.get("artists"))
        .and_then(|value| value.as_array())
        .map(|artists| {
            artists
                .iter()
                .filter_map(|artist| artist.get("name").and_then(|value| value.as_str()))
                .collect::<Vec<_>>()
                .join(" / ")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "未知歌手".into())
}

fn extract_album(item: &serde_json::Value) -> String {
    item.get("al")
        .or_else(|| item.get("album"))
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
        .unwrap_or("未知专辑")
        .to_string()
}

fn extract_cover_url(item: &serde_json::Value) -> Option<String> {
    item.get("al")
        .or_else(|| item.get("album"))
        .and_then(|value| value.get("picUrl").or_else(|| value.get("blurPicUrl")))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn extract_track_size(item: &serde_json::Value) -> u64 {
    for quality_key in ["hr", "sq", "h", "m", "l", "bMusic", "mMusic", "lMusic"] {
        let size = item
            .get(quality_key)
            .and_then(|value| value.get("size").or_else(|| value.get("fileSize")))
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        if size > 0 {
            return size;
        }
    }
    0
}

fn format_size(size_bytes: u64) -> String {
    if size_bytes == 0 {
        return "未知大小".into();
    }

    let size_mb = size_bytes as f64 / 1024f64 / 1024f64;
    if size_mb >= 1.0 {
        format!("{:.1} MB", size_mb)
    } else {
        let size_kb = size_bytes as f64 / 1024f64;
        format!("{:.0} KB", size_kb)
    }
}
