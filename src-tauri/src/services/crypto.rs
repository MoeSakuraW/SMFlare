use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// 从 D1 配置派生加密密钥（不使用 keyring）
fn derive_encryption_key(account_id: &str, database_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(database_id.as_bytes());
    hasher.update(b"smms_encryption_salt_v1"); // 固定盐值
    hasher.finalize().into()
}

/// 加密密码（需要 D1 配置派生密钥）
pub fn encrypt_password(password: &str, account_id: &str, database_id: &str) -> Result<String, String> {
    let key = derive_encryption_key(account_id, database_id);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建加密器失败: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, password.as_bytes())
        .map_err(|e| format!("加密失败: {}", e))?;

    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(general_purpose::STANDARD.encode(&result))
}

/// 解密密码（需要 D1 配置派生密钥）
pub fn decrypt_password(encrypted: &str, account_id: &str, database_id: &str) -> Result<String, String> {
    let key = derive_encryption_key(account_id, database_id);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建解密器失败: {}", e))?;

    let data = general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("解码加密数据失败: {}", e))?;

    if data.len() < 12 {
        return Err("加密数据格式错误".to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("解密失败: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("解密后的数据不是有效的 UTF-8: {}", e))
}
