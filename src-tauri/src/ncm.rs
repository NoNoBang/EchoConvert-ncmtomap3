use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use aes::cipher::{generic_array::GenericArray, BlockDecrypt, KeyInit};
use aes::Aes128;
use anyhow::{anyhow, Result};
use serde::Serialize;

const CORE_KEY: &[u8] = b"hzHRDJ5fy5HIGaKS";
const META_KEY: &[u8] = b"!@#$%^&*(^)_+-=[]{}|;',<>?";

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConversionItemResult {
    pub source_path: String,
    pub output_path: Option<String>,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConversionBatchResult {
    pub total: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub items: Vec<ConversionItemResult>,
}

pub fn convert(files: &[String]) -> ConversionBatchResult {
    let mut items = Vec::with_capacity(files.len());

    for file_path in files {
        match convert_single(file_path) {
            Ok(output_path) => {
                log::info!("Successfully converted: {}", file_path);
                items.push(ConversionItemResult {
                    source_path: file_path.clone(),
                    output_path: Some(output_path),
                    status: "done".into(),
                    message: "转换完成".into(),
                });
            }
            Err(error) => {
                log::error!("Failed to convert {}: {}", file_path, error);
                items.push(ConversionItemResult {
                    source_path: file_path.clone(),
                    output_path: None,
                    status: "error".into(),
                    message: error.to_string(),
                });
            }
        }
    }

    let success_count = items.iter().filter(|item| item.status == "done").count();
    let failure_count = items.len().saturating_sub(success_count);

    ConversionBatchResult {
        total: items.len(),
        success_count,
        failure_count,
        items,
    }
}

struct NcmFile {
    key_data: Vec<u8>,
    meta_data: Vec<u8>,
    audio_data: Vec<u8>,
}

fn convert_single(ncm_path: &str) -> Result<String> {
    let mut file = File::open(ncm_path)?;
    let ncm = parse_ncm(&mut file)?;

    let rc4_key = decrypt_aes(&ncm.key_data, CORE_KEY)?;
    let meta_json_raw = decrypt_aes(&ncm.meta_data, META_KEY)?;
    let meta_json = parse_metadata(&meta_json_raw)?;
    let audio_data = decrypt_rc4(&ncm.audio_data, &rc4_key)?;

    let format = meta_json
        .get("format")
        .and_then(|value| value.as_str())
        .unwrap_or("mp3");

    let output_path = build_output_path(Path::new(ncm_path), format);
    let mut out_file = File::create(&output_path)?;
    out_file.write_all(&audio_data)?;

    Ok(output_path.to_string_lossy().into_owned())
}

fn build_output_path(input_path: &Path, extension: &str) -> PathBuf {
    let mut output_path = input_path.to_path_buf();
    output_path.set_extension(extension);
    output_path
}

fn parse_ncm(file: &mut File) -> Result<NcmFile> {
    let mut header = [0u8; 8];
    file.read_exact(&mut header)?;

    if header != *b"CTNFv2\0\0" && !header.starts_with(b"CTNFv") {
        return Err(anyhow!("无效的 NCM 文件头"));
    }

    let mut key_len_bytes = [0u8; 4];
    file.read_exact(&mut key_len_bytes)?;
    let key_len = u32::from_le_bytes(key_len_bytes) as usize;
    let mut key_data = vec![0u8; key_len];
    file.read_exact(&mut key_data)?;

    let mut meta_len_bytes = [0u8; 4];
    file.read_exact(&mut meta_len_bytes)?;
    let meta_len = u32::from_le_bytes(meta_len_bytes) as usize;
    let mut meta_data = vec![0u8; meta_len];
    file.read_exact(&mut meta_data)?;

    let mut skip_buf = [0u8; 9];
    let _ = file.read(&mut skip_buf);

    let mut audio_data = Vec::new();
    file.read_to_end(&mut audio_data)?;

    Ok(NcmFile {
        key_data,
        meta_data,
        audio_data,
    })
}

fn decrypt_aes(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 16 {
        return Err(anyhow!("AES 密钥长度错误"));
    }

    if data.len() % 16 != 0 {
        return Err(anyhow!("加密数据长度不是 16 的倍数"));
    }

    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut decrypted = data.to_vec();

    for chunk in decrypted.chunks_exact_mut(16) {
        let block = GenericArray::from_mut_slice(chunk);
        cipher.decrypt_block(block);
    }

    let padding_len = *decrypted.last().ok_or_else(|| anyhow!("空数据"))? as usize;
    if padding_len > 0 && padding_len <= 16 {
        let new_len = decrypted.len().saturating_sub(padding_len);
        decrypted.truncate(new_len);
    }

    Ok(decrypted)
}

fn parse_metadata(raw: &[u8]) -> Result<serde_json::Value> {
    let start_marker = b"163 key(NetEase cloud music)";

    let data_start = if let Some(position) = raw
        .windows(start_marker.len())
        .position(|window| window == start_marker)
    {
        position + start_marker.len()
    } else {
        0
    };

    let meta_str = String::from_utf8_lossy(&raw[data_start..]);

    match serde_json::from_str::<serde_json::Value>(&meta_str) {
        Ok(json) => Ok(json),
        Err(_) => {
            serde_json::from_slice(raw).map_err(|error| anyhow!("元数据 JSON 解析失败: {}", error))
        }
    }
}

fn decrypt_rc4(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if key.is_empty() {
        return Err(anyhow!("RC4 密钥为空"));
    }

    let mut s = [0u8; 256];
    for (index, item) in s.iter_mut().enumerate() {
        *item = index as u8;
    }

    let mut j = 0u8;
    for i in 0..256 {
        j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
        s.swap(i, j as usize);
    }

    let mut decrypted = data.to_vec();
    let mut i = 0u8;
    let mut j = 0u8;

    for byte in &mut decrypted {
        i = i.wrapping_add(1);
        j = j.wrapping_add(s[i as usize]);
        s.swap(i as usize, j as usize);
        let k = s[(s[i as usize].wrapping_add(s[j as usize])) as usize];
        *byte ^= k;
    }

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_path_replaces_extension() {
        let path = build_output_path(Path::new("/tmp/example.ncm"), "mp3");
        assert_eq!(path.to_string_lossy(), "/tmp/example.mp3");
    }
}
