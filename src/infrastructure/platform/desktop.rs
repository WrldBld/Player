//! Desktop platform implementations
//!
//! Provides platform-specific implementations for desktop using
//! standard library and native crates.

use crate::application::ports::outbound::platform::{
    DocumentProvider, LogProvider, Platform, RandomProvider, StorageProvider, TimeProvider,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Desktop time provider using std::time
#[derive(Clone, Default)]
pub struct DesktopTimeProvider;

impl TimeProvider for DesktopTimeProvider {
    fn now_unix_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn now_millis(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Desktop random provider using rand crate
#[derive(Clone, Default)]
pub struct DesktopRandomProvider;

impl RandomProvider for DesktopRandomProvider {
    fn random_f64(&self) -> f64 {
        use rand::Rng;
        rand::thread_rng().gen()
    }

    fn random_range(&self, min: i32, max: i32) -> i32 {
        use rand::Rng;
        rand::thread_rng().gen_range(min..=max)
    }
}

/// Desktop storage provider
///
/// For desktop, we use a simple in-memory implementation.
/// Could be extended to use directories crate + JSON file for persistence.
#[derive(Clone, Default)]
pub struct DesktopStorageProvider;

impl StorageProvider for DesktopStorageProvider {
    fn save(&self, _key: &str, _value: &str) {
        // Desktop: could write to ~/.config/wrldbldr/
        // For now, no-op since desktop doesn't need localStorage equivalent
        // The desktop app maintains state in memory during session
    }

    fn load(&self, _key: &str) -> Option<String> {
        // No persistence on desktop for now
        None
    }

    fn remove(&self, _key: &str) {
        // No-op
    }
}

/// Desktop log provider using tracing
#[derive(Clone, Default)]
pub struct DesktopLogProvider;

impl LogProvider for DesktopLogProvider {
    fn info(&self, msg: &str) {
        tracing::info!("{}", msg);
    }

    fn error(&self, msg: &str) {
        tracing::error!("{}", msg);
    }

    fn debug(&self, msg: &str) {
        tracing::debug!("{}", msg);
    }

    fn warn(&self, msg: &str) {
        tracing::warn!("{}", msg);
    }
}

/// Desktop document provider (no-op for page title)
#[derive(Clone, Default)]
pub struct DesktopDocumentProvider;

impl DocumentProvider for DesktopDocumentProvider {
    fn set_page_title(&self, _title: &str) {
        // No-op on desktop - window title is managed by OS/Dioxus desktop
    }
}

/// Create platform services for desktop
pub fn create_platform() -> Platform {
    Platform::new(
        DesktopTimeProvider,
        DesktopRandomProvider,
        DesktopStorageProvider,
        DesktopLogProvider,
        DesktopDocumentProvider,
    )
}
