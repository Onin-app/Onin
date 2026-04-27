use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn new_scan_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("scan-{}", millis)
}

pub(super) fn escape_like_term(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub(super) fn fts_prefix_term(term: &str) -> String {
    format!("\"{}\"*", term.replace('"', "\"\""))
}

pub(super) fn fts_phrase_term(term: &str) -> String {
    format!("\"{}\"", term.replace('"', "\"\""))
}

pub(super) fn fts_rowid_for_path(path: &str) -> i64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in path.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    (hash & 0x7fff_ffff_ffff_ffff) as i64
}
