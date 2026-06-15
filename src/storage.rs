use crate::model::DiaryEntry;

// ---------------------------------------------------------------------------
// Pure (de)serialization — host-testable, no browser APIs.
// Gated to contexts where they are actually called (tests + wasm32 build).
// ---------------------------------------------------------------------------

#[cfg(any(test, target_arch = "wasm32"))]
pub fn serialize(entries: &[DiaryEntry]) -> String {
    serde_json::to_string(entries).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(any(test, target_arch = "wasm32"))]
pub fn deserialize(json: &str) -> Vec<DiaryEntry> {
    serde_json::from_str(json).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Thin localStorage wrappers — not unit-tested (browser only).
// The public `load` / `save` API is always present so the rest of the crate
// compiles on every target; the body diverges via cfg.
// ---------------------------------------------------------------------------

const STORAGE_KEY: &str = "diary-entries";

/// Load entries from localStorage. Returns an empty vec on the host target or
/// when the key is absent / the stored value is corrupt.
pub fn load() -> Vec<DiaryEntry> {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_storage::{LocalStorage, Storage};
        let raw: Result<String, _> = LocalStorage::get(STORAGE_KEY);
        return match raw {
            Ok(json) => deserialize(&json),
            Err(_) => vec![],
        };
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = STORAGE_KEY; // suppress unused-constant lint on host
        vec![]
    }
}

/// Persist entries to localStorage. No-op on the host target.
pub fn save(entries: &[DiaryEntry]) {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_storage::{LocalStorage, Storage};
        let json = serialize(entries);
        let _ = LocalStorage::set(STORAGE_KEY, json);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (entries, STORAGE_KEY); // suppress unused-variable lints on host
    }
}

// ---------------------------------------------------------------------------
// Tests — exercise the pure serialize/deserialize pair on the host target.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{deserialize, serialize};
    use crate::model::DiaryEntry;

    fn sample_entries() -> Vec<DiaryEntry> {
        vec![
            DiaryEntry::new("id-1", "2026-01-01T00:00:00.000Z", "Hello world"),
            DiaryEntry::new("id-2", "2026-01-02T00:00:00.000Z", "Another entry"),
        ]
    }

    #[test]
    fn round_trip_serialize_deserialize() {
        let original = sample_entries();
        let json = serialize(&original);
        let restored = deserialize(&json);
        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].id, "id-1");
        assert_eq!(restored[0].body, "Hello world");
        assert_eq!(restored[1].id, "id-2");
        assert_eq!(restored[1].body, "Another entry");
    }

    #[test]
    fn serialize_empty_vec() {
        let json = serialize(&[]);
        assert_eq!(json, "[]");
    }

    #[test]
    fn deserialize_empty_array() {
        let entries = deserialize("[]");
        assert!(entries.is_empty());
    }

    #[test]
    fn deserialize_corrupt_returns_empty() {
        let entries = deserialize("not valid json {{{{");
        assert!(entries.is_empty());
    }

    #[test]
    fn deserialize_preserves_all_fields() {
        let entries = sample_entries();
        let json = serialize(&entries);
        let restored = deserialize(&json);
        assert_eq!(restored[0].created_at, "2026-01-01T00:00:00.000Z");
        assert_eq!(restored[0].updated_at, "2026-01-01T00:00:00.000Z");
        assert_eq!(restored[1].created_at, "2026-01-02T00:00:00.000Z");
    }
}
