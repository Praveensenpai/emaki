use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmakiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Downloader error: {0}")]
    Downloader(String),

    #[error("WhatsApp client error: {0}")]
    WhatsApp(String),
}

pub type Result<T> = std::result::Result<T, EmakiError>;
