# Koda 🐻

A high-performance AI coding agent built in Rust.

Single compiled binary. Multi-provider LLM support. Zero runtime dependencies.

## Install

```bash
# From crates.io
cargo install koda-agent

# From source
git clone https://github.com/lijunzh/koda.git
cd koda && cargo build --release
```

On first run, an onboarding wizard guides you through provider and API key setup.

## Quick Start

```bash
koda                              # Interactive REPL (auto-detects LM Studio)
koda --provider anthropic         # Use a cloud provider
koda -p "fix the bug in auth.rs"  # Headless one-shot
echo "explain this" | koda        # Piped input
```

## What's Inside

- **16 built-in tools** — file ops, search, shell, web fetch, memory, agents
- **6 LLM providers** — LM Studio, OpenAI, Anthropic, Gemini, Groq, Grok
- **5 embedded agents** — default, code reviewer, security auditor, test writer, release engineer
- **Parallel execution** — concurrent tool calls and sub-agent orchestration
- **Smart context** — auto-compact at 80%, sliding window, prompt caching (Anthropic)
- **Image analysis** — `@image.png` or drag-and-drop for multi-modal input
- **Git integration** — `/diff` review, commit message generation
- **Headless mode** — `koda -p "prompt"` with JSON output for CI/CD
- **Persistent memory** — project (`MEMORY.md`) and global (`~/.config/koda/memory.md`)

## REPL Commands

| Command | Description |
|---------|-------------|
| `/help` | Command palette (select & execute) |
| `/agent` | List available sub-agents |
| `/compact` | Summarize conversation to reclaim context |
| `/cost` | Show token usage for this session |
| `/diff` | Show/review uncommitted changes |
| `/memory` | View/save project & global memory |
| `/model` | Pick a model (↑↓ arrow keys) |
| `/provider` | Switch LLM provider |
| `/sessions` | List, resume, or delete sessions |

**Tips:** `@file` to attach context · `Ctrl+C` to interrupt · `Ctrl+D` to exit

## Documentation

- **[DESIGN.md](DESIGN.md)** — Architecture, technical stack, component breakdown
- **[FUTURE.md](FUTURE.md)** — Roadmap, competitive analysis, v0.2.0 plans
- **[CHANGELOG.md](CHANGELOG.md)** — Release history and detailed feature list

## Development

```bash
cargo test          # Run all tests
cargo clippy        # Lint
cargo run           # Run locally
```

**v0.1.x** delivers a rock-solid agent with all essential features.
**v0.2.0** is being built with Koda itself — MCP protocol, concurrent TUI, browser automation.
See [FUTURE.md](FUTURE.md) for the full roadmap.

## License

MIT
