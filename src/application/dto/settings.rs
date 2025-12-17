//! Settings DTOs - Application settings types
//!
//! This module contains DTOs for Engine application settings.
//! These types mirror the Engine's AppSettings structure.

use serde::{Deserialize, Serialize};

/// Application settings from the Engine
///
/// These settings control various aspects of the Engine's behavior,
/// including conversation limits, circuit breaker configuration,
/// animation timing, and validation constraints.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// Maximum number of conversation turns before automatic summarization
    pub max_conversation_turns: usize,

    /// Number of consecutive failures before circuit breaker opens
    pub circuit_breaker_failure_threshold: u32,

    /// Duration (in seconds) that circuit breaker remains open
    pub circuit_breaker_open_duration_secs: u64,

    /// TTL (in seconds) for health check cache entries
    pub health_check_cache_ttl_secs: u64,

    /// Maximum allowed length for name fields
    pub max_name_length: usize,

    /// Maximum allowed length for description fields
    pub max_description_length: usize,

    /// Delay (in milliseconds) between sentences in typewriter effect
    pub typewriter_sentence_delay_ms: u64,

    /// Delay (in milliseconds) for pause punctuation in typewriter effect
    pub typewriter_pause_delay_ms: u64,

    /// Delay (in milliseconds) between characters in typewriter effect
    pub typewriter_char_delay_ms: u64,

    /// Default maximum value for character stats
    pub default_max_stat_value: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_conversation_turns: 50,
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_open_duration_secs: 60,
            health_check_cache_ttl_secs: 300,
            max_name_length: 100,
            max_description_length: 2000,
            typewriter_sentence_delay_ms: 300,
            typewriter_pause_delay_ms: 150,
            typewriter_char_delay_ms: 30,
            default_max_stat_value: 100,
        }
    }
}
