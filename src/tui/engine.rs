#![allow(dead_code)]
//! TUI engine bridge: connects the inference loop to the TUI via channels.
//!
//! Runs as an async task that receives `TuiCommand`s from the renderer
//! and dispatches them through the existing inference pipeline.

use super::event::{StatusInfo, UiEvent, UiSender};
use super::renderer::TuiCommand;
use super::state::TuiState;

use crate::approval::{ApprovalMode, Settings};
use crate::config::KodaConfig;
use crate::db::Database;
use crate::inference;
use crate::input;
use crate::providers::LlmProvider;
use crate::repl::{self, ReplAction};
use crate::tools::ToolRegistry;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

/// Run the engine task that processes commands from the TUI.
#[allow(clippy::too_many_arguments)]
pub async fn run_engine(
    project_root: PathBuf,
    config: KodaConfig,
    db: Database,
    session_id: String,
    provider: Arc<RwLock<Box<dyn LlmProvider>>>,
    tools: ToolRegistry,
    system_prompt: String,
    ui_tx: UiSender,
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<TuiCommand>,
    state: Arc<Mutex<TuiState>>,
    mode: ApprovalMode,
    mut settings: Settings,
) {
    // Install the global UI sender so display.rs and inference.rs can route through TUI
    crate::tui::event::set_global_sender(ui_tx.clone());

    let tool_defs = tools.get_definitions(&config.allowed_tools);

    // Send initial status
    let _ = ui_tx.send(UiEvent::StatusUpdate(StatusInfo {
        model: config.model.clone(),
        provider: config.provider_type.to_string(),
        context_percent: 0.0,
        approval_mode: format!("{:?}", mode),
        active_tools: tool_defs.len(),
    }));

    let _ = ui_tx.send(UiEvent::Info(format!(
        "\u{1f43b} Koda v{} \u{00b7} {} \u{00b7} {}",
        env!("CARGO_PKG_VERSION"),
        config.provider_type,
        config.model
    )));
    let _ = ui_tx.send(UiEvent::Info(
        "Type your prompt and press Enter. Ctrl+C to interrupt, Ctrl+D to quit.".into(),
    ));

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            TuiCommand::Quit => break,
            TuiCommand::Interrupt => {
                crate::interrupt::clear();
                // Set again so inference loop sees it
                // We use the existing INTERRUPTED flag
                let _ = ui_tx.send(UiEvent::Warn("Interrupted".into()));
            }
            TuiCommand::UserPrompt(raw_input) => {
                let input = raw_input.trim().to_string();
                if input.is_empty() {
                    continue;
                }

                // Handle slash commands
                if input.starts_with('/') {
                    let action = repl::handle_command(&input, &config, &provider).await;
                    match action {
                        ReplAction::Quit => break,
                        ReplAction::ShowHelp => {
                            let help_lines = [
                                "",
                                "  🐻 Koda Commands",
                                "",
                                "  /agent      List available sub-agents",
                                "  /compact    Summarize conversation to reclaim context",
                                "  /cost       Show token usage for this session",
                                "  /diff       Show git diff / review / commit message",
                                "  /mcp        MCP servers: status / add / remove / restart",
                                "  /memory     View/save project & global memory",
                                "  /model      Pick a model interactively",
                                "  /provider   Switch LLM provider",
                                "  /sessions   List/resume/delete sessions",
                                "  /trust      Set approval mode (always / auto / never)",
                                "  /exit       Quit the session",
                                "",
                                "  Tips: @file to attach context · Tab to complete commands · PageUp/Down to scroll",
                                "",
                            ];
                            for line in help_lines {
                                let _ = ui_tx.send(UiEvent::Info(line.to_string()));
                            }
                        }
                        ReplAction::ShowCost => {
                            if let Ok(u) = db.session_token_usage(&session_id).await {
                                let total = u.prompt_tokens + u.completion_tokens;
                                let _ = ui_tx.send(UiEvent::Info(format!(
                                    "Session: {} prompt + {} completion = {} total tokens",
                                    u.prompt_tokens, u.completion_tokens, total
                                )));
                            }
                        }
                        ReplAction::Handled | ReplAction::NotACommand => {}
                        ReplAction::SwitchModel(model) => {
                            let _ = ui_tx.send(UiEvent::Info(format!("✓ Model set to: {model}")));
                        }
                        ReplAction::Compact => {
                            let _ = ui_tx.send(UiEvent::Info("Compacting conversation...".into()));
                        }
                        _ => {
                            let _ = ui_tx.send(UiEvent::Warn(
                                "This command requires the classic REPL. Run koda without --tui."
                                    .into(),
                            ));
                        }
                    }
                    continue;
                }

                // Process input (file attachments, etc.)
                let processed = input::process_input(&input, &project_root);

                // Store user message
                if let Err(e) = db
                    .insert_message(
                        &session_id,
                        &crate::db::Role::User,
                        Some(&processed.prompt),
                        None,
                        None,
                        None,
                    )
                    .await
                {
                    let _ = ui_tx.send(UiEvent::Error(format!("DB error: {e}")));
                    continue;
                }

                // Update status to busy
                {
                    let mut st = state.lock().unwrap();
                    st.busy = true;
                    st.spinner_msg = "\u{1f36f} Thinking...".into();
                }
                let _ = ui_tx.send(UiEvent::SpinnerStart("\u{1f36f} Thinking...".into()));

                // Run inference
                let prov = provider.read().await;
                let prov_ref: &dyn LlmProvider = &**prov;
                let result = inference::inference_loop(
                    &project_root,
                    &config,
                    &db,
                    &session_id,
                    &system_prompt,
                    prov_ref,
                    &tools,
                    &tool_defs,
                    if processed.images.is_empty() {
                        None
                    } else {
                        Some(processed.images)
                    },
                    mode,
                    &mut settings,
                )
                .await;

                // Update status to idle
                {
                    let mut st = state.lock().unwrap();
                    st.busy = false;
                    st.spinner_msg.clear();
                }
                let _ = ui_tx.send(UiEvent::SpinnerStop);

                if let Err(e) = result {
                    let _ = ui_tx.send(UiEvent::Error(format!("Inference error: {e}")));
                }
            }
        }
    }
}
