use aes_gcm::aead::{Aead, AeadCore, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use zeroize::Zeroizing;

use crate::error::CryptoError;

/// 加密单个字段，返回 base64 编码的 nonce + ciphertext
/// nonce 每次随机生成（12 字节），prepend 到密文前
pub fn encrypt_field(key: &[u8; 32], plaintext: &[u8]) -> Result<String, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::CipherInitError)?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let mut ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| CryptoError::EncryptError)?;

    // prepend nonce (12 字节) 到密文前
    let mut result = nonce.to_vec();
    result.append(&mut ciphertext);

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        result,
    ))
}

/// 解密字段，解密后的数据用 Zeroizing 包装确保使用后清零
pub fn decrypt_field(key: &[u8; 32], encoded: &str) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    let data = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        encoded,
    )
    .map_err(|_| CryptoError::Base64Error)?;

    if data.len() < 12 {
        return Err(CryptoError::InvalidCiphertext);
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::CipherInitError)?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptError)?;

    Ok(Zeroizing::new(plaintext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"hello world secret";
        let encrypted = encrypt_field(&key, plaintext).unwrap();
        let decrypted = decrypt_field(&key, &encrypted).unwrap();
        assert_eq!(&*decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_different_nonces() {
        let key = [42u8; 32];
        let plaintext = b"same data";
        let enc1 = encrypt_field(&key, plaintext).unwrap();
        let enc2 = encrypt_field(&key, plaintext).unwrap();
        assert_ne!(enc1, enc2); // Different nonces
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let encrypted = encrypt_field(&key1, b"secret").unwrap();
        assert!(decrypt_field(&key2, &encrypted).is_err());
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let key = [0u8; 32];
        assert!(decrypt_field(&key, "not-valid-base64!!!").is_err());
    }

    #[test]
    fn test_decrypt_too_short() {
        let key = [0u8; 32];
        let short = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [0u8; 5]);
        assert!(decrypt_field(&key, &short).is_err());
    }

    #[test]
    fn test_encrypt_empty_plaintext() {
        let key = [42u8; 32];
        let encrypted = encrypt_field(&key, b"").unwrap();
        let decrypted = decrypt_field(&key, &encrypted).unwrap();
        assert_eq!(&*decrypted, b"");
    }
}
