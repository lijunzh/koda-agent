//! Terminal UI module.
//!
//! Provides the event bus, shared state, and ratatui-based rendering.
//! The legacy `select()` widget is re-exported for backward compatibility.

pub mod engine;
pub mod event;
pub mod renderer;
pub mod select;
pub mod state;

// Re-export legacy select widget so existing code (`tui::select`, `tui::SelectOption`) still works.
pub use select::{SelectOption, select};
