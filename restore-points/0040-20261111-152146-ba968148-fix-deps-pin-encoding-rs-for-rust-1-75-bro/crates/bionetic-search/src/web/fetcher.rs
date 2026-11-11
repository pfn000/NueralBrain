#![cfg(feature = "web")]

use std::io::Read;
use std::time::Duration;
use url::Url;

use crate::error::{SearchError, SearchResult};

#[derive(Debug, Clone)]
pub struct WebFetcherConfig {
    pub host_allowlist: Vec<String>,
    pub timeout: Duration,
    pub max_bytes: usize,
    pub user_agent: String,
}

impl Default for WebFetcherConfig {
    fn default() -> Self {
        Self {
            host_allowlist: Vec::new(),
            timeout: Duration::from_secs(10),
            max_bytes: 256 * 1024,
            user_agent: "NueralBrain-Search/0.2".into(),
        }
    }
}

pub struct WebFetcher {
    config: WebFetcherConfig,
    agent: ureq::Agent,
}

impl WebFetcher {
    pub fn new(config: WebFetcherConfig) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(config.timeout)
            .timeout_read(config.timeout)
            .user_agent(&config.user_agent)
            .redirects(0)
            .build();
        Self { config, agent }
    }

    pub fn fetch(&self, address: &str) -> SearchResult<String> {
        let url = Url::parse(address).map_err(|error| SearchError::Web(error.to_string()))?;
        if !matches!(url.scheme(), "http" | "https") || !url.username().is_empty() || url.password().is_some() {
            return Err(SearchError::Web("only credential-free http/https URLs are allowed".into()));
        }
        let host = url.host_str().ok_or_else(|| SearchError::Web("URL has no host".into()))?;
        let allowed = self.config.host_allowlist.iter().any(|entry| {
            let domain = entry.trim_start_matches('.').trim_end_matches('.');
            host == domain || host.ends_with(&format!(".{domain}"))
        });
        if !allowed {
            return Err(SearchError::Web(format!("host {host} is not allowlisted")));
        }

        let response = self.agent.get(url.as_str()).call()
            .map_err(|error| SearchError::Web(error.to_string()))?;
        let status = response.status();
        if !(200..300).contains(&status) {
            return Err(SearchError::Web(format!("HTTP {status}; redirects are not followed")));
        }

        let mut bytes = Vec::new();
        response.into_reader().take(self.config.max_bytes as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| SearchError::Web(error.to_string()))?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}
