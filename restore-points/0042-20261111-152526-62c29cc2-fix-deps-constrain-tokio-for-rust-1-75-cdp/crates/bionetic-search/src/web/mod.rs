#![cfg(feature = "web")]

pub mod extract;
pub mod fetcher;

pub use extract::extract_text;
pub use fetcher::{WebFetcher, WebFetcherConfig};
