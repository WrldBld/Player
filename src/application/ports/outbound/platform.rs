//! Platform abstraction ports for cross-platform compatibility
//!
//! These traits abstract platform-specific operations so that:
//! 1. Application/presentation code remains platform-agnostic
//! 2. Platform-specific code is isolated in infrastructure
//! 3. Code becomes easily testable with mock implementations

use std::{future::Future, pin::Pin};

/// Time operations abstraction
pub trait TimeProvider: Clone + 'static {
    /// Get current time as Unix timestamp in seconds
    fn now_unix_secs(&self) -> u64;

    /// Get current time in milliseconds since epoch
    fn now_millis(&self) -> u64;
}

/// Async sleep abstraction
///
/// Used to avoid `#[cfg]` branches in UI code (e.g. typewriter effect).
pub trait SleepProvider: Clone + 'static {
    fn sleep_ms(&self, ms: u64) -> Pin<Box<dyn Future<Output = ()> + 'static>>;
}

/// Random number generation abstraction
pub trait RandomProvider: Clone + 'static {
    /// Generate random f64 in range [0.0, 1.0)
    fn random_f64(&self) -> f64;

    /// Generate random i32 in range [min, max] (inclusive)
    fn random_range(&self, min: i32, max: i32) -> i32;
}

/// Persistent storage abstraction (localStorage/file-based)
pub trait StorageProvider: Clone + 'static {
    /// Save a string value with the given key
    fn save(&self, key: &str, value: &str);

    /// Load a string value by key, returns None if not found
    fn load(&self, key: &str) -> Option<String>;

    /// Remove a value by key
    fn remove(&self, key: &str);
}

/// Logging abstraction
pub trait LogProvider: Clone + 'static {
    fn info(&self, msg: &str);
    fn error(&self, msg: &str);
    fn debug(&self, msg: &str);
    fn warn(&self, msg: &str);
}

/// Browser document operations (page title, etc.)
pub trait DocumentProvider: Clone + 'static {
    /// Set the browser page title (no-op on desktop)
    fn set_page_title(&self, title: &str);
}

/// Unified platform services container
///
/// Provides all platform abstractions through a single injectable type.
/// Use via Dioxus context: `use_context::<Platform>()`
#[derive(Clone)]
pub struct Platform {
    time: std::sync::Arc<dyn TimeProviderDyn>,
    sleep: std::sync::Arc<dyn SleepProviderDyn>,
    random: std::sync::Arc<dyn RandomProviderDyn>,
    storage: std::sync::Arc<dyn StorageProviderDyn>,
    log: std::sync::Arc<dyn LogProviderDyn>,
    document: std::sync::Arc<dyn DocumentProviderDyn>,
}

// Dynamic trait versions for Arc storage (need Send + Sync for Dioxus context)
trait TimeProviderDyn: Send + Sync {
    fn now_unix_secs(&self) -> u64;
    fn now_millis(&self) -> u64;
}

trait SleepProviderDyn: Send + Sync {
    fn sleep_ms(&self, ms: u64) -> Pin<Box<dyn Future<Output = ()> + 'static>>;
}

trait RandomProviderDyn: Send + Sync {
    fn random_f64(&self) -> f64;
    fn random_range(&self, min: i32, max: i32) -> i32;
}

trait StorageProviderDyn: Send + Sync {
    fn save(&self, key: &str, value: &str);
    fn load(&self, key: &str) -> Option<String>;
    fn remove(&self, key: &str);
}

trait LogProviderDyn: Send + Sync {
    fn info(&self, msg: &str);
    fn error(&self, msg: &str);
    fn debug(&self, msg: &str);
    fn warn(&self, msg: &str);
}

trait DocumentProviderDyn: Send + Sync {
    fn set_page_title(&self, title: &str);
}

// Blanket implementations
impl<T: TimeProvider + Send + Sync> TimeProviderDyn for T {
    fn now_unix_secs(&self) -> u64 {
        TimeProvider::now_unix_secs(self)
    }
    fn now_millis(&self) -> u64 {
        TimeProvider::now_millis(self)
    }
}

impl<T: SleepProvider + Send + Sync> SleepProviderDyn for T {
    fn sleep_ms(&self, ms: u64) -> Pin<Box<dyn Future<Output = ()> + 'static>> {
        SleepProvider::sleep_ms(self, ms)
    }
}

impl<T: RandomProvider + Send + Sync> RandomProviderDyn for T {
    fn random_f64(&self) -> f64 {
        RandomProvider::random_f64(self)
    }
    fn random_range(&self, min: i32, max: i32) -> i32 {
        RandomProvider::random_range(self, min, max)
    }
}

impl<T: StorageProvider + Send + Sync> StorageProviderDyn for T {
    fn save(&self, key: &str, value: &str) {
        StorageProvider::save(self, key, value)
    }
    fn load(&self, key: &str) -> Option<String> {
        StorageProvider::load(self, key)
    }
    fn remove(&self, key: &str) {
        StorageProvider::remove(self, key)
    }
}

impl<T: LogProvider + Send + Sync> LogProviderDyn for T {
    fn info(&self, msg: &str) {
        LogProvider::info(self, msg)
    }
    fn error(&self, msg: &str) {
        LogProvider::error(self, msg)
    }
    fn debug(&self, msg: &str) {
        LogProvider::debug(self, msg)
    }
    fn warn(&self, msg: &str) {
        LogProvider::warn(self, msg)
    }
}

impl<T: DocumentProvider + Send + Sync> DocumentProviderDyn for T {
    fn set_page_title(&self, title: &str) {
        DocumentProvider::set_page_title(self, title)
    }
}

impl Platform {
    /// Create a new Platform with the given providers
    pub fn new<Tm, Sl, R, S, L, D>(
        time: Tm,
        sleep: Sl,
        random: R,
        storage: S,
        log: L,
        document: D,
    ) -> Self
    where
        Tm: TimeProvider + Send + Sync,
        Sl: SleepProvider + Send + Sync,
        R: RandomProvider + Send + Sync,
        S: StorageProvider + Send + Sync,
        L: LogProvider + Send + Sync,
        D: DocumentProvider + Send + Sync,
    {
        Self {
            time: std::sync::Arc::new(time),
            sleep: std::sync::Arc::new(sleep),
            random: std::sync::Arc::new(random),
            storage: std::sync::Arc::new(storage),
            log: std::sync::Arc::new(log),
            document: std::sync::Arc::new(document),
        }
    }

    /// Get current time as Unix timestamp in seconds
    pub fn now_unix_secs(&self) -> u64 {
        self.time.now_unix_secs()
    }

    /// Get current time in milliseconds since epoch
    pub fn now_millis(&self) -> u64 {
        self.time.now_millis()
    }

    /// Sleep for the given number of milliseconds.
    pub fn sleep_ms(&self, ms: u64) -> Pin<Box<dyn Future<Output = ()> + 'static>> {
        self.sleep.sleep_ms(ms)
    }

    /// Generate random f64 in range [0.0, 1.0)
    pub fn random_f64(&self) -> f64 {
        self.random.random_f64()
    }

    /// Generate random i32 in range [min, max] (inclusive)
    pub fn random_range(&self, min: i32, max: i32) -> i32 {
        self.random.random_range(min, max)
    }

    /// Save a string value with the given key
    pub fn storage_save(&self, key: &str, value: &str) {
        self.storage.save(key, value)
    }

    /// Load a string value by key, returns None if not found
    pub fn storage_load(&self, key: &str) -> Option<String> {
        self.storage.load(key)
    }

    /// Remove a value by key
    pub fn storage_remove(&self, key: &str) {
        self.storage.remove(key)
    }

    /// Log an info message
    pub fn log_info(&self, msg: &str) {
        self.log.info(msg)
    }

    /// Log an error message
    pub fn log_error(&self, msg: &str) {
        self.log.error(msg)
    }

    /// Log a debug message
    pub fn log_debug(&self, msg: &str) {
        self.log.debug(msg)
    }

    /// Log a warning message
    pub fn log_warn(&self, msg: &str) {
        self.log.warn(msg)
    }

    /// Set the browser page title (no-op on desktop)
    pub fn set_page_title(&self, title: &str) {
        self.document.set_page_title(title)
    }
}

/// Storage key constants
pub mod storage_keys {
    pub const SERVER_URL: &str = "wrldbldr_server_url";
    pub const ROLE: &str = "wrldbldr_role";
    pub const LAST_WORLD: &str = "wrldbldr_last_world";
}
