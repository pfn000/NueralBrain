//! # Brain-to-Brain Communication Protocol
//!
//! Lets multiple NeuralBrain instances form a network and share pulse
//! traffic in real time. The protocol is transport-agnostic — it emits
//! and consumes byte buffers which you ship over your choice of channel
//! (WebSocket, WebRTC, TCP, shared memory, HuggingFace Space endpoints).
//!
//! ## Message flow
//!
//! ```text
//!   Brain A                            Brain B
//!     │                                   │
//!     │  Hello(id_a, corpora, tick_rate)  │
//!     │──────────────────────────────────▶│
//!     │          Ack(id_b)                │
//!     │◀──────────────────────────────────│
//!     │                                   │
//!     │  PulseBundle(from=a, corpus=2)    │
//!     │──────────────────────────────────▶│  → injected into corpus 2
//!     │                                   │
//!     │  SyncRequest(tick=1234)           │
//!     │──────────────────────────────────▶│
//!     │   SyncSnapshot(tick, dots, chem)  │
//!     │◀──────────────────────────────────│
//!     │                                   │
//!     │  Goodbye(id_a)                    │
//!     │──────────────────────────────────▶│
//! ```
//!
//! ## Guarantees
//! - **Idempotent handshake** — duplicate `Hello` messages are safe.
//! - **Versioned** — `PROTOCOL_VERSION` lets peers reject mismatched builds.
//! - **No ordering assumptions** — pulses carry their own `tick` number.
//! - **No encryption** — wrap the transport (TLS, WSS) for security.

use bionetic_core::chemicals::ChemicalSnapshot;
use bionetic_core::ids::CorpusId;
use bionetic_core::pulse::Pulse;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// Current wire-protocol version. Bump on breaking changes.
pub const PROTOCOL_VERSION: u16 = 1;

/// Maximum pulses allowed in a single `PulseBundle` message.
/// Larger payloads must be split into multiple bundles.
pub const MAX_PULSES_PER_BUNDLE: usize = 4096;

// ──────────────────────────────────────────────
// Brain identity
// ──────────────────────────────────────────────

/// Globally unique identifier for a brain instance in a network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrainId(pub [u8; 16]);

impl BrainId {
    /// Generate a fresh random brain ID.
    pub fn new() -> Self {
        Self(*Uuid::new_v4().as_bytes())
    }

    /// Construct from raw bytes (for persistence across restarts).
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Hex string representation for logging.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Short 8-char hex prefix for human display.
    pub fn short(&self) -> String {
        self.to_hex()[..8].to_string()
    }
}

impl Default for BrainId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BrainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Brain({})", self.short())
    }
}

// ──────────────────────────────────────────────
// Wire-format messages
// ──────────────────────────────────────────────

/// All message types exchanged between brains.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BrainMessage {
    /// Handshake initiation: announce identity + capabilities.
    Hello {
        protocol_version: u16,
        brain_id: BrainId,
        corpora: Vec<CorpusId>,
        tick_rate_hz: u32,
        features: Vec<String>, // e.g. "vision", "swarm", "dream"
    },

    /// Handshake response.
    Ack {
        protocol_version: u16,
        brain_id: BrainId,
        accepted: bool,
        reason: Option<String>,
    },

    /// Keep-alive ping (optional, transport-dependent).
    Ping {
        brain_id: BrainId,
        tick: u64,
    },

    /// Keep-alive response.
    Pong {
        brain_id: BrainId,
        tick: u64,
    },

    /// A batch of pulses crossing from one brain into another's corpus.
    PulseBundle {
        from_brain: BrainId,
        target_corpus: CorpusId,
        pulses: Vec<Pulse>,
        source_tick: u64,
    },

    /// Request a state snapshot (for new peers or resync).
    SyncRequest {
        from_brain: BrainId,
        at_tick: u64,
    },

    /// Snapshot response. Minimal — just metadata, not the full state.
    SyncSnapshot {
        from_brain: BrainId,
        tick: u64,
        total_dots: usize,
        active_corpora: Vec<CorpusId>,
        chemicals: ChemicalSnapshot,
    },

    /// Graceful disconnect.
    Goodbye {
        brain_id: BrainId,
        reason: Option<String>,
    },

    /// Error / protocol violation report.
    Error {
        from_brain: BrainId,
        code: u16,
        message: String,
    },
}

impl BrainMessage {
    /// Returns the sender's brain ID, if the message carries one.
    pub fn sender(&self) -> Option<BrainId> {
        match self {
            BrainMessage::Hello { brain_id, .. }
            | BrainMessage::Ack { brain_id, .. }
            | BrainMessage::Ping { brain_id, .. }
            | BrainMessage::Pong { brain_id, .. }
            | BrainMessage::Goodbye { brain_id, .. } => Some(*brain_id),
            BrainMessage::PulseBundle { from_brain, .. }
            | BrainMessage::SyncRequest { from_brain, .. }
            | BrainMessage::SyncSnapshot { from_brain, .. }
            | BrainMessage::Error { from_brain, .. } => Some(*from_brain),
        }
    }

    /// Human-readable type name for logging.
    pub fn type_name(&self) -> &'static str {
        match self {
            BrainMessage::Hello { .. } => "Hello",
            BrainMessage::Ack { .. } => "Ack",
            BrainMessage::Ping { .. } => "Ping",
            BrainMessage::Pong { .. } => "Pong",
            BrainMessage::PulseBundle { .. } => "PulseBundle",
            BrainMessage::SyncRequest { .. } => "SyncRequest",
            BrainMessage::SyncSnapshot { .. } => "SyncSnapshot",
            BrainMessage::Goodbye { .. } => "Goodbye",
            BrainMessage::Error { .. } => "Error",
        }
    }
}

// ──────────────────────────────────────────────
// Codec
// ──────────────────────────────────────────────

/// Codec errors.
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("message exceeds max pulse batch ({0} > {max})", max = MAX_PULSES_PER_BUNDLE)]
    BundleTooLarge(usize),

    #[error("protocol version mismatch: local={local}, remote={remote}")]
    VersionMismatch { local: u16, remote: u16 },
}

/// Encoder/decoder for brain messages.
///
/// Default implementation uses JSON for human-readability during development.
/// For production, swap in CBOR or MessagePack by implementing your own codec.
pub struct BrainCodec;

impl BrainCodec {
    /// Encode a message to bytes for the transport layer.
    pub fn encode(msg: &BrainMessage) -> Result<Vec<u8>, CodecError> {
        // Enforce max-pulse limit before serialization
        if let BrainMessage::PulseBundle { pulses, .. } = msg {
            if pulses.len() > MAX_PULSES_PER_BUNDLE {
                return Err(CodecError::BundleTooLarge(pulses.len()));
            }
        }
        Ok(serde_json::to_vec(msg)?)
    }

    /// Decode bytes from the transport layer into a message.
    pub fn decode(bytes: &[u8]) -> Result<BrainMessage, CodecError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Split an oversized pulse batch into multiple valid bundles.
    pub fn split_bundle(
        from_brain: BrainId,
        target_corpus: CorpusId,
        pulses: Vec<Pulse>,
        source_tick: u64,
    ) -> Vec<BrainMessage> {
        pulses
            .chunks(MAX_PULSES_PER_BUNDLE)
            .map(|chunk| BrainMessage::PulseBundle {
                from_brain,
                target_corpus,
                pulses: chunk.to_vec(),
                source_tick,
            })
            .collect()
    }
}

// ──────────────────────────────────────────────
// Peer state tracking
// ──────────────────────────────────────────────

/// What we know about a connected peer.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub brain_id: BrainId,
    pub corpora: Vec<CorpusId>,
    pub tick_rate_hz: u32,
    pub features: HashSet<String>,
    pub connected_at_tick: u64,
    pub last_seen_tick: u64,
    pub handshake_complete: bool,
}

impl PeerInfo {
    pub fn supports(&self, feature: &str) -> bool {
        self.features.contains(feature)
    }
}

// ──────────────────────────────────────────────
// BrainNetwork — the peer manager
// ──────────────────────────────────────────────

/// Manages connections, handshakes, and message queues for a brain
/// participating in a network of other brains.
#[derive(Debug)]
pub struct BrainNetwork {
    local_id: BrainId,
    local_corpora: Vec<CorpusId>,
    local_features: Vec<String>,
    tick_rate_hz: u32,
    peers: HashMap<BrainId, PeerInfo>,
    outbox: VecDeque<(Option<BrainId>, BrainMessage)>, // None = broadcast
    inbox_pulses: Vec<(CorpusId, Pulse)>, // pulses to inject into local corpora
    current_tick: u64,
}

impl BrainNetwork {
    /// Create a new network manager for this brain.
    pub fn new(local_corpora: Vec<CorpusId>, tick_rate_hz: u32) -> Self {
        Self {
            local_id: BrainId::new(),
            local_corpora,
            local_features: vec![
                "pulse-exchange".to_string(),
                "sync".to_string(),
            ],
            tick_rate_hz,
            peers: HashMap::new(),
            outbox: VecDeque::new(),
            inbox_pulses: Vec::new(),
            current_tick: 0,
        }
    }

    /// Advertise an additional capability (e.g. "vision", "swarm").
    pub fn advertise_feature(&mut self, feature: impl Into<String>) {
        self.local_features.push(feature.into());
    }

    /// Get our own brain ID.
    pub fn local_id(&self) -> BrainId {
        self.local_id
    }

    /// Update the current tick (call every engine tick).
    pub fn set_tick(&mut self, tick: u64) {
        self.current_tick = tick;
    }

    // ── Connection management ──

    /// Initiate a handshake with a new peer by queueing a Hello message.
    /// The caller then drains the outbox and sends via the transport.
    pub fn initiate_handshake(&mut self, target: BrainId) {
        self.outbox.push_back((
            Some(target),
            BrainMessage::Hello {
                protocol_version: PROTOCOL_VERSION,
                brain_id: self.local_id,
                corpora: self.local_corpora.clone(),
                tick_rate_hz: self.tick_rate_hz,
                features: self.local_features.clone(),
            },
        ));
    }

    /// Send pulses to a specific peer's corpus.
    pub fn send_pulses(
        &mut self,
        target_brain: BrainId,
        target_corpus: CorpusId,
        pulses: Vec<Pulse>,
    ) {
        let bundles = BrainCodec::split_bundle(
            self.local_id,
            target_corpus,
            pulses,
            self.current_tick,
        );
        for b in bundles {
            self.outbox.push_back((Some(target_brain), b));
        }
    }

    /// Broadcast pulses to all connected peers.
    pub fn broadcast_pulses(&mut self, target_corpus: CorpusId, pulses: Vec<Pulse>) {
        let bundles = BrainCodec::split_bundle(
            self.local_id,
            target_corpus,
            pulses,
            self.current_tick,
        );
        for b in bundles {
            self.outbox.push_back((None, b));
        }
    }

    /// Request a sync snapshot from a peer.
    pub fn request_sync(&mut self, target: BrainId) {
        self.outbox.push_back((
            Some(target),
            BrainMessage::SyncRequest {
                from_brain: self.local_id,
                at_tick: self.current_tick,
            },
        ));
    }

    /// Gracefully disconnect from a peer.
    pub fn disconnect(&mut self, target: BrainId, reason: Option<String>) {
        self.outbox.push_back((
            Some(target),
            BrainMessage::Goodbye {
                brain_id: self.local_id,
                reason,
            },
        ));
        self.peers.remove(&target);
    }

    // ── Inbound message handling ──

    /// Process raw bytes received from the transport.
    /// Returns `Some(message)` for logging / app-level handling, or
    /// `None` if the bytes couldn't be decoded.
    pub fn receive_bytes(&mut self, bytes: &[u8]) -> Option<BrainMessage> {
        let msg = BrainCodec::decode(bytes).ok()?;
        self.handle_message(msg.clone());
        Some(msg)
    }

    fn handle_message(&mut self, msg: BrainMessage) {
        match msg {
            BrainMessage::Hello {
                protocol_version,
                brain_id,
                corpora,
                tick_rate_hz,
                features,
            } => {
                if protocol_version != PROTOCOL_VERSION {
                    self.outbox.push_back((
                        Some(brain_id),
                        BrainMessage::Ack {
                            protocol_version: PROTOCOL_VERSION,
                            brain_id: self.local_id,
                            accepted: false,
                            reason: Some(format!(
                                "protocol version mismatch: want {PROTOCOL_VERSION}, got {protocol_version}"
                            )),
                        },
                    ));
                    return;
                }

                let peer = PeerInfo {
                    brain_id,
                    corpora,
                    tick_rate_hz,
                    features: features.into_iter().collect(),
                    connected_at_tick: self.current_tick,
                    last_seen_tick: self.current_tick,
                    handshake_complete: true,
                };
                self.peers.insert(brain_id, peer);

                self.outbox.push_back((
                    Some(brain_id),
                    BrainMessage::Ack {
                        protocol_version: PROTOCOL_VERSION,
                        brain_id: self.local_id,
                        accepted: true,
                        reason: None,
                    },
                ));
            }

            BrainMessage::Ack {
                brain_id, accepted, ..
            } => {
                if accepted {
                    // Mark peer as fully handshaken
                    if let Some(p) = self.peers.get_mut(&brain_id) {
                        p.handshake_complete = true;
                        p.last_seen_tick = self.current_tick;
                    } else {
                        // We initiated, now record the peer
                        self.peers.insert(
                            brain_id,
                            PeerInfo {
                                brain_id,
                                corpora: Vec::new(),
                                tick_rate_hz: 0,
                                features: HashSet::new(),
                                connected_at_tick: self.current_tick,
                                last_seen_tick: self.current_tick,
                                handshake_complete: true,
                            },
                        );
                    }
                }
            }

            BrainMessage::Ping { brain_id, tick } => {
                if let Some(p) = self.peers.get_mut(&brain_id) {
                    p.last_seen_tick = self.current_tick;
                }
                self.outbox.push_back((
                    Some(brain_id),
                    BrainMessage::Pong {
                        brain_id: self.local_id,
                        tick,
                    },
                ));
            }

            BrainMessage::Pong { brain_id, .. } => {
                if let Some(p) = self.peers.get_mut(&brain_id) {
                    p.last_seen_tick = self.current_tick;
                }
            }

            BrainMessage::PulseBundle {
                from_brain,
                target_corpus,
                pulses,
                ..
            } => {
                if let Some(p) = self.peers.get_mut(&from_brain) {
                    p.last_seen_tick = self.current_tick;
                }
                // Only accept if we own that corpus
                if self.local_corpora.contains(&target_corpus) {
                    for pulse in pulses {
                        self.inbox_pulses.push((target_corpus, pulse));
                    }
                }
            }

            BrainMessage::SyncRequest { from_brain, .. } => {
                if let Some(p) = self.peers.get_mut(&from_brain) {
                    p.last_seen_tick = self.current_tick;
                }
                // Reply with a stub snapshot. The engine will fill
                // in the real chemical state via `set_sync_response`.
                self.outbox.push_back((
                    Some(from_brain),
                    BrainMessage::SyncSnapshot {
                        from_brain: self.local_id,
                        tick: self.current_tick,
                        total_dots: 0, // engine overrides
                        active_corpora: self.local_corpora.clone(),
                        chemicals: ChemicalSnapshot::default(),
                    },
                ));
            }

            BrainMessage::SyncSnapshot { from_brain, .. } => {
                if let Some(p) = self.peers.get_mut(&from_brain) {
                    p.last_seen_tick = self.current_tick;
                }
                // App-level code reads this via `receive_bytes` return value
            }

            BrainMessage::Goodbye { brain_id, .. } => {
                self.peers.remove(&brain_id);
            }

            BrainMessage::Error { from_brain, .. } => {
                if let Some(p) = self.peers.get_mut(&from_brain) {
                    p.last_seen_tick = self.current_tick;
                }
            }
        }
    }

    // ── Outbound: drain for transport ──

    /// Drain all queued outbound messages, returning them as encoded bytes
    /// paired with their target (`None` means broadcast to all peers).
    pub fn drain_outbox(&mut self) -> Vec<(Option<BrainId>, Vec<u8>)> {
        let mut out = Vec::new();
        while let Some((target, msg)) = self.outbox.pop_front() {
            match BrainCodec::encode(&msg) {
                Ok(bytes) => out.push((target, bytes)),
                Err(e) => {
                    eprintln!("brain-protocol encode error: {e}");
                }
            }
        }
        out
    }

    /// Drain inbound pulses that should be injected into local corpora.
    pub fn drain_inbox_pulses(&mut self) -> Vec<(CorpusId, Pulse)> {
        std::mem::take(&mut self.inbox_pulses)
    }

    // ── Peer queries ──

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn active_peers(&self) -> impl Iterator<Item = &PeerInfo> {
        self.peers.values().filter(|p| p.handshake_complete)
    }

    pub fn peer(&self, id: &BrainId) -> Option<&PeerInfo> {
        self.peers.get(id)
    }

    /// Remove peers that haven't been heard from in `timeout_ticks`.
    pub fn prune_stale_peers(&mut self, timeout_ticks: u64) -> Vec<BrainId> {
        let now = self.current_tick;
        let stale: Vec<BrainId> = self
            .peers
            .iter()
            .filter(|(_, p)| now.saturating_sub(p.last_seen_tick) > timeout_ticks)
            .map(|(id, _)| *id)
            .collect();
        for id in &stale {
            self.peers.remove(id);
        }
        stale
    }
}

// ──────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bionetic_core::ids::DotId;
    use bionetic_core::rate::Rate;

    fn test_pulse(tick: u64) -> Pulse {
        Pulse::value_pulse(DotId(1), 0.5, 0.5, tick)
    }

    #[test]
    fn test_brain_id_generation() {
        let a = BrainId::new();
        let b = BrainId::new();
        assert_ne!(a, b);
        assert_eq!(a.to_hex().len(), 32);
        assert_eq!(a.short().len(), 8);
    }

    #[test]
    fn test_codec_roundtrip_hello() {
        let msg = BrainMessage::Hello {
            protocol_version: PROTOCOL_VERSION,
            brain_id: BrainId::new(),
            corpora: vec![CorpusId(0), CorpusId(1)],
            tick_rate_hz: 100,
            features: vec!["vision".into(), "swarm".into()],
        };

        let bytes = BrainCodec::encode(&msg).unwrap();
        let decoded = BrainCodec::decode(&bytes).unwrap();

        assert_eq!(decoded.type_name(), "Hello");
    }

    #[test]
    fn test_codec_roundtrip_pulse_bundle() {
        let msg = BrainMessage::PulseBundle {
            from_brain: BrainId::new(),
            target_corpus: CorpusId(3),
            pulses: vec![test_pulse(10), test_pulse(11)],
            source_tick: 10,
        };

        let bytes = BrainCodec::encode(&msg).unwrap();
        let decoded = BrainCodec::decode(&bytes).unwrap();

        if let BrainMessage::PulseBundle { pulses, target_corpus, .. } = decoded {
            assert_eq!(pulses.len(), 2);
            assert_eq!(target_corpus, CorpusId(3));
        } else {
            panic!("wrong message type");
        }
    }

    #[test]
    fn test_oversized_bundle_rejected() {
        let huge_pulses: Vec<Pulse> = (0..MAX_PULSES_PER_BUNDLE + 1)
            .map(|i| test_pulse(i as u64))
            .collect();

        let msg = BrainMessage::PulseBundle {
            from_brain: BrainId::new(),
            target_corpus: CorpusId(0),
            pulses: huge_pulses,
            source_tick: 0,
        };

        let result = BrainCodec::encode(&msg);
        assert!(matches!(result, Err(CodecError::BundleTooLarge(_))));
    }

    #[test]
    fn test_split_bundle() {
        let pulses: Vec<Pulse> = (0..(MAX_PULSES_PER_BUNDLE * 2 + 50))
            .map(|i| test_pulse(i as u64))
            .collect();

        let bundles = BrainCodec::split_bundle(
            BrainId::new(),
            CorpusId(0),
            pulses,
            0,
        );
        assert_eq!(bundles.len(), 3);
    }

    #[test]
    fn test_handshake_flow() {
        let mut a = BrainNetwork::new(vec![CorpusId(0), CorpusId(1)], 100);
        let mut b = BrainNetwork::new(vec![CorpusId(2), CorpusId(3)], 100);

        let a_id = a.local_id();
        let b_id = b.local_id();

        // A initiates
        a.initiate_handshake(b_id);
        let a_out = a.drain_outbox();
        assert_eq!(a_out.len(), 1);

        // B receives the Hello
        for (_, bytes) in &a_out {
            b.receive_bytes(bytes);
        }

        // B should now know about A and have queued an Ack
        assert_eq!(b.peer_count(), 1);
        let b_out = b.drain_outbox();
        assert_eq!(b_out.len(), 1);

        // A receives the Ack
        for (_, bytes) in &b_out {
            a.receive_bytes(bytes);
        }

        // Both sides should see each other as peers
        assert_eq!(a.peer_count(), 1);
        assert_eq!(b.peer_count(), 1);
        assert!(a.peer(&b_id).unwrap().handshake_complete);
        assert!(b.peer(&a_id).unwrap().handshake_complete);
    }

    #[test]
    fn test_pulse_exchange() {
        let mut a = BrainNetwork::new(vec![CorpusId(0)], 100);
        let mut b = BrainNetwork::new(vec![CorpusId(1)], 100);
        let b_id = b.local_id();

        // Complete handshake
        a.initiate_handshake(b_id);
        for (_, bytes) in a.drain_outbox() {
            b.receive_bytes(&bytes);
        }
        for (_, bytes) in b.drain_outbox() {
            a.receive_bytes(&bytes);
        }

        // A sends pulses to B's corpus 1
        a.send_pulses(b_id, CorpusId(1), vec![test_pulse(5), test_pulse(6)]);
        for (_, bytes) in a.drain_outbox() {
            b.receive_bytes(&bytes);
        }

        // B should have the pulses queued for injection
        let inbox = b.drain_inbox_pulses();
        assert_eq!(inbox.len(), 2);
        assert_eq!(inbox[0].0, CorpusId(1));
    }

    #[test]
    fn test_pulses_for_unowned_corpus_dropped() {
        let mut a = BrainNetwork::new(vec![CorpusId(0)], 100);
        let mut b = BrainNetwork::new(vec![CorpusId(1)], 100);
        let b_id = b.local_id();

        a.initiate_handshake(b_id);
        for (_, bytes) in a.drain_outbox() {
            b.receive_bytes(&bytes);
        }
        for (_, bytes) in b.drain_outbox() {
            a.receive_bytes(&bytes);
        }

        // A sends to CorpusId(99) which B doesn't own
        a.send_pulses(b_id, CorpusId(99), vec![test_pulse(0)]);
        for (_, bytes) in a.drain_outbox() {
            b.receive_bytes(&bytes);
        }

        assert_eq!(b.drain_inbox_pulses().len(), 0);
    }

    #[test]
    fn test_version_mismatch_rejected() {
        let mut net = BrainNetwork::new(vec![CorpusId(0)], 100);

        let bad_hello = BrainMessage::Hello {
            protocol_version: 999,
            brain_id: BrainId::new(),
            corpora: vec![],
            tick_rate_hz: 100,
            features: vec![],
        };
        let bytes = BrainCodec::encode(&bad_hello).unwrap();
        net.receive_bytes(&bytes);

        // Should have queued a rejection Ack
        let out = net.drain_outbox();
        assert_eq!(out.len(), 1);
        let msg = BrainCodec::decode(&out[0].1).unwrap();
        if let BrainMessage::Ack { accepted, .. } = msg {
            assert!(!accepted);
        } else {
            panic!("expected Ack");
        }

        // No peer should have been registered
        assert_eq!(net.peer_count(), 0);
    }

    #[test]
    fn test_stale_peer_pruning() {
        let mut net = BrainNetwork::new(vec![CorpusId(0)], 100);
        let peer_id = BrainId::new();

        // Manually inject a peer that was last seen at tick 0
        net.peers.insert(
            peer_id,
            PeerInfo {
                brain_id: peer_id,
                corpora: vec![],
                tick_rate_hz: 100,
                features: HashSet::new(),
                connected_at_tick: 0,
                last_seen_tick: 0,
                handshake_complete: true,
            },
        );

        net.set_tick(1000);
        let pruned = net.prune_stale_peers(500);

        assert_eq!(pruned.len(), 1);
        assert_eq!(pruned[0], peer_id);
        assert_eq!(net.peer_count(), 0);
    }

    #[test]
    fn test_ping_pong() {
        let mut a = BrainNetwork::new(vec![CorpusId(0)], 100);
        let a_id = a.local_id();

        let ping = BrainMessage::Ping {
            brain_id: BrainId::new(),
            tick: 42,
        };
        let bytes = BrainCodec::encode(&ping).unwrap();
        a.receive_bytes(&bytes);

        let out = a.drain_outbox();
        assert_eq!(out.len(), 1);
        let decoded = BrainCodec::decode(&out[0].1).unwrap();
        assert_eq!(decoded.type_name(), "Pong");
    }
}