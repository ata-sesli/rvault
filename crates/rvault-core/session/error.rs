use thiserror::Error;

/// Failures returned by typed session-key access.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SessionError {
    #[error("vault is locked")]
    Locked,
    #[error("session expired")]
    Expired,
    #[error("invalid session token")]
    InvalidToken,
    #[error("corrupt session data")]
    CorruptSession,
    #[error("session I/O error")]
    Io(#[from] std::io::Error),
}
