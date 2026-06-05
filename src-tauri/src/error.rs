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

/// 将数据库错误记录到日志并返回用户友好消息
pub fn ipc_db_err(e: sqlx::Error) -> String {
    tracing::error!("数据库错误: {:?}", e);
    "操作失败，请重试".to_string()
}

/// 将加密错误记录到日志并返回用户友好消息
pub fn ipc_crypto_err(e: CryptoError) -> String {
    tracing::error!("加密错误: {:?}", e);
    "操作失败，请重试".to_string()
}

/// 将同步错误记录到日志并返回用户友好消息
pub fn ipc_sync_err(context: &str, e: impl std::fmt::Debug) -> String {
    tracing::error!("{context}: {e:?}");
    "同步操作失败，请重试".to_string()
}

/// 将网络/WebDAV 错误记录到日志并返回用户友好消息
pub fn ipc_network_err(context: &str, e: impl std::fmt::Debug) -> String {
    tracing::error!("{context}: {e:?}");
    "网络连接失败，请检查 WebDAV 配置".to_string()
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
