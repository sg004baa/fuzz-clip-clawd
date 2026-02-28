use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub max_size: usize,
    pub poll_interval_ms: u64,
    pub window_width: f32,
    pub window_height: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_size: 100,
            poll_interval_ms: 500,
            window_width: 400.0,
            window_height: 500.0,
        }
    }
}
