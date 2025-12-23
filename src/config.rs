//! Configuration for superego
//!
//! Reads settings from .superego/config.yaml

use std::fs;
use std::path::Path;

/// Superego configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Minutes between periodic evaluations (default: 5)
    pub eval_interval_minutes: i64,
    /// Number of recent decisions to include in carryover context (default: 2)
    pub carryover_decision_count: usize,
    /// Minutes of recent messages to include in carryover context (default: 5)
    pub carryover_window_minutes: i64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            eval_interval_minutes: 5,
            carryover_decision_count: 2,
            carryover_window_minutes: 5,
        }
    }
}

impl Config {
    /// Load config from .superego/config.yaml
    /// Falls back to defaults for missing values
    pub fn load(superego_dir: &Path) -> Self {
        let config_path = superego_dir.join("config.yaml");
        if !config_path.exists() {
            return Config::default();
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return Config::default(),
        };

        let mut config = Config::default();

        // Simple line-by-line parsing (no YAML crate dependency)
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "eval_interval_minutes" => {
                        if let Ok(v) = value.parse() {
                            config.eval_interval_minutes = v;
                        }
                    }
                    "carryover_decision_count" => {
                        if let Ok(v) = value.parse() {
                            config.carryover_decision_count = v;
                        }
                    }
                    "carryover_window_minutes" => {
                        if let Ok(v) = value.parse() {
                            config.carryover_window_minutes = v;
                        }
                    }
                    _ => {} // Ignore unknown keys
                }
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.eval_interval_minutes, 5);
        assert_eq!(config.carryover_decision_count, 2);
        assert_eq!(config.carryover_window_minutes, 5);
    }

    #[test]
    fn test_load_missing_file() {
        let dir = tempdir().unwrap();
        let config = Config::load(dir.path());
        assert_eq!(config.eval_interval_minutes, 5);
    }

    #[test]
    fn test_load_partial_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        fs::write(&config_path, "carryover_decision_count: 5\n").unwrap();

        let config = Config::load(dir.path());
        assert_eq!(config.carryover_decision_count, 5);
        assert_eq!(config.carryover_window_minutes, 5); // default
        assert_eq!(config.eval_interval_minutes, 5); // default
    }

    #[test]
    fn test_load_full_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        fs::write(
            &config_path,
            "eval_interval_minutes: 10\ncarryover_decision_count: 3\ncarryover_window_minutes: 7\n",
        )
        .unwrap();

        let config = Config::load(dir.path());
        assert_eq!(config.eval_interval_minutes, 10);
        assert_eq!(config.carryover_decision_count, 3);
        assert_eq!(config.carryover_window_minutes, 7);
    }
}
