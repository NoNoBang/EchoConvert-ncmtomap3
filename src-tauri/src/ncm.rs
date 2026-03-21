use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use aes::Aes128;
use aes::cipher::{BlockDecrypt, KeyInit};
use generic_array::GenericArray;

const CORE_KEY: &[u8] = b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
const META_KEY: &[u8] = b"687A4852416D646FEB7C6C6F62615D4D";

pub fn convert(files: &[String]) -> Result<usize, String> {
    let mut converted = 0;
    
    for file_path in files {
        if let Err(e) = convert_single(file_path) {
            eprintln!("Failed to convert {}: {}", file_path, e);
            // Continue with other files instead of failing completely
        } else {
            converted += 1;
        }
    }
    
    Ok(converted)
}

fn convert_single(ncm_path: &str) -> Result<(), String> {
    // 1. Read NCM file
    let mut ncm_data = Vec::new();
    let mut file = File::open(ncm_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    file.read_to_end(&mut ncm_data)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // 2. Validate NCM header
    if !ncm_data.starts_with(b"CTN") {
        return Err("Invalid NCM file format".to_string());
    }
    
    // 3. Find audio data (NCM structure: header + metadata + audio)
    // For now, this is a placeholder - real implementation requires full NCM parsing
    
    // 4. Decrypt and extract MP3 data
    // TODO: Implement actual AES decryption and RC4 cipher
    
    // 5. Write output MP3
    let output_path = ncm_path.replace(".ncm", ".mp3");
    let mut output_file = File::create(&output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    // For now, just create a placeholder
    output_file.write_all(b"ID3")  // MP3 ID3 tag prefix
        .map_err(|e| format!("Failed to write output: {}", e))?;
    
    Ok(())
}

// Helper: Decrypt with AES-128-ECB
fn aes_128_ecb_decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 16 {
        return Err("Key must be 16 bytes".to_string());
    }
    
    let cipher = Aes128::new_from_slice(key)
        .map_err(|_| "Invalid key size".to_string())?;
    
    let mut decrypted = data.to_vec();
    
    for chunk in decrypted.chunks_exact_mut(16) {
        let block = GenericArray::from_mut_slice(chunk);
        cipher.decrypt_block(block);
    }
    
    Ok(decrypted)
}
