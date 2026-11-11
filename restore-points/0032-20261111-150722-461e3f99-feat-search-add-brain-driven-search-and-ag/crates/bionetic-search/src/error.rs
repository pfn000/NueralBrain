use thiserror::Error;

pub type SearchResult<T> = Result<T, SearchError>;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("corpus error: {0}")]
    Corpus(String),
    #[error("index error: {0}")]
    Index(String),
    #[error("unknown concept id: {0}")]
    UnknownConcept(u32),
    #[error("empty query")]
    EmptyQuery,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(feature = "corpus-json")]
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "web")]
    #[error("web retrieval error: {0}")]
    Web(String),
    #[error("configuration error: {0}")]
    Config(String),
}
