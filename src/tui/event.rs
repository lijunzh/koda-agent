#![allow(dead_code)]
//! UI event types for decoupling output from stdout.
//!
//! All rendering output flows through `UiEvent` variants sent over
//! a `tokio::sync::mpsc` channel. This allows the rendering backend
//! (terminal, ratatui, or tests) to consume events independently.

use crate::providers::ToolCall;

/// Events emitted by the inference engine and tool execution layer.
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// A chunk of streaming markdown text from the LLM.
    TextDelta(String),

    /// The LLM finished streaming text. Flush any buffered output.
    TextDone,

    /// A tool is about to be called.
    ToolCall(ToolCall),

    /// Tool execution produced output.
    ToolOutput {
        tool_name: String,
        output: String,
        is_sub_agent: bool,
    },

    /// The LLM is thinking (native thinking / `<think>` blocks).
    ThinkingStart,

    /// Thinking block content.
    ThinkingDelta(String),

    /// Thinking block finished.
    ThinkingDone,

    /// The LLM response section is starting.
    ResponseStart,

    /// Spinner state change.
    SpinnerStart(String),
    SpinnerStop,

    /// Status bar update (model, context %, mode, etc.).
    StatusUpdate(StatusInfo),

    /// An informational message (not from the LLM).
    Info(String),

    /// A warning message.
    Warn(String),

    /// An error message.
    Error(String),

    /// Session footer with timing and token stats.
    Footer(FooterInfo),
}

/// Data for the persistent status bar.
#[derive(Debug, Clone, Default)]
pub struct StatusInfo {
    pub model: String,
    pub provider: String,
    pub context_percent: f64,
    pub approval_mode: String,
    pub active_tools: usize,
}

/// Data for the response footer (shown after inference completes).
#[derive(Debug, Clone)]
pub struct FooterInfo {
    pub tokens: i64,
    pub time: String,
    pub rate: f64,
    pub context: String,
    pub cache_info: Option<String>,
}

/// The sender half for UI events.
pub type UiSender = tokio::sync::mpsc::UnboundedSender<UiEvent>;

/// The receiver half for UI events.
pub type UiReceiver = tokio::sync::mpsc::UnboundedReceiver<UiEvent>;

/// Create a new UI event channel.
pub fn channel() -> (UiSender, UiReceiver) {
    tokio::sync::mpsc::unbounded_channel()
}

/// Global UI sender for use by the inference engine and display module.
/// When set, output flows through the TUI instead of stdout.
static GLOBAL_UI_TX: std::sync::OnceLock<UiSender> = std::sync::OnceLock::new();

/// Install the global UI sender (call once when TUI mode starts).
pub fn set_global_sender(tx: UiSender) {
    let _ = GLOBAL_UI_TX.set(tx);
}

/// Try to send a UI event through the global sender.
/// Returns false if no global sender is installed (classic REPL mode).
pub fn try_send(event: UiEvent) -> bool {
    if let Some(tx) = GLOBAL_UI_TX.get() {
        tx.send(event).is_ok()
    } else {
        false
    }
}

/// Check if TUI mode is active.
pub fn is_tui_mode() -> bool {
    GLOBAL_UI_TX.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_sends_and_receives() {
        let (tx, mut rx) = channel();
        tx.send(UiEvent::Info("hello".into())).unwrap();
        let event = rx.try_recv().unwrap();
        assert!(matches!(event, UiEvent::Info(msg) if msg == "hello"));
    }

    #[test]
    fn test_status_info_default() {
        let info = StatusInfo::default();
        assert!(info.model.is_empty());
        assert_eq!(info.context_percent, 0.0);
    }
}
