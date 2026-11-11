use bionetic_agent::{AgentBuilder, Capability, PermissionPolicy};
use bionetic_core::ids::CorpusId;
use bionetic_sdk::BrainBuilder;

fn stimulate(engine: &mut bionetic_engine::engine::BioneticEngine, offset: usize, size: usize) {
    let tick = engine.tick;
    if let Some(corpus) = engine.corpora.get_mut(&CorpusId(5)) {
        let mut ids: Vec<_> = corpus.dots.keys().copied().collect();
        ids.sort_unstable();
        for id in ids.into_iter().skip(offset).take(size) {
            if let Some(dot) = corpus.dots.get_mut(&id) { dot.fire(tick); }
        }
    }
}

#[cfg(feature = "effector-shell")]
#[test]
fn shell_echo_runs_only_when_granted() {
    use std::time::Duration;
    use bionetic_agent::effectors::{ShellEffector, shell::{ShellCommand, ShellConfig}};
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let config = ShellConfig { command_slots: vec![ShellCommand {
        program: "echo".into(), args: vec!["ok".into()], accept_payload_arg: false,
        timeout: Duration::from_secs(2), max_output_bytes: 64,
    }] };
    let mut agent = AgentBuilder::new()
        .with_effector(ShellEffector::new(config))
        .policy(PermissionPolicy::deny_all().grant(Capability::ShellExecute))
        .cooldown_ticks(0).build().unwrap();
    stimulate(brain.engine_mut(), 16, 16);
    let report = agent.tick(brain.engine_mut()).unwrap();
    assert!(report.executed);
    assert!(String::from_utf8_lossy(&report.outcome.unwrap().bytes).contains("ok"));
}

#[cfg(feature = "effector-shell")]
#[test]
fn shell_is_blocked_without_capability() {
    use std::time::Duration;
    use bionetic_agent::effectors::{ShellEffector, shell::{ShellCommand, ShellConfig}};
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let config = ShellConfig { command_slots: vec![ShellCommand {
        program: "echo".into(), args: vec!["must-not-run".into()], accept_payload_arg: false,
        timeout: Duration::from_secs(2), max_output_bytes: 64,
    }] };
    let mut agent = AgentBuilder::new()
        .with_effector(ShellEffector::new(config))
        .policy(PermissionPolicy::deny_all())
        .cooldown_ticks(0).build().unwrap();
    stimulate(brain.engine_mut(), 16, 16);
    let report = agent.tick(brain.engine_mut()).unwrap();
    assert!(report.blocked);
}

#[cfg(feature = "effector-fs")]
#[test]
fn filesystem_reads_only_inside_jail() {
    use std::fs;
    use bionetic_agent::effectors::{FsEffector, filesystem::FsConfig};
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("inside.txt"), b"hello").unwrap();
    let config = FsConfig {
        root: temp.path().to_path_buf(),
        path_slots: vec!["inside.txt".into()],
        allow_write: false,
        max_bytes: 1024,
    };
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(FsEffector::new(config))
        .policy(PermissionPolicy::deny_all()
            .grant(Capability::FilesystemRead(temp.path().to_string_lossy().into_owned())))
        .cooldown_ticks(0).build().unwrap();
    stimulate(brain.engine_mut(), 0, 16);
    let report = agent.tick(brain.engine_mut()).unwrap();
    assert_eq!(report.outcome.unwrap().bytes, b"hello");
}

#[cfg(feature = "effector-fs")]
#[test]
fn filesystem_rejects_parent_traversal() {
    use bionetic_agent::effectors::{FsEffector, filesystem::FsConfig};
    let temp = tempfile::tempdir().unwrap();
    let config = FsConfig {
        root: temp.path().to_path_buf(),
        path_slots: vec!["../outside.txt".into()],
        allow_write: false,
        max_bytes: 1024,
    };
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(FsEffector::new(config))
        .policy(PermissionPolicy::deny_all()
            .grant(Capability::FilesystemRead(temp.path().to_string_lossy().into_owned())))
        .cooldown_ticks(0).build().unwrap();
    stimulate(brain.engine_mut(), 0, 16);
    assert!(agent.tick(brain.engine_mut()).is_err());
}

#[cfg(feature = "effector-code")]
#[test]
fn code_emit_writes_selected_template() {
    use bionetic_agent::effectors::{CodeEffector, code_emit::{CodeConfig, CodeTemplate}};
    let temp = tempfile::tempdir().unwrap();
    let config = CodeConfig {
        project_root: temp.path().to_path_buf(),
        templates: vec![CodeTemplate {
            name: "hello-rs".into(),
            body: "fn main() { println!(\"{{PARAM}}\"); }\n".into(),
            ext: "rs".into(),
        }],
        name_slots: vec!["hello".into()],
        max_param_bytes: 64,
    };
    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let mut agent = AgentBuilder::new()
        .with_effector(CodeEffector::new(config))
        .policy(PermissionPolicy::deny_all()
            .grant(Capability::CodeEmit)
            .grant(Capability::FilesystemWrite(temp.path().to_string_lossy().into_owned())))
        .cooldown_ticks(0).build().unwrap();
    stimulate(brain.engine_mut(), 48, 16);
    let report = agent.tick(brain.engine_mut()).unwrap();
    assert!(report.executed);
    assert!(std::fs::read_to_string(temp.path().join("hello.rs")).unwrap().contains("fn main"));
}
