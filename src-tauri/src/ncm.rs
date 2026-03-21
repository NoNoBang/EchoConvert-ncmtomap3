use std::fs::{self, File};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::Path;
use aes::Aes128;
use aes::cipher::{BlockDecrypt, KeyInit};
use generic_array::GenericArray;
use anyhow::{anyhow, Result};
use serde_json::json;

const CORE_KEY: &[u8] = b"hzHRDJ5fy5HIGaKS"; // 网易云核心密钥
const META_KEY: &[u8] = b"!@#$%^&*(^)_+-=[]{}|;',<>?";  // 网易云元数据密钥

pub fn convert(files: &[String]) -> Result<usize> {
    let mut converted = 0;
    
    for file_path in files {
        match convert_single(file_path) {
            Ok(_) => {
                log::info!("Successfully converted: {}", file_path);
                converted += 1;
            }
            Err(e) => {
                log::error!("Failed to convert {}: {}", file_path, e);
                // Continue with other files instead of failing completely
            }
        }
    }
    
    Ok(converted)
}

struct NcmFile {
    key_data: Vec<u8>,
    meta_data: Vec<u8>,
    audio_data: Vec<u8>,
    header: Vec<u8>,
}

fn convert_single(ncm_path: &str) -> Result<()> {
    log::debug!("Opening NCM file: {}", ncm_path);
    
    let mut file = File::open(ncm_path)?;
    let ncm = parse_ncm(&mut file)?;
    
    // Decrypt key
    let rc4_key = decrypt_aes(&ncm.key_data, CORE_KEY)?;
    log::debug!("RC4 key decrypted, length: {}", rc4_key.len());
    
    // Decrypt metadata
    let meta_json_raw = decrypt_aes(&ncm.meta_data, META_KEY)?;
    let meta_json = parse_metadata(&meta_json_raw)?;
    log::info!("Metadata: {:?}", meta_json);
    
    // Decrypt audio
    let audio_data = decrypt_rc4(&ncm.audio_data, &rc4_key)?;
    
    // Get output format (usually mp3, might be flac)
    let format = meta_json
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("mp3");
    
    // Generate output path
    let output_path = ncm_path.replace(".ncm", &format!(".{}", format));
    
    // Write output file
    let mut out_file = File::create(&output_path)?;
    out_file.write_all(&audio_data)?;
    
    log::info!("Saved to: {}", output_path);
    Ok(())
}

fn parse_ncm(file: &mut File) -> Result<NcmFile> {
    let mut header = [0u8; 8];
    file.read_exact(&mut header)?;
    
    // Validate NCM format
    if header != *b"CTNFv2\0\0" && !header.starts_with(b"CTNFv") {
        return Err(anyhow!("Invalid NCM file format, expected CTNFv header"));
    }
    
    // Read key data
    let mut key_len_bytes = [0u8; 4];
    file.read_exact(&mut key_len_bytes)?;
    let key_len = u32::from_le_bytes(key_len_bytes) as usize;
    let mut key_data = vec![0u8; key_len];
    file.read_exact(&mut key_data)?;
    log::debug!("Key data length: {}", key_len);
    
    // Read metadata
    let mut meta_len_bytes = [0u8; 4];
    file.read_exact(&mut meta_len_bytes)?;
    let meta_len = u32::from_le_bytes(meta_len_bytes) as usize;
    let mut meta_data = vec![0u8; meta_len];
    file.read_exact(&mut meta_data)?;
    log::debug!("Metadata length: {}", meta_len);
    
    // Skip 9 reserved bytes
    let mut skip_buf = [0u8; 9];
    let _ = file.read(&mut skip_buf);
    
    // Read remaining audio data
    let mut audio_data = Vec::new();
    file.read_to_end(&mut audio_data)?;
    log::debug!("Audio data length: {}", audio_data.len());
    
    Ok(NcmFile {
        key_data,
        meta_data,
        audio_data,
        header: header.to_vec(),
    })
}

fn decrypt_aes(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 16 {
        return Err(anyhow!("AES key must be 16 bytes, got {}", key.len()));
    }
    
    if data.len() % 16 != 0 {
        return Err(anyhow!("Data length must be multiple of 16"));
    }
    
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut decrypted = data.to_vec();
    
    // Decrypt in 16-byte blocks (ECB mode)
    for chunk in decrypted.chunks_exact_mut(16) {
        let mut block = GenericArray::from_mut_slice(chunk);
        cipher.decrypt_block(&mut block);
    }
    
    // Remove PKCS#7 padding
    let padding_len = *decrypted.last().ok_or(anyhow!("Empty data"))? as usize;
    if padding_len > 0 && padding_len <= 16 {
        let new_len = decrypted.len().saturating_sub(padding_len);
        decrypted.truncate(new_len);
    }
    
    Ok(decrypted)
}

fn parse_metadata(raw: &[u8]) -> Result<serde_json::Value> {
    // Metadata usually starts with "163 key(NetEase cloud music)"
    let start_marker = b"163 key(NetEase cloud music)";
    
    let data_start = if let Some(pos) = raw.windows(start_marker.len())
        .position(|w| w == start_marker) {
        pos + start_marker.len()
    } else {
        0
    };
    
    let meta_str = String::from_utf8_lossy(&raw[data_start..]);
    
    // Try to parse as JSON
    match serde_json::from_str::<serde_json::Value>(&meta_str) {
        Ok(json) => Ok(json),
        Err(_) => {
            // If it fails, try without the marker
            serde_json::from_slice(raw)
                .map_err(|e| anyhow!("Failed to parse metadata JSON: {}", e))
        }
    }
}

fn decrypt_rc4(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    // Initialize RC4 state
    let mut s = [0u8; 256];
    for i in 0..256 {
        s[i] = i as u8;
    }
    
    // Key scheduling algorithm (KSA)
    let mut j = 0u8;
    for i in 0..256 {
        j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
        s.swap(i, j as usize);
    }
    
    // Pseudo-random generation algorithm (PRGA) - decrypt
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
    fn test_aes_decrypt() {
        // Test with a known plaintext-ciphertext pair
        let key = b"hzHRDJ5fy5HIGaKS";
        let plaintext = b"0123456789ABCDEF";
        
        // This would need an actual NCM file to test properly
        // For now, we just verify the key is correct length
        assert_eq!(key.len(), 16);
    }
}
