use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("KDF 参数错误")]
    KdfParamError,
    #[error("密钥派生失败")]
    KdfError,
    #[error("密码哈希失败")]
    HashError,
    #[error("哈希解析失败")]
    HashParseError,
    #[error("加密初始化失败")]
    CipherInitError,
    #[error("加密失败")]
    EncryptError,
    #[error("解密失败")]
    DecryptError,
    #[error("Base64 编解码失败")]
    Base64Error,
    #[error("无效的密文")]
    InvalidCiphertext,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误")]
    Database(String),
    #[error("加密错误: {0}")]
    Crypto(#[from] CryptoError),
    #[error("会话已过期")]
    SessionExpired,
    #[error("密码管理器已锁定")]
    Locked,
    #[error("密码管理器未初始化")]
    NotInitialized,
    #[error("密码错误")]
    WrongPassword,
    #[error("网络请求失败: {0}")]
    Network(String),
    #[error("{0}")]
    BadRequest(String),
}

impl From<sqlx::Error> for AppError {
    fn from(_e: sqlx::Error) -> Self {
        // 不泄露 SQL 细节
        AppError::Database("数据库操作失败".to_string())
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
