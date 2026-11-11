//! Template-based source emission into a preconfigured project directory.
//! This is curated template selection, not unconstrained code generation.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::effector::{ActionOutcome, Effector, EffectorId};
use crate::error::{AgentError, AgentResult};
use crate::intent::{Intent, IntentKind};
use crate::policy::Capability;
use super::payload;

#[derive(Debug, Clone)]
pub struct CodeTemplate {
    pub name: String,
    pub body: String,
    pub ext: String,
}

#[derive(Debug, Clone)]
pub struct CodeConfig {
    pub project_root: PathBuf,
    pub templates: Vec<CodeTemplate>,
    pub name_slots: Vec<String>,
    pub max_param_bytes: usize,
}

pub struct CodeEffector {
    id: EffectorId,
    cfg: CodeConfig,
}

impl CodeEffector {
    pub fn new(cfg: CodeConfig) -> Self { Self { id: EffectorId::new("code"), cfg } }
}

impl Effector for CodeEffector {
    fn id(&self) -> EffectorId { self.id.clone() }
    fn handles(&self, intent: &Intent) -> bool { matches!(intent.kind, IntentKind::Emit) }
    fn required_capabilities(&self, _intent: &Intent) -> Vec<Capability> {
        vec![
            Capability::CodeEmit,
            Capability::FilesystemWrite(self.cfg.project_root.to_string_lossy().into_owned()),
        ]
    }

    fn execute(&self, intent: &Intent) -> AgentResult<ActionOutcome> {
        let started = Instant::now();
        if self.cfg.templates.is_empty() || self.cfg.name_slots.is_empty() {
            return Err(AgentError::Config("no code templates or destination slots configured".into()));
        }
        if intent.payload.len() < 2 {
            return Err(AgentError::Decode("code-emission payload needs template and name slots".into()));
        }

        let template_index = payload::slot_index(&intent.payload[..1], self.cfg.templates.len()).unwrap_or(0);
        let name_index = payload::slot_index(&intent.payload[1..2], self.cfg.name_slots.len()).unwrap_or(0);
        let template = &self.cfg.templates[template_index];
        let name = &self.cfg.name_slots[name_index];

        if !safe_component(name) || !safe_component(&template.ext) {
            return Err(AgentError::Config("destination name and extension must be simple filename components".into()));
        }
        let root = self.cfg.project_root.canonicalize()
            .map_err(|error| AgentError::EffectorFailed(self.id.to_string(), error.to_string()))?;
        let path = root.join(format!("{name}.{}", template.ext));
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if metadata.file_type().is_symlink() {
                return Err(AgentError::EffectorFailed(self.id.to_string(), "destination symlink refused".into()));
            }
        }

        let raw = payload::as_text(intent.payload.get(2..).unwrap_or_default()).unwrap_or_default();
        let parameter: String = raw.chars()
            .filter(|character| character.is_ascii_graphic() || *character == ' ')
            .take(self.cfg.max_param_bytes)
            .collect();
        let body = template.body.replace("{{PARAM}}", &parameter);
        fs::write(&path, body.as_bytes())
            .map_err(|error| AgentError::EffectorFailed(self.id.to_string(), error.to_string()))?;

        Ok(ActionOutcome::success(
            format!("emitted {} ({} bytes)", path.display(), body.len()).into_bytes(),
            started.elapsed().as_micros() as u64,
        ))
    }
}

fn safe_component(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| {
        character.is_ascii_alphanumeric() || character == '_' || character == '-'
    })
}

#[allow(dead_code)]
fn _path_type(_: &Path) {}
