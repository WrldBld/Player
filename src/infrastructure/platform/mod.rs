//! Platform-specific implementations
//!
//! This module provides platform-specific implementations of the
//! platform abstraction traits defined in application/ports/outbound/platform.rs.
//!
//! The correct platform is selected at compile time based on the target architecture.

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
mod desktop;

pub mod mock;

// Re-export the platform-specific module
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;

// Always export mock for testing
pub use mock::{
    create_mock_platform, MockDocumentProvider, MockLogProvider, MockPlatformBuilder,
    MockRandomProvider, MockStorageProvider, MockTimeProvider,
};
