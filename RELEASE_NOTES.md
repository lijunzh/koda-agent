# Koda v0.1.0 🐻

**A high-performance AI coding agent built in Rust.** Single binary. Multi-provider. Zero dependencies.

## Highlights

- **19 built-in tools** — file ops, search, shell, web fetch, memory, task tracking, and more
- **6 LLM providers** — LM Studio, OpenAI, Anthropic, Gemini, Groq, Grok
- **5 embedded agents** — code reviewer, security auditor, test writer, release engineer
- **Parallel sub-agents** — run reviewer + security + testgen concurrently
- **Image analysis** — `@image.png` or drag-and-drop from file manager
- **Headless mode** — `koda -p "fix the bug"` for CI/CD and scripting
- **Auto-compact** — automatic context window management at 80% usage
- **Prompt caching** — 90% cheaper Anthropic input tokens
- **288 tests** — all passing (fmt, clippy, doc, test)

## Install

```bash
cargo install koda-agent
```

Or download the binary from the release assets below.

## Quick Start

```bash
# Interactive REPL
koda

# One-shot
koda "explain this codebase"

# Pre-release check (parallel agents)
koda -p "run reviewer, security, and testgen agents"
```

See [README.md](https://github.com/lijunzh/koda/blob/main/README.md) for full documentation.
