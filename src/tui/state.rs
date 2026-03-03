#![allow(dead_code)]
//! Shared TUI state for rendering.
//!
//! `TuiState` holds the scrollback buffer, input line, and status bar
//! data. It is wrapped in `Arc<Mutex<>>` and shared between the input
//! task and the rendering task.

use super::event::StatusInfo;

/// Maximum number of lines to keep in the scrollback buffer.
const MAX_SCROLLBACK: usize = 10_000;

/// A single line in the scrollback buffer with optional styling.
#[derive(Debug, Clone)]
pub struct OutputLine {
    pub text: String,
    pub style: LineStyle,
}

/// Visual style hint for an output line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineStyle {
    Normal,
    Dim,
    ToolBanner,
    ToolOutput,
    Diff(DiffKind),
    Thinking,
    Error,
    Warning,
    Info,
}

/// Diff line classification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffKind {
    Added,
    Removed,
    Hunk,
    Context,
}

/// The shared TUI state.
#[derive(Debug)]
pub struct TuiState {
    /// Scrollback buffer of rendered output lines.
    pub lines: Vec<OutputLine>,

    /// Current scroll offset from the bottom (0 = follow tail).
    pub scroll_offset: usize,

    /// Current input line text.
    pub input: String,

    /// Cursor position within the input line.
    pub cursor_pos: usize,

    /// Input history for up/down arrow navigation.
    pub history: Vec<String>,

    /// Current position in history (history.len() = "new input").
    pub history_pos: usize,

    /// Status bar data.
    pub status: StatusInfo,

    /// Whether the engine is currently busy (inference/tool execution).
    pub busy: bool,

    /// Spinner message while busy.
    pub spinner_msg: String,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            lines: Vec::with_capacity(1024),
            scroll_offset: 0,
            input: String::new(),
            cursor_pos: 0,
            history: Vec::new(),
            history_pos: 0,
            status: StatusInfo::default(),
            busy: false,
            spinner_msg: String::new(),
        }
    }

    /// Append a line to the scrollback buffer.
    pub fn push_line(&mut self, text: impl Into<String>, style: LineStyle) {
        self.lines.push(OutputLine {
            text: text.into(),
            style,
        });

        // Trim old lines if buffer is too large
        if self.lines.len() > MAX_SCROLLBACK {
            let drain = self.lines.len() - MAX_SCROLLBACK;
            self.lines.drain(..drain);
            self.scroll_offset = self.scroll_offset.saturating_sub(drain);
        }

        // Auto-scroll to bottom when new content arrives (if user hasn't scrolled up)
        if self.scroll_offset == 0 {
            // Already at bottom, stay there
        }
    }

    /// Append multiple lines from a multiline string.
    pub fn push_text(&mut self, text: &str, style: LineStyle) {
        for line in text.lines() {
            self.push_line(line, style);
        }
    }

    /// Get visible lines for a given viewport height.
    pub fn visible_lines(&self, height: usize) -> &[OutputLine] {
        let total = self.lines.len();
        if total == 0 || height == 0 {
            return &[];
        }
        let end = total.saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(height);
        &self.lines[start..end]
    }

    /// Scroll up by n lines.
    pub fn scroll_up(&mut self, n: usize) {
        let max_offset = self.lines.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + n).min(max_offset);
    }

    /// Scroll down by n lines (toward the tail).
    pub fn scroll_down(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    /// Submit the current input, returning it and clearing the line.
    pub fn submit_input(&mut self) -> String {
        let input = self.input.clone();
        if !input.trim().is_empty() {
            self.history.push(input.clone());
        }
        self.input.clear();
        self.cursor_pos = 0;
        self.history_pos = self.history.len();
        input
    }

    /// Navigate history up.
    pub fn history_up(&mut self) {
        if self.history_pos > 0 {
            self.history_pos -= 1;
            self.input = self.history[self.history_pos].clone();
            self.cursor_pos = self.input.len();
        }
    }

    /// Navigate history down.
    pub fn history_down(&mut self) {
        if self.history_pos < self.history.len() {
            self.history_pos += 1;
            if self.history_pos < self.history.len() {
                self.input = self.history[self.history_pos].clone();
            } else {
                self.input.clear();
            }
            self.cursor_pos = self.input.len();
        }
    }
}

impl Default for TuiState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_visible() {
        let mut state = TuiState::new();
        for i in 0..20 {
            state.push_line(format!("line {i}"), LineStyle::Normal);
        }
        let visible = state.visible_lines(5);
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[4].text, "line 19");
    }

    #[test]
    fn test_scroll_up_down() {
        let mut state = TuiState::new();
        for i in 0..20 {
            state.push_line(format!("line {i}"), LineStyle::Normal);
        }
        state.scroll_up(5);
        assert_eq!(state.scroll_offset, 5);
        let visible = state.visible_lines(5);
        assert_eq!(visible[4].text, "line 14");
        state.scroll_down(3);
        assert_eq!(state.scroll_offset, 2);
    }

    #[test]
    fn test_submit_input() {
        let mut state = TuiState::new();
        state.input = "hello world".to_string();
        state.cursor_pos = 11;
        let submitted = state.submit_input();
        assert_eq!(submitted, "hello world");
        assert!(state.input.is_empty());
        assert_eq!(state.history.len(), 1);
    }

    #[test]
    fn test_history_navigation() {
        let mut state = TuiState::new();
        state.input = "first".to_string();
        state.submit_input();
        state.input = "second".to_string();
        state.submit_input();

        state.history_up();
        assert_eq!(state.input, "second");
        state.history_up();
        assert_eq!(state.input, "first");
        state.history_down();
        assert_eq!(state.input, "second");
        state.history_down();
        assert!(state.input.is_empty());
    }

    #[test]
    fn test_max_scrollback_trims() {
        let mut state = TuiState::new();
        for i in 0..10_050 {
            state.push_line(format!("line {i}"), LineStyle::Normal);
        }
        assert!(state.lines.len() <= 10_000);
    }
}
