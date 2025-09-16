use aes::Aes128;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use cbc::Decryptor;
use cbc::cipher::{BlockDecryptMut, KeyIvInit, block_padding};
use rand::Rng;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;

type Aes128CbcDec = Decryptor<Aes128>;

pub fn generate_nonce_str(length: usize) -> String {
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset.chars().nth(idx).unwrap()
        })
        .collect()
}

pub fn calculate_signature(
    jsapi_ticket: &str,
    nonce_str: &str,
    timestamp: u64,
    url: &str,
) -> String {
    let timestamp_str = timestamp.to_string();
    let mut params = BTreeMap::new();
    params.insert("jsapi_ticket", jsapi_ticket);
    params.insert("noncestr", nonce_str);
    params.insert("timestamp", timestamp_str.as_str());
    params.insert("url", url);

    let query_string = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let mut hasher = Sha1::new();
    hasher.update(query_string.as_bytes());
    let result = hasher.finalize();

    hex::encode(result)
}

pub fn decrypt_phone_data(
    encrypted_data: &str,
    session_key: &str,
    iv: &str,
) -> Result<String, String> {
    let encrypted = BASE64
        .decode(encrypted_data)
        .map_err(|e| format!("Failed to decode encrypted data: {}", e))?;

    let key = BASE64
        .decode(session_key)
        .map_err(|e| format!("Failed to decode session key: {}", e))?;

    let iv_bytes = BASE64
        .decode(iv)
        .map_err(|e| format!("Failed to decode IV: {}", e))?;

    if key.len() != 16 {
        return Err(format!("Invalid session key length: {}", key.len()));
    }

    if iv_bytes.len() != 16 {
        return Err(format!("Invalid IV length: {}", iv_bytes.len()));
    }

    // 将key和iv转换为固定大小的数组
    let key_array: [u8; 16] = key.try_into()
        .map_err(|_| "Failed to convert key to array".to_string())?;
    let iv_array: [u8; 16] = iv_bytes.try_into()
        .map_err(|_| "Failed to convert IV to array".to_string())?;

    let cipher = Aes128CbcDec::new(&key_array.into(), &iv_array.into());

    let mut buf = encrypted.clone();
    let decrypted = cipher.decrypt_padded_mut::<block_padding::Pkcs7>(&mut buf)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(decrypted.to_vec())
        .map_err(|e| format!("Failed to convert decrypted data to UTF-8: {}", e))
}

pub fn sha1_hash(data: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_nonce_str() {
        let nonce = generate_nonce_str(16);
        assert_eq!(nonce.len(), 16);
        assert!(nonce.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_sha1_hash() {
        let data = "test_string";
        let hash = sha1_hash(data);
        assert_eq!(hash.len(), 40);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_calculate_signature() {
        let jsapi_ticket = "test_ticket";
        let nonce_str = "test_nonce";
        let timestamp = 1234567890;
        let url = "https://example.com";

        let signature = calculate_signature(jsapi_ticket, nonce_str, timestamp, url);
        assert_eq!(signature.len(), 40);
    }
}