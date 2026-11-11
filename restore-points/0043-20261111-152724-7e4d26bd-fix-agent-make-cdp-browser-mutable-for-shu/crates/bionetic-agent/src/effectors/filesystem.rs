//! Filesystem effector jailed to a configured root. Relative paths only; symlinks are refused.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::Instant;

use crate::effector::{ActionOutcome, Effector, EffectorId};
use crate::error::{AgentError, AgentResult};
use crate::intent::{Intent, IntentKind};
use crate::policy::Capability;
use super::payload;

#[derive(Debug, Clone)]
pub struct FsConfig {
    pub root: PathBuf,
    pub path_slots: Vec<PathBuf>,
    pub allow_write: bool,
    pub max_bytes: usize,
}

impl FsConfig {
    pub fn read_only(root: PathBuf) -> Self {
        Self { root, path_slots: Vec::new(), allow_write: false, max_bytes: 64 * 1024 }
    }
}

pub struct FsEffector {
    id: EffectorId,
    cfg: FsConfig,
}

impl FsEffector {
    pub fn new(cfg: FsConfig) -> Self { Self { id: EffectorId::new("fs"), cfg } }

    fn resolve_safe(&self, relative: &Path) -> AgentResult<PathBuf> {
        if relative.is_absolute() || relative.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        }) {
            return Err(AgentError::EffectorFailed(self.id.to_string(), "absolute paths and parent traversal are refused".into()));
        }

        let root = self.cfg.root.canonicalize()
            .map_err(|error| AgentError::EffectorFailed(self.id.to_string(), format!("invalid jail root: {error}")))?;
        let mut candidate = root.clone();

        for component in relative.components() {
            if let Component::Normal(part) = component {
                candidate.push(part);
                match fs::symlink_metadata(&candidate) {
                    Ok(metadata) => {
                        if metadata.file_type().is_symlink() {
                            return Err(AgentError::EffectorFailed(self.id.to_string(), "symlink component refused".into()));
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(AgentError::EffectorFailed(self.id.to_string(), error.to_string())),
                }
            }
        }

        let check = if candidate.exists() {
            candidate.canonicalize()
        } else {
            candidate.parent().unwrap_or(&root).canonicalize()
        }.map_err(|error| AgentError::EffectorFailed(self.id.to_string(), error.to_string()))?;

        if !check.starts_with(&root) {
            return Err(AgentError::EffectorFailed(self.id.to_string(), "path escapes jail".into()));
        }
        Ok(candidate)
    }
}

impl Effector for FsEffector {
    fn id(&self) -> EffectorId { self.id.clone() }
    fn handles(&self, intent: &Intent) -> bool {
        matches!(intent.kind, IntentKind::Query | IntentKind::Act)
    }

    fn required_capabilities(&self, intent: &Intent) -> Vec<Capability> {
        let root = self.cfg.root.to_string_lossy().into_owned();
        match intent.kind {
            IntentKind::Query => vec![Capability::FilesystemRead(root)],
            IntentKind::Act => vec![Capability::FilesystemWrite(root)],
            _ => Vec::new(),
        }
    }

    fn execute(&self, intent: &Intent) -> AgentResult<ActionOutcome> {
        let started = Instant::now();
        let index = payload::slot_index(&intent.payload, self.cfg.path_slots.len())
            .ok_or_else(|| AgentError::EffectorFailed(self.id.to_string(), "no path slots".into()))?;
        let path = self.resolve_safe(&self.cfg.path_slots[index])?;

        match intent.kind {
            IntentKind::Query => {
                let mut bytes = Vec::new();
                File::open(&path)
                    .and_then(|file| file.take(self.cfg.max_bytes as u64).read_to_end(&mut bytes))
                    .map_err(|error| AgentError::EffectorFailed(self.id.to_string(), error.to_string()))?;
                Ok(ActionOutcome::success(bytes, started.elapsed().as_micros() as u64))
            }
            IntentKind::Act => {
                if !self.cfg.allow_write {
                    return Err(AgentError::EffectorFailed(self.id.to_string(), "filesystem writes are disabled".into()));
                }
                let bytes = intent.payload.get(1..).unwrap_or_default()
                    .iter().copied().take(self.cfg.max_bytes).collect::<Vec<_>>();
                fs::write(&path, &bytes)
                    .map_err(|error| AgentError::EffectorFailed(self.id.to_string(), error.to_string()))?;
                Ok(ActionOutcome::success(
                    format!("wrote {} bytes", bytes.len()).into_bytes(),
                    started.elapsed().as_micros() as u64,
                ))
            }
            _ => Err(AgentError::Decode("unsupported filesystem intent".into())),
        }
    }
}
