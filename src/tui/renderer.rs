#![allow(dead_code)]
//! The ratatui-based terminal renderer.
//!
//! Drives the alternate-screen TUI with three regions:
//! - Scrollable output area
//! - Editable input line
//! - Persistent status footer

use super::event::{UiEvent, UiReceiver};
use super::state::{LineStyle, TuiState};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Commands from the TUI to the engine.
#[derive(Debug)]
pub enum TuiCommand {
    /// User submitted a prompt.
    UserPrompt(String),
    /// User requested quit.
    Quit,
    /// User pressed Ctrl+C to interrupt the current task.
    Interrupt,
}

/// Run the full TUI event loop.
///
/// This takes ownership of the terminal and runs until the user quits.
/// It communicates with the engine via:
/// - `ui_rx`: receives `UiEvent`s from the engine
/// - `cmd_tx`: sends `TuiCommand`s to the engine
pub async fn run_tui(
    state: Arc<Mutex<TuiState>>,
    mut ui_rx: UiReceiver,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<TuiCommand>,
) -> io::Result<()> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = tui_loop(&mut terminal, state, &mut ui_rx, &cmd_tx).await;

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
    terminal.show_cursor()?;

    result
}

async fn tui_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    state: Arc<Mutex<TuiState>>,
    ui_rx: &mut UiReceiver,
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<TuiCommand>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(50);

    loop {
        // Draw the UI
        {
            let st = state.lock().unwrap();
            terminal.draw(|f| draw_ui(f, &st))?;
        }

        // Poll for events with a timeout so we can also check the UI channel
        let has_crossterm_event =
            tokio::task::block_in_place(|| event::poll(tick_rate).unwrap_or(false));

        if has_crossterm_event {
            let evt = tokio::task::block_in_place(event::read)?;
            match evt {
                Event::Key(key) => {
                    if handle_key_event(key, &state, cmd_tx) {
                        return Ok(()); // Quit signal
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal will redraw on next iteration
                }
                _ => {}
            }
        }

        // Drain UI events from the engine
        while let Ok(event) = ui_rx.try_recv() {
            let mut st = state.lock().unwrap();
            apply_ui_event(&mut st, event);
        }
    }
}

/// Returns true if the app should quit.
fn handle_key_event(
    key: KeyEvent,
    state: &Arc<Mutex<TuiState>>,
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<TuiCommand>,
) -> bool {
    let mut st = state.lock().unwrap();

    match key.code {
        KeyCode::Enter => {
            if !st.input.trim().is_empty() {
                let input = st.submit_input();
                st.push_line(format!("\n> {}", input), LineStyle::Normal);
                let _ = cmd_tx.send(TuiCommand::UserPrompt(input));
            }
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if st.busy {
                let _ = cmd_tx.send(TuiCommand::Interrupt);
            } else if st.input.is_empty() {
                let _ = cmd_tx.send(TuiCommand::Quit);
                return true;
            } else {
                st.input.clear();
                st.cursor_pos = 0;
            }
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if st.input.is_empty() {
                let _ = cmd_tx.send(TuiCommand::Quit);
                return true;
            }
        }
        KeyCode::Char(c) => {
            let pos = st.cursor_pos;
            st.input.insert(pos, c);
            st.cursor_pos += c.len_utf8();

            // Auto-complete slash commands on Tab
        }
        KeyCode::Tab => {
            if st.input.starts_with('/') {
                let commands = [
                    "/help",
                    "/agent",
                    "/compact",
                    "/cost",
                    "/diff",
                    "/mcp",
                    "/memory",
                    "/model",
                    "/provider",
                    "/sessions",
                    "/trust",
                    "/exit",
                ];
                let partial = st.input.as_str();
                let matches: Vec<&&str> =
                    commands.iter().filter(|c| c.starts_with(partial)).collect();
                if matches.len() == 1 {
                    st.input = matches[0].to_string();
                    st.cursor_pos = st.input.len();
                } else if matches.len() > 1 {
                    // Show completions as info
                    let completions = matches.iter().map(|c| **c).collect::<Vec<_>>().join("  ");
                    st.push_line(format!("  {completions}"), LineStyle::Dim);
                }
            }
        }
        KeyCode::Backspace => {
            if st.cursor_pos > 0 {
                let pos = st.cursor_pos;
                let prev_len = st.input[..pos]
                    .chars()
                    .last()
                    .map(|c| c.len_utf8())
                    .unwrap_or(1);
                st.cursor_pos -= prev_len;
                let remove_pos = st.cursor_pos;
                st.input.remove(remove_pos);
            }
        }
        KeyCode::Left => {
            if st.cursor_pos > 0 {
                let pos = st.cursor_pos;
                let prev_len = st.input[..pos]
                    .chars()
                    .last()
                    .map(|c| c.len_utf8())
                    .unwrap_or(1);
                st.cursor_pos -= prev_len;
            }
        }
        KeyCode::Right => {
            let pos = st.cursor_pos;
            let len = st.input.len();
            if pos < len {
                let next_len = st.input[pos..]
                    .chars()
                    .next()
                    .map(|c| c.len_utf8())
                    .unwrap_or(1);
                st.cursor_pos += next_len;
            }
        }
        KeyCode::Up => st.history_up(),
        KeyCode::Down => st.history_down(),
        KeyCode::Home => st.cursor_pos = 0,
        KeyCode::End => st.cursor_pos = st.input.len(),
        KeyCode::PageUp => st.scroll_up(10),
        KeyCode::PageDown => st.scroll_down(10),
        KeyCode::Esc => {
            st.input.clear();
            st.cursor_pos = 0;
        }
        _ => {}
    }
    false
}

/// Apply a UI event to the TUI state.
fn apply_ui_event(state: &mut TuiState, event: UiEvent) {
    match event {
        UiEvent::TextDelta(text) => {
            // Streaming text: accumulate into the current line, break on \n
            // We apply basic markdown detection when lines are complete
            state.append_text(&text, LineStyle::Normal);
        }
        UiEvent::TextDone => {
            // Apply basic markdown styling to accumulated lines
            apply_markdown_styles(state);
            state.push_line("", LineStyle::Normal);
        }
        UiEvent::ToolCall(tc) => {
            state.push_line(
                format!("● {} — {}", tc.function_name, truncate_args(&tc.arguments)),
                LineStyle::ToolBanner,
            );
        }
        UiEvent::ToolOutput {
            tool_name, output, ..
        } => {
            for line in output.lines().take(10) {
                state.push_line(format!("│ {line}"), LineStyle::ToolOutput);
            }
            let total = output.lines().count();
            if total > 10 {
                state.push_line(format!("│ … +{} more lines", total - 10), LineStyle::Dim);
            }
            let _ = tool_name; // used for styling in future
        }
        UiEvent::ThinkingStart => {
            state.push_line("", LineStyle::Normal);
            state.push_line("🍯 Thinking ⚡", LineStyle::Thinking);
        }
        UiEvent::ThinkingDelta(text) => {
            for line in text.lines() {
                state.push_line(format!("│ {line}"), LineStyle::Thinking);
            }
        }
        UiEvent::ThinkingDone => {}
        UiEvent::ResponseStart => {
            state.push_line("", LineStyle::Normal);
            state.push_line("● Response", LineStyle::Info);
            state.push_line("", LineStyle::Normal);
        }
        UiEvent::SpinnerStart(msg) => {
            state.busy = true;
            state.spinner_msg = msg;
        }
        UiEvent::SpinnerStop => {
            state.busy = false;
            state.spinner_msg.clear();
        }
        UiEvent::StatusUpdate(info) => {
            state.status = info;
        }
        UiEvent::Info(msg) => {
            state.push_line(msg, LineStyle::Info);
        }
        UiEvent::Warn(msg) => {
            state.push_line(format!("⚠ {msg}"), LineStyle::Warning);
        }
        UiEvent::Error(msg) => {
            state.push_line(format!("✗ {msg}"), LineStyle::Error);
        }
        UiEvent::Footer(info) => {
            state.push_line(
                format!(
                    "  {} tokens · {} · {:.1} tok/s",
                    info.tokens, info.time, info.rate
                ),
                LineStyle::Dim,
            );
            if let Some(cache) = info.cache_info {
                state.push_line(format!("  {cache}"), LineStyle::Dim);
            }
        }
    }
}

/// Draw the three-region UI.
fn draw_ui(f: &mut Frame, state: &TuiState) {
    let area = f.area();

    // Layout: [output (flex)] [input (3)] [footer (1)]
    let chunks = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .split(area);

    // === Output Area ===
    let output_height = chunks[0].height as usize;
    let visible = state.visible_lines(output_height.saturating_sub(2)); // account for borders
    let output_lines: Vec<Line> = visible
        .iter()
        .map(|ol| {
            let style = match ol.style {
                LineStyle::Normal => Style::default(),
                LineStyle::Dim => Style::default().fg(Color::DarkGray),
                LineStyle::ToolBanner => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                LineStyle::ToolOutput => Style::default().fg(Color::DarkGray),
                LineStyle::Diff(super::state::DiffKind::Added) => Style::default().fg(Color::Green),
                LineStyle::Diff(super::state::DiffKind::Removed) => Style::default().fg(Color::Red),
                LineStyle::Diff(super::state::DiffKind::Hunk) => Style::default().fg(Color::Cyan),
                LineStyle::Diff(super::state::DiffKind::Context) => Style::default(),
                LineStyle::Thinking => Style::default().fg(Color::Magenta),
                LineStyle::Error => Style::default().fg(Color::Red),
                LineStyle::Warning => Style::default().fg(Color::Yellow),
                LineStyle::Info => Style::default().fg(Color::Cyan),
            };
            Line::from(Span::styled(&ol.text, style))
        })
        .collect();

    let scroll_indicator = if state.scroll_offset > 0 {
        format!(" ↑{} ", state.scroll_offset)
    } else {
        String::new()
    };

    let output_widget = Paragraph::new(output_lines)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .title(format!("Koda 🐻{scroll_indicator}")),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(output_widget, chunks[0]);

    // === Input Area ===
    let input_display = if state.busy {
        format!("  {} {}", spinner_char(), state.spinner_msg)
    } else {
        format!("  > {}", state.input)
    };
    let input_style = if state.busy {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    let input_widget = Paragraph::new(Line::from(Span::styled(&input_display, input_style)))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(input_widget, chunks[1]);

    // Place cursor
    if !state.busy {
        let cursor_x = (state.cursor_pos + 4) as u16; // "  > " prefix
        let cursor_y = chunks[1].y + 1;
        f.set_cursor_position((chunks[1].x + cursor_x.min(chunks[1].width - 1), cursor_y));
    }

    // === Footer ===
    let ctx_pct = if state.status.context_percent > 0.0 {
        format!("ctx: {:.0}%", state.status.context_percent)
    } else {
        String::new()
    };
    let footer_text = format!(
        " {} · {} · {} {}",
        state.status.model,
        state.status.approval_mode,
        ctx_pct,
        if state.status.active_tools > 0 {
            format!("⚡ {} tools", state.status.active_tools)
        } else {
            String::new()
        },
    );
    let footer = Paragraph::new(Line::from(Span::styled(
        footer_text,
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )));
    f.render_widget(footer, chunks[2]);
}

/// Simple spinner character based on elapsed time.
fn spinner_char() -> char {
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        / 100) as usize;
    ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'][idx % 10]
}

/// Apply basic markdown styling to recently accumulated lines.
/// This runs when TextDone fires, retroactively styling lines.
fn apply_markdown_styles(state: &mut TuiState) {
    for line in state.lines.iter_mut() {
        if line.style != LineStyle::Normal {
            continue;
        }
        let trimmed = line.text.trim();
        if trimmed.starts_with("# ") || trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            line.style = LineStyle::ToolBanner; // Bold cyan for headers
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            // Lists stay normal but that's fine
        } else if trimmed.starts_with("```") {
            line.style = LineStyle::Dim;
        }
    }
}

/// Truncate tool call arguments for display.
fn truncate_args(args_json: &str) -> String {
    let args: serde_json::Value = serde_json::from_str(args_json).unwrap_or_default();

    // Try to extract the most meaningful field
    if let Some(path) = args.get("file_path").and_then(|v| v.as_str()) {
        return path.to_string();
    }
    if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
        let short = if cmd.len() > 50 { &cmd[..50] } else { cmd };
        return format!("$ {short}");
    }
    if let Some(q) = args.get("search_string").and_then(|v| v.as_str()) {
        return format!("'{q}'");
    }

    // Fallback: show truncated JSON
    if args_json.len() > 60 {
        format!("{}…", &args_json[..60])
    } else {
        args_json.to_string()
    }
}
