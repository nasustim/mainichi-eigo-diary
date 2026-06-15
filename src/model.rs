use std::rc::Rc;

use yew::Reducible;

/// A single diary entry.
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub struct DiaryEntry {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub body: String,
}

impl DiaryEntry {
    /// Pure constructor — takes explicit id and timestamp so tests remain deterministic.
    pub fn new(
        id: impl Into<String>,
        timestamp: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        let ts = timestamp.into();
        Self {
            id: id.into(),
            created_at: ts.clone(),
            updated_at: ts,
            body: body.into(),
        }
    }
}

/// Reducer state holding all diary entries.
#[derive(Clone, PartialEq, Default)]
pub struct EntriesState {
    pub entries: Vec<DiaryEntry>,
}

impl EntriesState {
    pub fn new(entries: Vec<DiaryEntry>) -> Self {
        Self { entries }
    }
}

/// Actions dispatched to the reducer.
#[allow(dead_code)] // ReplaceAll is used by future import/export feature (#5)
pub enum EntriesAction {
    Add(DiaryEntry),
    /// Update the body of an existing entry. `updated_at` must be supplied by the caller
    /// (keeps the reducer pure — no JS Date calls inside).
    Update {
        id: String,
        body: String,
        updated_at: String,
    },
    Delete(String),
    ReplaceAll(Vec<DiaryEntry>),
}

impl Reducible for EntriesState {
    type Action = EntriesAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut entries = self.entries.clone();
        match action {
            EntriesAction::Add(entry) => {
                // Newest first
                entries.insert(0, entry);
            }
            EntriesAction::Update {
                id,
                body,
                updated_at,
            } => {
                for entry in entries.iter_mut() {
                    if entry.id == id {
                        entry.body = body.clone();
                        entry.updated_at = updated_at.clone();
                        break;
                    }
                }
            }
            EntriesAction::Delete(id) => {
                entries.retain(|e| e.id != id);
            }
            EntriesAction::ReplaceAll(new_entries) => {
                entries = new_entries;
            }
        }
        Rc::new(EntriesState { entries })
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use yew::Reducible;

    use super::{DiaryEntry, EntriesAction, EntriesState};

    fn entry(id: &str, ts: &str, body: &str) -> DiaryEntry {
        DiaryEntry::new(id, ts, body)
    }

    fn state(entries: Vec<DiaryEntry>) -> Rc<EntriesState> {
        Rc::new(EntriesState::new(entries))
    }

    #[test]
    fn diary_entry_new_sets_fields() {
        let e = entry("id-1", "2026-01-01T00:00:00.000Z", "hello");
        assert_eq!(e.id, "id-1");
        assert_eq!(e.created_at, "2026-01-01T00:00:00.000Z");
        assert_eq!(e.updated_at, "2026-01-01T00:00:00.000Z");
        assert_eq!(e.body, "hello");
    }

    #[test]
    fn add_inserts_newest_first() {
        let s = state(vec![entry("a", "2026-01-01", "first")]);
        let s = s.reduce(EntriesAction::Add(entry("b", "2026-01-02", "second")));
        assert_eq!(s.entries.len(), 2);
        assert_eq!(s.entries[0].id, "b");
        assert_eq!(s.entries[1].id, "a");
    }

    #[test]
    fn update_changes_body_and_updated_at() {
        let s = state(vec![entry("a", "2026-01-01", "original")]);
        let s = s.reduce(EntriesAction::Update {
            id: "a".to_string(),
            body: "edited".to_string(),
            updated_at: "2026-01-02".to_string(),
        });
        assert_eq!(s.entries[0].body, "edited");
        assert_eq!(s.entries[0].updated_at, "2026-01-02");
        assert_eq!(s.entries[0].created_at, "2026-01-01"); // unchanged
    }

    #[test]
    fn update_nonexistent_id_is_noop() {
        let s = state(vec![entry("a", "2026-01-01", "original")]);
        let s = s.reduce(EntriesAction::Update {
            id: "z".to_string(),
            body: "ghost".to_string(),
            updated_at: "2026-01-02".to_string(),
        });
        assert_eq!(s.entries.len(), 1);
        assert_eq!(s.entries[0].body, "original");
    }

    #[test]
    fn delete_removes_entry() {
        let s = state(vec![
            entry("a", "2026-01-01", "first"),
            entry("b", "2026-01-02", "second"),
        ]);
        let s = s.reduce(EntriesAction::Delete("a".to_string()));
        assert_eq!(s.entries.len(), 1);
        assert_eq!(s.entries[0].id, "b");
    }

    #[test]
    fn delete_nonexistent_id_is_noop() {
        let s = state(vec![entry("a", "2026-01-01", "first")]);
        let s = s.reduce(EntriesAction::Delete("z".to_string()));
        assert_eq!(s.entries.len(), 1);
    }

    #[test]
    fn replace_all_overwrites_entries() {
        let s = state(vec![entry("a", "2026-01-01", "old")]);
        let new = vec![
            entry("x", "2026-06-01", "new1"),
            entry("y", "2026-06-02", "new2"),
        ];
        let s = s.reduce(EntriesAction::ReplaceAll(new.clone()));
        assert_eq!(s.entries.len(), 2);
        assert_eq!(s.entries[0].id, "x");
        assert_eq!(s.entries[1].id, "y");
    }

    #[test]
    fn default_state_has_empty_entries() {
        let s = EntriesState::default();
        assert!(s.entries.is_empty());
    }
}
