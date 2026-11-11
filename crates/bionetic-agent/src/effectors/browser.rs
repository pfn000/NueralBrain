//! Browser navigation by preconfigured URL slot. CDP is optional; Iris is an explicit stub.

use std::time::{Duration, Instant};
use url::Url;

use crate::effector::{ActionOutcome, Effector, EffectorId};
use crate::error::{AgentError, AgentResult};
use crate::intent::{Intent, IntentKind};
use crate::policy::Capability;
use super::payload;

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub url_slots: Vec<String>,
    pub host_allowlist: Vec<String>,
    pub navigate_timeout: Duration,
    pub max_response_bytes: usize,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            url_slots: Vec::new(),
            host_allowlist: Vec::new(),
            navigate_timeout: Duration::from_secs(15),
            max_response_bytes: 128 * 1024,
        }
    }
}

fn selected_url(cfg: &BrowserConfig, intent: &Intent) -> AgentResult<String> {
    let index = payload::slot_index(&intent.payload, cfg.url_slots.len())
        .ok_or_else(|| AgentError::Decode("no browser URL slots or empty payload".into()))?;
    let raw = &cfg.url_slots[index];
    let url = Url::parse(raw).map_err(|error| AgentError::Decode(error.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") || !url.username().is_empty() || url.password().is_some() {
        return Err(AgentError::Decode("browser URLs must be credential-free http/https URLs".into()));
    }
    let host = url.host_str().unwrap_or_default();
    let allowed = cfg.host_allowlist.iter().any(|entry| {
        let domain = entry.trim_start_matches('.').trim_end_matches('.');
        host == domain || host.ends_with(&format!(".{domain}"))
    });
    if !allowed {
        return Err(AgentError::EffectorFailed("browser".into(), format!("host {host} is not allowlisted")));
    }
    Ok(url.to_string())
}

#[cfg(feature = "effector-browser-cdp")]
pub struct CdpBrowserEffector {
    id: EffectorId,
    cfg: BrowserConfig,
    runtime: tokio::runtime::Runtime,
}

#[cfg(feature = "effector-browser-cdp")]
impl CdpBrowserEffector {
    pub fn new(cfg: BrowserConfig) -> AgentResult<Self> {
        if cfg.navigate_timeout.is_zero() {
            return Err(AgentError::Config("browser navigation timeout must be non-zero".into()));
        }
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()
            .map_err(|error| AgentError::Config(format!("tokio runtime: {error}")))?;
        Ok(Self { id: EffectorId::new("browser-cdp"), cfg, runtime })
    }
}

#[cfg(feature = "effector-browser-cdp")]
impl Effector for CdpBrowserEffector {
    fn id(&self) -> EffectorId { self.id.clone() }
    fn handles(&self, intent: &Intent) -> bool {
        matches!(intent.kind, IntentKind::Navigate | IntentKind::Query)
    }
    fn required_capabilities(&self, _intent: &Intent) -> Vec<Capability> {
        vec![Capability::BrowserControl, Capability::NetworkHttp]
    }

    fn execute(&self, intent: &Intent) -> AgentResult<ActionOutcome> {
        use chromiumoxide::{Browser, BrowserConfig as CdpConfig};
        use futures::StreamExt;

        let started = Instant::now();
        let url = selected_url(&self.cfg, intent)?;
        let timeout = self.cfg.navigate_timeout;
        let max_bytes = self.cfg.max_response_bytes;

        let bytes = self.runtime.block_on(async move {
            let config = CdpConfig::builder().build()
                .map_err(|error| AgentError::Config(format!("CDP config: {error}")))?;
            let (mut browser, mut handler) = tokio::time::timeout(timeout, Browser::launch(config)).await
                .map_err(|_| AgentError::Timeout("browser-cdp".into(), timeout.as_millis() as u64))?
                .map_err(|error| AgentError::EffectorFailed("browser-cdp".into(), error.to_string()))?;
            let handler_task = tokio::spawn(async move {
                while handler.next().await.is_some() {}
            });

            let page = tokio::time::timeout(timeout, browser.new_page(url)).await
                .map_err(|_| AgentError::Timeout("browser-cdp".into(), timeout.as_millis() as u64))?
                .map_err(|error| AgentError::EffectorFailed("browser-cdp".into(), error.to_string()))?;
            let content = tokio::time::timeout(timeout, page.content()).await
                .map_err(|_| AgentError::Timeout("browser-cdp".into(), timeout.as_millis() as u64))?
                .map_err(|error| AgentError::EffectorFailed("browser-cdp".into(), error.to_string()))?;

            let _ = browser.close().await;
            handler_task.abort();
            let mut output = content.into_bytes();
            output.truncate(max_bytes);
            Ok::<Vec<u8>, AgentError>(output)
        })?;

        Ok(ActionOutcome::success(bytes, started.elapsed().as_micros() as u64))
    }
}

#[cfg(feature = "effector-browser-iris")]
pub struct IrisBrowserEffector {
    id: EffectorId,
    cfg: BrowserConfig,
}

#[cfg(feature = "effector-browser-iris")]
impl IrisBrowserEffector {
    pub fn new(cfg: BrowserConfig) -> Self {
        Self { id: EffectorId::new("browser-iris"), cfg }
    }
}

#[cfg(feature = "effector-browser-iris")]
impl Effector for IrisBrowserEffector {
    fn id(&self) -> EffectorId { self.id.clone() }
    fn handles(&self, intent: &Intent) -> bool {
        matches!(intent.kind, IntentKind::Navigate | IntentKind::Query)
    }
    fn required_capabilities(&self, _intent: &Intent) -> Vec<Capability> {
        vec![Capability::BrowserControl]
    }
    fn execute(&self, intent: &Intent) -> AgentResult<ActionOutcome> {
        let _ = selected_url(&self.cfg, intent)?;
        Err(AgentError::EffectorFailed(
            self.id.to_string(),
            "Iris backend is not wired: provide the actual Iris crate/API before enabling live browsing".into(),
        ))
    }
}
