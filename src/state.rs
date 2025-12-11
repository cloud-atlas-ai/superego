/// State management for superego
///
/// Maintains current phase and pending override in .superego/state.json

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::decision::Phase;

/// Pending override - allows a single blocked action to proceed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOverride {
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

/// Current superego state (cached between user messages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub phase: Phase,
    pub since: DateTime<Utc>,
    pub approved_scope: Option<String>,
    pub last_evaluated: Option<DateTime<Utc>>,
    pub pending_override: Option<PendingOverride>,
    pub disabled: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            phase: Phase::Exploring,
            since: Utc::now(),
            approved_scope: None,
            last_evaluated: None,
            pending_override: None,
            disabled: false,
        }
    }
}

impl State {
    /// Create a new state with the given phase
    pub fn with_phase(phase: Phase) -> Self {
        State {
            phase,
            since: Utc::now(),
            ..Default::default()
        }
    }

    /// Check if writes are allowed (phase is READY or override pending)
    pub fn allows_write(&self) -> bool {
        if self.disabled {
            return true;
        }
        self.phase == Phase::Ready || self.pending_override.is_some()
    }

    /// Consume the pending override (call after allowing a blocked action)
    pub fn consume_override(&mut self) {
        self.pending_override = None;
    }

    /// Set a pending override
    pub fn set_override(&mut self, reason: String) {
        self.pending_override = Some(PendingOverride {
            reason,
            timestamp: Utc::now(),
        });
    }

    /// Update to a new phase
    pub fn transition_to(&mut self, phase: Phase, scope: Option<String>) {
        self.phase = phase;
        self.since = Utc::now();
        self.approved_scope = scope;
        self.last_evaluated = Some(Utc::now());
    }
}

/// Error type for state operations
#[derive(Debug)]
pub enum StateError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::IoError(e) => write!(f, "IO error: {}", e),
            StateError::JsonError(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for StateError {}

impl From<std::io::Error> for StateError {
    fn from(e: std::io::Error) -> Self {
        StateError::IoError(e)
    }
}

impl From<serde_json::Error> for StateError {
    fn from(e: serde_json::Error) -> Self {
        StateError::JsonError(e)
    }
}

/// State manager - reads and writes .superego/state.json
pub struct StateManager {
    state_path: PathBuf,
}

impl StateManager {
    /// Create a new state manager for the given .superego directory
    pub fn new(superego_dir: &Path) -> Self {
        StateManager {
            state_path: superego_dir.join("state.json"),
        }
    }

    /// Load state from disk (returns default if file doesn't exist)
    pub fn load(&self) -> Result<State, StateError> {
        if !self.state_path.exists() {
            return Ok(State::default());
        }

        let file = File::open(&self.state_path)?;
        let reader = BufReader::new(file);
        let state = serde_json::from_reader(reader)?;
        Ok(state)
    }

    /// Save state to disk
    pub fn save(&self, state: &State) -> Result<(), StateError> {
        // Ensure parent directory exists
        if let Some(parent) = self.state_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(&self.state_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, state)?;
        Ok(())
    }

    /// Load, modify, and save state atomically
    pub fn update<F>(&self, f: F) -> Result<State, StateError>
    where
        F: FnOnce(&mut State),
    {
        let mut state = self.load()?;
        f(&mut state);
        self.save(&state)?;
        Ok(state)
    }

    /// Check if state file exists
    pub fn exists(&self) -> bool {
        self.state_path.exists()
    }

    /// Delete state file (for reset)
    pub fn clear(&self) -> Result<(), StateError> {
        if self.state_path.exists() {
            fs::remove_file(&self.state_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_state() {
        let state = State::default();
        assert_eq!(state.phase, Phase::Exploring);
        assert!(!state.allows_write());
        assert!(!state.disabled);
    }

    #[test]
    fn test_ready_allows_write() {
        let state = State::with_phase(Phase::Ready);
        assert!(state.allows_write());
    }

    #[test]
    fn test_override_allows_write() {
        let mut state = State::with_phase(Phase::Discussing);
        assert!(!state.allows_write());

        state.set_override("user approved".to_string());
        assert!(state.allows_write());

        state.consume_override();
        assert!(!state.allows_write());
    }

    #[test]
    fn test_disabled_allows_write() {
        let mut state = State::with_phase(Phase::Exploring);
        state.disabled = true;
        assert!(state.allows_write());
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let manager = StateManager::new(dir.path());

        let mut state = State::with_phase(Phase::Ready);
        state.approved_scope = Some("implement auth".to_string());

        manager.save(&state).unwrap();

        let loaded = manager.load().unwrap();
        assert_eq!(loaded.phase, Phase::Ready);
        assert_eq!(loaded.approved_scope, Some("implement auth".to_string()));
    }

    #[test]
    fn test_load_missing_returns_default() {
        let dir = tempdir().unwrap();
        let manager = StateManager::new(dir.path());

        let state = manager.load().unwrap();
        assert_eq!(state.phase, Phase::Exploring);
    }

    #[test]
    fn test_update() {
        let dir = tempdir().unwrap();
        let manager = StateManager::new(dir.path());

        manager.update(|s| {
            s.transition_to(Phase::Ready, Some("build feature".to_string()));
        }).unwrap();

        let loaded = manager.load().unwrap();
        assert_eq!(loaded.phase, Phase::Ready);
        assert_eq!(loaded.approved_scope, Some("build feature".to_string()));
    }
}
