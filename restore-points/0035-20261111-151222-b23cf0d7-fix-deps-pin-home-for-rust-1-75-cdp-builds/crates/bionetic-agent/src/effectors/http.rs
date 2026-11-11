//! HTTP GET and opt-in POST against pre-registered URL slots and host allowlists.

use std::io::Read;
use std::time::{Duration, Instant};

use url::Url;

use crate::effector::{ActionOutcome, Effector, EffectorId};
use crate::error::{AgentError, AgentResult};
use crate::intent::{Intent, IntentKind};
use crate::policy::Capability;
use super::payload;

#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub host_allowlist: Vec<String>,
    pub url_slots: Vec<String>,
    pub allow_writes: bool,
    pub timeout: Duration,
    pub max_response_bytes: usize,
    pub user_agent: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            host_allowlist: Vec::new(),
            url_slots: Vec::new(),
            allow_writes: false,
            timeout: Duration::from_secs(10),
            max_response_bytes: 64 * 1024,
            user_agent: "NueralBrain/0.2".into(),
        }
    }
}

pub struct HttpEffector {
    id: EffectorId,
    cfg: HttpConfig,
    agent: ureq::Agent,
}

impl HttpEffector {
    pub fn new(cfg: HttpConfig) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(cfg.timeout)
            .timeout_read(cfg.timeout)
            .user_agent(&cfg.user_agent)
            .redirects(0)
            .build();
        Self { id: EffectorId::new("http"), cfg, agent }
    }

    fn selected_url(&self, intent: &Intent) -> AgentResult<Url> {
        let index = payload::slot_index(&intent.payload, self.cfg.url_slots.len())
            .ok_or_else(|| AgentError::Decode("no URL slots or empty payload".into()))?;
        let raw = &self.cfg.url_slots[index];
        let url = Url::parse(raw)
            .map_err(|error| AgentError::Decode(format!("invalid URL slot: {error}")))?;
        if !matches!(url.scheme(), "http" | "https") || !url.username().is_empty() || url.password().is_some() {
            return Err(AgentError::Decode("only credential-free http/https URLs are allowed".into()));
        }
        let host = url.host_str().unwrap_or_default();
        if !host_allowed(host, &self.cfg.host_allowlist) {
            return Err(AgentError::EffectorFailed(
                self.id.to_string(),
                format!("host {host} is not in the allowlist"),
            ));
        }
        Ok(url)
    }
}

impl Effector for HttpEffector {
    fn id(&self) -> EffectorId { self.id.clone() }
    fn handles(&self, intent: &Intent) -> bool {
        matches!(intent.kind, IntentKind::Query | IntentKind::Act)
    }
    fn required_capabilities(&self, _intent: &Intent) -> Vec<Capability> {
        vec![Capability::NetworkHttp]
    }

    fn execute(&self, intent: &Intent) -> AgentResult<ActionOutcome> {
        let started = Instant::now();
        let url = self.selected_url(intent)?;
        let response = if matches!(intent.kind, IntentKind::Act) {
            if !self.cfg.allow_writes {
                return Err(AgentError::EffectorFailed(self.id.to_string(), "HTTP writes are disabled".into()));
            }
            let body = payload::as_text(intent.payload.get(1..).unwrap_or_default()).unwrap_or_default();
            self.agent.post(url.as_str()).send_string(&body)
        } else {
            self.agent.get(url.as_str()).call()
        }.map_err(|error| AgentError::EffectorFailed(self.id.to_string(), error.to_string()))?;

        let status = response.status();
        if !(200..300).contains(&status) {
            return Ok(ActionOutcome::failure(
                format!("HTTP status {status}; redirects are not followed"),
                started.elapsed().as_micros() as u64,
            ));
        }

        let mut bytes = Vec::new();
        response.into_reader()
            .take(self.cfg.max_response_bytes as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| AgentError::EffectorFailed(self.id.to_string(), error.to_string()))?;
        Ok(ActionOutcome::success(bytes, started.elapsed().as_micros() as u64))
    }
}

fn host_allowed(host: &str, allowlist: &[String]) -> bool {
    allowlist.iter().any(|entry| {
        let domain = entry.trim_start_matches('.').trim_end_matches('.');
        host == domain || host.ends_with(&format!(".{domain}"))
    })
}
