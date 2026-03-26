#[derive(Debug, Clone)]
pub struct HttpError(pub String);

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for HttpError {}

impl From<reqwest::Error> for HttpError {
    fn from(e: reqwest::Error) -> Self {
        HttpError(e.to_string())
    }
}

impl From<vastrum_shared_types::compression::brotli::BrotliError> for HttpError {
    fn from(e: vastrum_shared_types::compression::brotli::BrotliError) -> Self {
        HttpError(e.to_string())
    }
}
