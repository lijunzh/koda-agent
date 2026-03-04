//! KodaAgent — shared, immutable agent configuration.
//!
//! Holds everything that's constant across turns within a session:
//! config, tools, system prompt, MCP registry. Shareable via `Arc`
//! for parallel sub-agents.

use crate::config::KodaConfig;
use crate::mcp::McpRegistry;
use crate::providers::{self, LlmProvider, ToolDefinition};
use crate::tools::ToolRegistry;
use crate::{inference, memory};

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared agent configuration. Immutable after construction.
///
/// Create once, share via `Arc<KodaAgent>` across sessions and sub-agents.
pub struct KodaAgent {
    pub config: KodaConfig,
    pub project_root: PathBuf,
    pub tools: ToolRegistry,
    pub tool_defs: Vec<ToolDefinition>,
    pub system_prompt: String,
    pub mcp_registry: Arc<RwLock<McpRegistry>>,
}

impl KodaAgent {
    /// Build a new agent from config and project root.
    ///
    /// Initializes tools, MCP servers, system prompt, and tool definitions.
    pub async fn new(config: KodaConfig, project_root: PathBuf) -> Result<Self> {
        let mcp_registry = Arc::new(RwLock::new(McpRegistry::new()));
        {
            let mut mcp = mcp_registry.write().await;
            mcp.start_from_config(&project_root).await;
        }

        let tools = ToolRegistry::new(project_root.clone()).with_mcp_registry(mcp_registry.clone());
        let tool_defs = tools.get_definitions(&config.allowed_tools);

        let semantic_memory = memory::load(&project_root)?;
        let system_prompt = inference::build_system_prompt(
            &config.system_prompt,
            &semantic_memory,
            &config.agents_dir,
        );

        Ok(Self {
            config,
            project_root,
            tools,
            tool_defs,
            system_prompt,
            mcp_registry,
        })
    }

    /// Create an LLM provider from this agent's config.
    pub fn create_provider(&self) -> Box<dyn LlmProvider> {
        providers::create_provider(&self.config)
    }
}
