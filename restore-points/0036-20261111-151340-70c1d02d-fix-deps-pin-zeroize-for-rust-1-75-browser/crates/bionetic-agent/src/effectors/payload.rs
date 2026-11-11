//! Common payload helpers. Payloads are fixed motor slots, not generated language.

pub fn slot_index(payload: &[u8], len: usize) -> Option<usize> {
    if len == 0 { return None; }
    Some(*payload.first()? as usize % len)
}

pub fn as_text(payload: &[u8]) -> Option<String> {
    let end = payload.iter().rposition(|byte| *byte != 0).map(|index| index + 1).unwrap_or(0);
    if end == 0 { return None; }
    std::str::from_utf8(&payload[..end]).ok().map(str::to_owned)
}

pub fn fingerprint(payload: &[u8]) -> u32 {
    let mut hash = 0x811c_9dc5u32;
    for byte in payload {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}
