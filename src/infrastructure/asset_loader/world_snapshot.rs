//! World snapshot loader (infrastructure adapter).
//!
//! DTO type definitions live in `crate::application::dto::world_snapshot` so this
//! module depends inward. This module is responsible only for IO + parsing.

use std::path::Path;

use anyhow::Result;

pub use crate::application::dto::world_snapshot::*;

/// Loader for world snapshots exported from the Engine.
pub struct WorldSnapshotLoader;

impl WorldSnapshotLoader {
    /// Load a world snapshot from a file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<WorldSnapshot> {
        let content = std::fs::read_to_string(path)?;
        Self::load_from_json(&content)
    }

    /// Load a world snapshot from a JSON string.
    pub fn load_from_json(json: &str) -> Result<WorldSnapshot> {
        Ok(serde_json::from_str(json)?)
    }
}
