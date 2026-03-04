//! KodaSession — per-conversation state.
//!
//! Holds mutable, per-turn state: database handle, session ID,
//! provider instance, approval settings, and cancellation token.
//! Instantiable N times for parallel sub-agents or cowork mode.

use crate::agent::KodaAgent;
use crate::approval::{ApprovalMode, Settings};
use crate::db::Database;
use crate::engine::{EngineCommand, EngineSink};
use crate::loop_guard;
use crate::providers::{ImageData, LlmProvider};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// A single conversation session with its own state.
///
/// Each session has its own provider, approval settings, and cancel token.
/// Multiple sessions can share the same `Arc<KodaAgent>`.
pub struct KodaSession {
    pub id: String,
    pub agent: Arc<KodaAgent>,
    pub db: Database,
    pub provider: Box<dyn LlmProvider>,
    pub mode: ApprovalMode,
    pub settings: Settings,
    pub cancel: CancellationToken,
}

impl KodaSession {
    /// Create a new session from an agent and database.
    pub fn new(id: String, agent: Arc<KodaAgent>, db: Database, mode: ApprovalMode) -> Self {
        let provider = agent.create_provider();
        let settings = Settings::load();
        Self {
            id,
            agent,
            db,
            provider,
            mode,
            settings,
            cancel: CancellationToken::new(),
        }
    }

    /// Run one inference turn: prompt → streaming → tool execution → response.
    ///
    /// This wraps `inference::inference_loop()` with all the session state.
    pub async fn run_turn(
        &mut self,
        pending_images: Option<Vec<ImageData>>,
        sink: &dyn EngineSink,
        cmd_rx: &mut mpsc::Receiver<EngineCommand>,
        loop_continue_prompt: &dyn Fn(u32, &[String]) -> loop_guard::LoopContinuation,
    ) -> Result<()> {
        crate::inference::inference_loop(
            &self.agent.project_root,
            &self.agent.config,
            &self.db,
            &self.id,
            &self.agent.system_prompt,
            self.provider.as_ref(),
            &self.agent.tools,
            &self.agent.tool_defs,
            pending_images,
            self.mode,
            &mut self.settings,
            sink,
            self.cancel.clone(),
            cmd_rx,
            loop_continue_prompt,
        )
        .await
    }
}
