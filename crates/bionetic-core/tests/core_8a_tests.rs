use bionetic_core::{
    possession_dot, CorpusId, CorpusKind, DotId, Pulse, PulseSource, PulseType,
    RegionState, SpanDotId, TransDirection, ALL_CORPORA, AUDITORY, CORPUS_COUNT_ACTIVE,
    CORPUS_COUNT_RESERVED, CORPUS_COUNT_TOTAL, MOTOR, RELATIONAL_CONTEXT,
    SENSORY_IN, SENSORY_OUT, SHORT_TERM_MEMORY, SPEECH_MOTOR, VISION,
};

#[test]
fn corpus_constants_match_the_canonical_layout() {
    assert_eq!(SENSORY_IN.0, 0);
    assert_eq!(SENSORY_OUT.0, 1);
    assert_eq!(VISION.0, 2);
    assert_eq!(MOTOR.0, 10);
    assert_eq!(AUDITORY.0, 12);
    assert_eq!(SPEECH_MOTOR.0, 13);
}

#[test]
fn corpus_counts_and_order_are_consistent() {
    assert_eq!(CORPUS_COUNT_ACTIVE, 12);
    assert_eq!(CORPUS_COUNT_RESERVED, 2);
    assert_eq!(CORPUS_COUNT_TOTAL, 14);
    assert_eq!(ALL_CORPORA.len(), CORPUS_COUNT_TOTAL);
    assert_eq!(ALL_CORPORA[0], SENSORY_IN);
    assert_eq!(ALL_CORPORA[13], SPEECH_MOTOR);
}

#[test]
fn corpus_kind_maps_to_named_ids_and_marks_reserved_slots() {
    assert_eq!(CorpusKind::SensoryIn.id(), SENSORY_IN);
    assert_eq!(CorpusKind::SensoryOut.id(), SENSORY_OUT);
    assert_eq!(CorpusKind::Vision.id(), VISION);
    assert_eq!(CorpusKind::Motor.id(), MOTOR);
    assert_eq!(CorpusKind::from_id(MOTOR), Some(CorpusKind::Motor));
    assert_eq!(CorpusKind::from_id(CorpusId(999)), None);
    assert!(CorpusKind::Auditory.is_reserved());
    assert!(CorpusKind::SpeechMotor.is_reserved());
    assert!(!CorpusKind::SensoryIn.is_reserved());
}

#[test]
fn span_dot_retains_logical_slots_without_faking_global_dot_ids() {
    let span = SpanDotId::new(vec![SENSORY_IN, SENSORY_OUT, MOTOR], 42);
    let slots: Vec<_> = span.as_slots().collect();
    assert_eq!(slots, vec![(SENSORY_IN, 42), (SENSORY_OUT, 42), (MOTOR, 42)]);

    let resolved = span.resolve(|corpus, index| {
        assert_eq!(index, 42);
        let _ = corpus;
        Some(DotId::next())
    });
    assert_eq!(resolved.len(), 3);
    assert_ne!(resolved[0], resolved[1]);
}

#[test]
fn possession_dot_spans_the_expected_io_regions() {
    let slot = possession_dot(7);
    assert!(slot.spans(SENSORY_IN));
    assert!(slot.spans(SENSORY_OUT));
    assert!(slot.spans(RELATIONAL_CONTEXT));
    assert!(slot.spans(SHORT_TERM_MEMORY));
    assert!(slot.spans(MOTOR));
    assert!(!slot.spans(VISION));
    assert_eq!(slot.index, 7);
}

#[test]
fn pulse_provenance_is_explicit() {
    let dot = DotId::next();
    let external = Pulse::new_external(dot, 0.5, PulseType::Value, 100);
    let internal = Pulse::new_internal(dot, 0.5, PulseType::Value, 100);
    assert!(external.is_external() && !external.is_internal());
    assert!(internal.is_internal() && !internal.is_external());
}

#[test]
fn trans_pulses_carry_direction_and_source() {
    let dot = DotId::next();
    let inbound = Pulse::new_trans_in(dot, 0.7, 1, PulseSource::External);
    let outbound = Pulse::new_trans_out(dot, 0.7, 1, PulseSource::Internal);
    assert_eq!(inbound.trans_direction, TransDirection::In);
    assert_eq!(outbound.trans_direction, TransDirection::Out);
    assert!(inbound.is_trans());
    assert!(outbound.is_trans());
    assert!(inbound.is_external());
    assert!(outbound.is_internal());
}

#[test]
fn region_state_exposes_tick_and_delivery_policy() {
    assert!(RegionState::Active.is_ticked());
    assert!(RegionState::Resting.is_ticked());
    assert!(!RegionState::Dormant.is_ticked());
    assert!(RegionState::Enhancing.is_ticked());
    assert!(!RegionState::Offline.is_ticked());

    assert_eq!(RegionState::Active.tick_divisor(), 1);
    assert_eq!(RegionState::Resting.tick_divisor(), 10);
    assert_eq!(RegionState::Dormant.tick_divisor(), 0);

    assert!(RegionState::Active.accepts_external_pulses());
    assert!(RegionState::Dormant.accepts_external_pulses());
    assert!(!RegionState::Enhancing.accepts_external_pulses());
    assert!(!RegionState::Offline.accepts_external_pulses());
}
