//! Shell effector. Runs only preconfigured executables and fixed argument lists.
//! No shell is spawned, so payload data is never interpreted as shell syntax.

use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::effector::{ActionOutcome, Effector, EffectorId};
use crate::error::{AgentError, AgentResult};
use crate::intent::{Intent, IntentKind};
use crate::policy::Capability;
use super::payload;

#[derive(Debug, Clone)]
pub struct ShellCommand {
    pub program: String,
    pub args: Vec<String>,
    pub accept_payload_arg: bool,
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ShellConfig {
    pub command_slots: Vec<ShellCommand>,
}

pub struct ShellEffector {
    id: EffectorId,
    cfg: ShellConfig,
}

impl ShellEffector {
    pub fn new(cfg: ShellConfig) -> Self {
        Self { id: EffectorId::new("shell"), cfg }
    }
}

impl Effector for ShellEffector {
    fn id(&self) -> EffectorId { self.id.clone() }
    fn handles(&self, intent: &Intent) -> bool { matches!(intent.kind, IntentKind::Act) }
    fn required_capabilities(&self, _intent: &Intent) -> Vec<Capability> {
        vec![Capability::ShellExecute]
    }

    fn execute(&self, intent: &Intent) -> AgentResult<ActionOutcome> {
        let started = Instant::now();
        let index = payload::slot_index(&intent.payload, self.cfg.command_slots.len())
            .ok_or_else(|| AgentError::EffectorFailed(self.id.to_string(), "no command slots".into()))?;
        let spec = &self.cfg.command_slots[index];
        if spec.timeout.is_zero() {
            return Err(AgentError::Config("shell timeout must be non-zero".into()));
        }

        let mut command = Command::new(&spec.program);
        command.args(&spec.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if spec.accept_payload_arg {
            if let Some(text) = payload::as_text(intent.payload.get(1..).unwrap_or_default()) {
                let cleaned: String = text.chars().filter(|character| !character.is_control()).collect();
                if !cleaned.is_empty() { command.arg(cleaned); }
            }
        }

        let mut child = command.spawn()
            .map_err(|error| AgentError::EffectorFailed(self.id.to_string(), format!("spawn failed: {error}")))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| AgentError::EffectorFailed(self.id.to_string(), "stdout pipe unavailable".into()))?;
        let stderr = child.stderr.take()
            .ok_or_else(|| AgentError::EffectorFailed(self.id.to_string(), "stderr pipe unavailable".into()))?;
        let max = spec.max_output_bytes;
        let stdout_reader = thread::spawn(move || read_bounded(stdout, max));
        let stderr_reader = thread::spawn(move || read_bounded(stderr, max));

        let deadline = Instant::now() + spec.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(AgentError::Timeout(
                        self.id.to_string(),
                        spec.timeout.as_millis() as u64,
                    ));
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(AgentError::EffectorFailed(self.id.to_string(), error.to_string()));
                }
            }
        };

        let mut output = stdout_reader.join()
            .map_err(|_| AgentError::EffectorFailed(self.id.to_string(), "stdout reader panicked".into()))?
            .map_err(|error| AgentError::EffectorFailed(self.id.to_string(), error.to_string()))?;
        let stderr = stderr_reader.join()
            .map_err(|_| AgentError::EffectorFailed(self.id.to_string(), "stderr reader panicked".into()))?
            .map_err(|error| AgentError::EffectorFailed(self.id.to_string(), error.to_string()))?;
        if output.is_empty() { output = stderr; }

        let elapsed = started.elapsed().as_micros() as u64;
        if !status.success() {
            return Ok(ActionOutcome::failure(
                format!("process exited with {}", status.code().unwrap_or(-1)),
                elapsed,
            ));
        }
        Ok(ActionOutcome::success(output, elapsed))
    }
}

/// Drain the complete pipe to prevent child-process deadlocks, retaining only a bounded prefix.
fn read_bounded<R: Read>(mut reader: R, limit: usize) -> io::Result<Vec<u8>> {
    let mut output = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 { break; }
        let keep = limit.saturating_sub(output.len()).min(count);
        output.extend_from_slice(&buffer[..keep]);
    }
    Ok(output)
}
