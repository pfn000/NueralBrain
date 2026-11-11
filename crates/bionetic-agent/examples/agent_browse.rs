#[cfg(feature = "effector-browser-cdp")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bionetic_agent::{AgentBuilder, Capability, PermissionPolicy};
    use bionetic_agent::effectors::{CdpBrowserEffector, browser::BrowserConfig};
    use bionetic_sdk::BrainBuilder;
    use bionetic_core::ids::CorpusId;

    let mut brain = BrainBuilder::new().with_corpora(8).with_dots_per_corpus(128).build();
    let effector = CdpBrowserEffector::new(BrowserConfig {
        url_slots: vec!["https://example.com".into()],
        host_allowlist: vec!["example.com".into()],
        ..Default::default()
    })?;
    let mut agent = AgentBuilder::new()
        .with_effector(effector)
        .policy(PermissionPolicy::deny_all()
            .grant(Capability::BrowserControl)
            .grant(Capability::NetworkHttp))
        .cooldown_ticks(0)
        .build()?;

    let tick = brain.engine().tick;
    if let Some(corpus) = brain.engine_mut().corpora.get_mut(&CorpusId(5)) {
        let mut ids: Vec<_> = corpus.dots.keys().copied().collect();
        ids.sort_unstable();
        for id in ids.into_iter().skip(32).take(16) {
            if let Some(dot) = corpus.dots.get_mut(&id) { dot.fire(tick); }
        }
    }
    let report = agent.tick(brain.engine_mut())?;
    println!("executed={} blocked={}", report.executed, report.blocked);
    Ok(())
}

#[cfg(not(feature = "effector-browser-cdp"))]
fn main() {
    eprintln!("enable the bionetic-agent/effector-browser-cdp feature");
}
