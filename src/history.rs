use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: u64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    entries: Vec<ClipboardEntry>,
    max_size: usize,
    #[serde(default)]
    next_id: u64,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
            next_id: 1,
        }
    }

    /// Add content to history.
    /// - If same as the most recent entry, skip.
    /// - If duplicate exists in history, move it to the front and update timestamp.
    /// - If over max_size, remove the oldest entry.
    pub fn push(&mut self, content: String) -> bool {
        // Skip if same as most recent
        if let Some(latest) = self.entries.first() {
            if latest.content == content {
                return false;
            }
        }

        // Check for duplicate in history
        if let Some(pos) = self.entries.iter().position(|e| e.content == content) {
            // Move existing entry to front with updated timestamp
            let mut entry = self.entries.remove(pos);
            entry.created_at = Utc::now();
            self.entries.insert(0, entry);
            return true;
        }

        // New entry
        let entry = ClipboardEntry {
            id: self.next_id,
            content,
            created_at: Utc::now(),
        };
        self.next_id += 1;
        self.entries.insert(0, entry);

        // Trim if over max size
        if self.entries.len() > self.max_size {
            self.entries.truncate(self.max_size);
        }

        true
    }

    pub fn entries(&self) -> &[ClipboardEntry] {
        &self.entries
    }

    #[allow(dead_code)]
    pub fn get_by_id(&self, id: u64) -> Option<&ClipboardEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_new_entry() {
        let mut history = History::new(100);
        assert!(history.push("hello".into()));
        assert_eq!(history.entries().len(), 1);
        assert_eq!(history.entries()[0].content, "hello");
    }

    #[test]
    fn test_skip_duplicate_of_most_recent() {
        let mut history = History::new(100);
        history.push("hello".into());
        assert!(!history.push("hello".into()));
        assert_eq!(history.entries().len(), 1);
    }

    #[test]
    fn test_move_past_duplicate_to_front() {
        let mut history = History::new(100);
        history.push("first".into());
        history.push("second".into());
        history.push("third".into());

        // Push "first" again — should move to front
        assert!(history.push("first".into()));
        assert_eq!(history.entries().len(), 3);
        assert_eq!(history.entries()[0].content, "first");
        assert_eq!(history.entries()[1].content, "third");
        assert_eq!(history.entries()[2].content, "second");
    }

    #[test]
    fn test_max_size_enforced() {
        let mut history = History::new(3);
        history.push("a".into());
        history.push("b".into());
        history.push("c".into());
        history.push("d".into());

        assert_eq!(history.entries().len(), 3);
        // Most recent first
        assert_eq!(history.entries()[0].content, "d");
        assert_eq!(history.entries()[1].content, "c");
        assert_eq!(history.entries()[2].content, "b");
    }

    #[test]
    fn test_get_by_id() {
        let mut history = History::new(100);
        history.push("hello".into());
        let id = history.entries()[0].id;
        assert!(history.get_by_id(id).is_some());
        assert!(history.get_by_id(9999).is_none());
    }

    #[test]
    fn test_id_increments() {
        let mut history = History::new(100);
        history.push("a".into());
        history.push("b".into());
        // IDs should be unique and incrementing
        let ids: Vec<u64> = history.entries().iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
    }
}
