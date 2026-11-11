# Migration guide — Section 8a-i

Section 8a-i adds the canonical 14-slot corpus layout, pulse provenance and
TRANS direction metadata, a logical cross-corpus slot primitive, and the
shared region-state data model.

## Canonical corpus layout

| ID | Name | Initial state |
|---:|---|---|
| 0 | SensoryIn | Active |
| 1 | SensoryOut | Active |
| 2 | Vision | Active |
| 3 | Prefrontal | Active |
| 4 | Association | Active |
| 5 | RelationalContext | Active |
| 6 | ShortTermMemory | Active |
| 7 | Hippocampus | Active |
| 8 | LongTermMemory | Active |
| 9 | Reward | Active |
| 10 | Motor | Active |
| 11 | Dream | Active |
| 12 | Auditory | Dormant |
| 13 | SpeechMotor | Dormant |

Use the named constants exported by `bionetic_core` rather than numeric IDs.
The existing engine still supports caller-selected corpus counts; the SDK
default is now 14. Reserved regions are allocated by the default configuration
and start dormant when the Section 8a-ii controller is installed.

## Compatibility-preserving additions

- Existing `DotId(u64)`, `CorpusId(u32)`, and the original `Pulse` fields
  remain intact.
- Existing `CorpusType` and its legacy `Sensory` variant remain available.
  A new `CorpusKind` maps canonical semantic regions to named IDs.
- `Pulse` gained `source` and `trans_direction`. Both fields have serde
  defaults so older snapshots decode as External / None.
- New pulse constructors include `new_internal`, `new_external`,
  `new_trans_in`, and `new_trans_out`.
- `SpanDotId` is a logical slot across corpus regions. Since this engine uses
  globally unique DotIds (not a per-corpus index pair), callers resolve its
  slots with `SpanDotId::resolve`; it does not duplicate or alias DotId
  identity. Runtime possession-dot binding is a later section.

## Protocol compatibility

The brain protocol advances from version 1 to version 2. The handshake accepts
v1 peers as well as v2 peers. Old serialized pulses missing the new metadata
fields decode with `PulseSource::External` and `TransDirection::None`.

## Migration notes

- Agent intent decoding and stimulation use `MOTOR`.
- Agent effector feedback and SearchEncoder use `SENSORY_IN`.
- Vision capture examples inject into `VISION`; generic multimodal SDK input
  continues to route into `SENSORY_IN`.
- The SDK default is 14 corpora with 128 initial dots per corpus. Explicit
  configuration through `with_corpora` and `with_dots_per_corpus` remains
  supported.
