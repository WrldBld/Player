//! Event handlers for presentation layer
//!
//! These handlers process events from the application layer and update
//! presentation state accordingly.

pub mod session_event_handler;

pub use session_event_handler::handle_session_event;
