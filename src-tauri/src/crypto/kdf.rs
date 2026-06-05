use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::error::CryptoError;

const DB_KEY_INFO: &[u8] = b"keyvault-v3-sqlcipher";

/// Argon2id 参数锁定
const MEMORY_COST: u32 = 65536;
const TIME_COST: u32 = 3;
const PARALLELISM: u32 = 1;
const OUTPUT_LEN: usize = 32;

/// 派生加密密钥（32 字节），用于 AES-256-GCM
pub fn derive_key(password: &[u8], salt: &[u8; 16]) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let params =
        Params::new(MEMORY_COST, TIME_COST, PARALLELISM, Some(OUTPUT_LEN)).map_err(|_| CryptoError::KdfParamError)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(password, salt, key.as_mut())
        .map_err(|_| CryptoError::KdfError)?;

    Ok(key)
}

/// 哈希主密码用于验证存储（使用 password_hash 格式，包含随机 salt）
pub fn hash_master_password(password: &[u8]) -> Result<String, CryptoError> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(MEMORY_COST, TIME_COST, PARALLELISM, None).map_err(|_| CryptoError::KdfParamError)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let hash = argon2
        .hash_password(password, &salt)
        .map_err(|_| CryptoError::HashError)?
        .to_string();
    Ok(hash)
}

/// 从字段加密密钥派生 SQLCipher 数据库密钥（HKDF 子密钥）
pub fn derive_db_key(field_key: &[u8; 32]) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, field_key);
    let mut db_key = Zeroizing::new([0u8; 32]);
    hk.expand(DB_KEY_INFO, db_key.as_mut())
        .map_err(|_| CryptoError::KdfError)?;
    Ok(db_key)
}

/// 验证主密码
pub fn verify_master_password(password: &[u8], hash: &str) -> Result<bool, CryptoError> {
    let parsed = PasswordHash::new(hash).map_err(|_| CryptoError::HashParseError)?;
    Ok(Argon2::default()
        .verify_password(password, &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let password = b"test_password_123";
        let salt = [0u8; 16];
        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();
        assert_eq!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = [0u8; 16];
        let key1 = derive_key(b"password_a", &salt).unwrap();
        let key2 = derive_key(b"password_b", &salt).unwrap();
        assert_ne!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_derive_key_different_salts() {
        let password = b"same_password";
        let salt1 = [0u8; 16];
        let salt2 = [1u8; 16];
        let key1 = derive_key(password, &salt1).unwrap();
        let key2 = derive_key(password, &salt2).unwrap();
        assert_ne!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_derive_key_length() {
        let key = derive_key(b"test", &[0u8; 16]).unwrap();
        assert_eq!(key.as_ref().len(), 32);
    }

    #[test]
    fn test_hash_and_verify() {
        let password = b"my_secure_password";
        let hash = hash_master_password(password).unwrap();
        assert!(verify_master_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = b"correct_password";
        let hash = hash_master_password(password).unwrap();
        assert!(!verify_master_password(b"wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_hash_not_deterministic() {
        let password = b"same_password";
        let hash1 = hash_master_password(password).unwrap();
        let hash2 = hash_master_password(password).unwrap();
        assert_ne!(hash1, hash2); // Different random salts
    }

    /// §九验收：Argon2id 单次派生应 ≥ 1s（抗暴力破解）
    #[test]
    fn test_derive_key_meets_minimum_duration() {
        use std::time::{Duration, Instant};
        let start = Instant::now();
        derive_key(b"benchmark_password_for_timing_test", &[0u8; 16]).unwrap();
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(900),
            "Argon2 derive_key 过快（{elapsed:?}），参数可能被降级"
        );
    }
}
