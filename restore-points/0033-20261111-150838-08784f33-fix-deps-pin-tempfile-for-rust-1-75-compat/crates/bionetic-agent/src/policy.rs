use std::collections::HashSet;
use std::path::Path;

use crate::intent::{Intent, IntentKind};

/// Named abilities required by effectors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Capability {
    NetworkHttp,
    NetworkWebSocket,
    BrowserControl,
    ShellExecute,
    FilesystemRead(String),
    FilesystemWrite(String),
    CodeEmit,
    ClipboardRead,
    ClipboardWrite,
    Custom(String),
}

impl Capability {
    pub fn as_label(&self) -> String {
        match self {
            Self::NetworkHttp => "network.http".into(),
            Self::NetworkWebSocket => "network.ws".into(),
            Self::BrowserControl => "browser.control".into(),
            Self::ShellExecute => "shell.execute".into(),
            Self::FilesystemRead(path) => format!("fs.read:{path}"),
            Self::FilesystemWrite(path) => format!("fs.write:{path}"),
            Self::CodeEmit => "code.emit".into(),
            Self::ClipboardRead => "clipboard.read".into(),
            Self::ClipboardWrite => "clipboard.write".into(),
            Self::Custom(name) => format!("custom:{name}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

/// Explicit policy. Default is deny-all; an optional predicate can veto grants.
pub struct PermissionPolicy {
    granted: HashSet<String>,
    predicate: Option<Box<dyn Fn(&Intent, &Capability) -> Decision + Send + Sync>>,
    read_only: bool,
}

impl Default for PermissionPolicy {
    fn default() -> Self { Self::deny_all() }
}

impl PermissionPolicy {
    pub fn deny_all() -> Self {
        Self { granted: HashSet::new(), predicate: None, read_only: false }
    }

    pub fn allow_all() -> Self {
        let mut policy = Self::deny_all();
        policy.granted.insert("*".into());
        policy
    }

    pub fn grant(mut self, capability: Capability) -> Self {
        self.granted.insert(capability.as_label());
        self
    }

    pub fn grant_many<I: IntoIterator<Item = Capability>>(mut self, capabilities: I) -> Self {
        for capability in capabilities { self.granted.insert(capability.as_label()); }
        self
    }

    pub fn with_predicate<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&Intent, &Capability) -> Decision + Send + Sync + 'static,
    {
        self.predicate = Some(Box::new(predicate));
        self
    }

    pub fn read_only(mut self, enabled: bool) -> Self {
        self.read_only = enabled;
        self
    }

    pub fn is_granted(&self, capability: &Capability) -> bool {
        if self.granted.contains("*") { return true; }
        let label = capability.as_label();
        if self.granted.contains(&label) { return true; }

        match capability {
            Capability::FilesystemRead(path) => self.granted.iter().any(|grant| {
                grant.strip_prefix("fs.read:")
                    .map(|prefix| Path::new(path).starts_with(Path::new(prefix)))
                    .unwrap_or(false)
            }),
            Capability::FilesystemWrite(path) => self.granted.iter().any(|grant| {
                grant.strip_prefix("fs.write:")
                    .map(|prefix| Path::new(path).starts_with(Path::new(prefix)))
                    .unwrap_or(false)
            }),
            _ => false,
        }
    }

    pub fn evaluate(&self, intent: &Intent, capability: &Capability) -> Decision {
        if self.read_only && matches!(intent.kind, IntentKind::Act) {
            return Decision::Deny;
        }
        if !self.is_granted(capability) {
            return Decision::Deny;
        }
        self.predicate.as_ref()
            .map(|predicate| predicate(intent, capability))
            .unwrap_or(Decision::Allow)
    }
}
