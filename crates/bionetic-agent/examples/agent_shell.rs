#[cfg(feature = "effector-shell")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use bionetic_agent::{AgentBuilder, Capability, PermissionPolicy};
    use bionetic_agent::effectors::{ShellEffector, shell::{ShellCommand, ShellConfig}};
    use bionetic_sdk::BrainBuilder;
    use bionetic_core::MOTOR;

    let mut brain = BrainBuilder::new().with_corpora(bionetic_core::CORPUS_COUNT_TOTAL).with_dots_per_corpus(128).build();
    let effector = ShellEffector::new(ShellConfig {
        command_slots: vec![ShellCommand {
            program: "echo".into(),
            args: vec!["hello from NeuralBrain".into()],
            accept_payload_arg: false,
            timeout: Duration::from_secs(2),
            max_output_bytes: 1024,
        }],
    });
    let mut agent = AgentBuilder::new()
        .with_effector(effector)
        .policy(PermissionPolicy::deny_all().grant(Capability::ShellExecute))
        .cooldown_ticks(0)
        .build()?;

    let tick = brain.engine().tick;
    if let Some(corpus) = brain.engine_mut().corpora.get_mut(&MOTOR) {
        let mut ids: Vec<_> = corpus.dots.keys().copied().collect();
        ids.sort_unstable();
        for id in ids.into_iter().skip(16).take(16) {
            if let Some(dot) = corpus.dots.get_mut(&id) { dot.fire(tick); }
        }
    }
    let report = agent.tick(brain.engine_mut())?;
    if let Some(outcome) = report.outcome {
        println!("{}", String::from_utf8_lossy(&outcome.bytes));
    }
    Ok(())
}

#[cfg(not(feature = "effector-shell"))]
fn main() {
    eprintln!("enable the bionetic-agent/effector-shell feature");
}
