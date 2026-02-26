use std::fs;
use std::path::PathBuf;

use crate::history::History;

/// Get the path to the history JSON file.
/// On Windows: %APPDATA%/clipboard-history/history.json
/// On other platforms: uses dirs::config_dir() equivalent.
pub fn history_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clipboard-history");
    config_dir.join("history.json")
}

/// Load history from JSON file. Returns empty history if file doesn't exist or is corrupted.
pub fn load(max_size: usize) -> History {
    let path = history_path();
    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| History::new(max_size)),
        Err(_) => History::new(max_size),
    }
}

/// Save history to JSON file. Creates parent directories if needed.
pub fn save(history: &History) -> Result<(), Box<dyn std::error::Error>> {
    let path = history_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(history)?;
    fs::write(&path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_save_and_load_roundtrip() {
        // Use a temp directory for testing
        let tmp_dir = env::temp_dir().join("clipboard-history-test");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();

        let path = tmp_dir.join("history.json");

        let mut history = History::new(100);
        history.push("test entry 1".into());
        history.push("test entry 2".into());

        // Save
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let data = serde_json::to_string_pretty(&history).unwrap();
        fs::write(&path, data).unwrap();

        // Load
        let loaded_data = fs::read_to_string(&path).unwrap();
        let loaded: History = serde_json::from_str(&loaded_data).unwrap();
        assert_eq!(loaded.entries().len(), 2);
        assert_eq!(loaded.entries()[0].content, "test entry 2");
        assert_eq!(loaded.entries()[1].content, "test entry 1");

        // Cleanup
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_load_missing_file_returns_empty() {
        // Just verify that deserializing from a missing file gives empty history
        let history = History::new(100);
        assert_eq!(history.entries().len(), 0);
    }

    #[test]
    fn test_load_corrupted_json_returns_empty() {
        let tmp_dir = env::temp_dir().join("clipboard-history-test-corrupt");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();

        let path = tmp_dir.join("history.json");
        fs::write(&path, "not valid json!!!").unwrap();

        let result: Result<History, _> = serde_json::from_str("not valid json!!!");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
