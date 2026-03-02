# Changelog

All notable changes to Koda are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.1.0] - 2025-03-01

First public release. Single compiled Rust binary with multi-provider LLM
support, 19 built-in tools, 5 embedded agents, and zero runtime dependencies.

### Core
- 19 built-in tools: file ops (Read/Write/Edit/Delete/List), search (Grep/Glob),
  shell (Bash), web (WebFetch), memory (MemoryRead/Write), tasks (TodoRead/Write),
  agents (InvokeAgent/ListAgents/CreateAgent), tools (CreateTool/ListTools/DeleteTool)
- 6 LLM providers: LM Studio, OpenAI, Anthropic, Gemini, Groq, Grok
- Streaming responses with inline markdown rendering and syntax highlighting
- SQLite-backed durable sessions with session management
- Smart input: tab completion, ghost hints, @file context injection
- Interactive TUI menus for model/provider selection
- Tool confirmation system for destructive operations
- Safe path validation preventing directory traversal attacks
- Onboarding wizard for first-run setup

### Agents
- 5 built-in agents embedded at compile time (zero disk dependency):
  - **default** — main coding assistant
  - **reviewer** — critical code reviewer (read-only, severity-tagged)
  - **security** — security auditor (OWASP, CWE-tagged)
  - **testgen** — QA engineer (finds gaps, writes tests)
  - **releaser** — GitHub release workflow (fmt/clippy/test/doc pre-flight)
- Agent discovery merges built-in + user (~/.config/koda/agents/) + project (agents/)
- CreateAgent tool with validation (name, duplicates, prompt quality)
- `/agent` command to list available sub-agents with source tags

### Parallel Execution
- Parallel tool execution via `futures::join_all` when no confirmation needed
- Enables concurrent sub-agent invocation (reviewer + security + testgen)

### Image Analysis
- Multi-modal image support via `@image.png` references
- Drag-and-drop: auto-detects bare image paths (absolute, ~/,  ./,  quoted)
- Provider-specific formats: OpenAI image_url data URIs, Anthropic image blocks
- Supported formats: PNG, JPEG, GIF, WebP, BMP

### Context Management
- Auto-compact at 80% context usage (LLM summarizes conversation)
- Manual `/compact` command
- Sliding window safety net for token budget
- Context percentage shown in prompt at ≥75%

### Headless Mode
- `koda -p "prompt"` or `koda "prompt"` for one-shot execution
- Piped stdin auto-detection: `echo "explain" | koda`
- `--output-format json` for CI/CD integration
- Exit code 0/1 for scripting

### Git Integration
- `/diff` — show uncommitted changes summary
- `/diff review` — LLM code review of uncommitted changes
- `/diff commit` — generate conventional commit messages

### Performance
- Anthropic prompt caching: system prompt + tools cached (90% cheaper input tokens)
- Lean system prompt: 778 tokens (tool guidance lives in tool descriptions)
- Sub-agent prompts optimized: 39% smaller via same principle

### Memory & Persistence
- Project memory: MEMORY.md (also reads CLAUDE.md, AGENTS.md)
- Global memory: ~/.config/koda/memory.md
- Persistent API keys: ~/.config/koda/keys.toml
- REPL history: ~/.config/koda/history
- Clipboard integration: /copy and /paste

### Testing
- 288 tests across 6 suites
- All CI checks passing: cargo fmt, clippy -D warnings, test, doc
