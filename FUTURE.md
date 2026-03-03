# Koda — Design Philosophy & Competitive Positioning

**Koda's core strength:** Single compiled Rust binary, multi-provider LLM
support, zero runtime dependencies. No other agent matches that deployment story.

**Feature requests and roadmap items are tracked as GitHub issues.**
See the [issue tracker](https://github.com/lijunzh/koda/issues) for the full backlog.

---

## Design Principles

1. **Zero runtime dependencies** — single compiled binary, ships everywhere.
2. **Built-in first for core coding loop** — Read/Write/Edit/Bash/Grep cover ~95%
   of daily coding tasks. MCP fills the rest for domain-specific needs.
3. **MCP for everything else** — databases, cloud APIs, browser automation, docs.
   Don't bloat the binary with features that belong in MCP servers.
4. **Stay lean** — YAGNI. Goose's MCP code is 2200 lines; Koda's is ~400.
   Complexity is a cost. Every line must earn its place.

### Built-in vs MCP Decision Rule

**Built-in if ALL of these are true:**
- Used in >50% of coding sessions
- Latency-sensitive (called frequently in loops)
- Small implementation (<200 lines)
- Zero external dependencies

**MCP if ANY of these are true:**
- Domain-specific (databases, cloud, Slack, GitHub API)
- Requires external runtime (node, python, docker)
- Large implementation (>500 lines)
- Already exists as a quality community MCP server

---

## Security Audit — 2025-04-01

**RUSTSEC-2023-0071 (Medium 5.9):** `rsa` v0.9.10 via `sqlx → sqlx-mysql`.
Marvin Attack timing sidechannel. No fixed upstream yet. Koda uses SQLite
primarily — low risk. Monitor for upstream fix.

**RUSTSEC-2025-0141:** `bincode` unmaintained (via `syntect`). No CVE. Stable.

**RUSTSEC-2025-0119:** `number_prefix` unmaintained (via `indicatif`). No CVE. Stable.

---

## Competitive Reference

### Core Agent Capabilities

| Capability | Koda | Goose | Claude Code | Code Puppy |
|------------|:----:|:-----:|:-----------:|:----------:|
| Core file/shell/search | ✅ | ✅ | ✅ | ✅ |
| Multi-provider LLM | ✅ (8+) | ✅ (25+) | ❌ (1) | ✅ (65+) |
| Streaming + markdown | ✅ | ✅ | ✅ | ✅ |
| Sub-agent delegation | ✅ | ✅ | ✅ | ✅ |
| Zero-dependency binary | ✅ | ❌ | ❌ | ❌ |
| Session management | ✅ | ✅ | ✅ | ✅ |
| Auto-memory | ✅ | ✅ | ✅ | ❌ |
| Context compression | ✅ | ✅ | ✅ | ❌ |
| Headless/CI mode | ✅ | ✅ | ✅ | ✅ |
| Prompt caching | ✅ | ✅ | ✅ | ✅ |
| MCP protocol | ✅ | ✅ | ✅ | ✅ |
| Agent teams (parallel) | ✅ Phase 1 | ✅ | ✅ | ✅ |
| Extended thinking | ✅ | ✅ | ✅ | ✅ |

### Gaps (tracked as issues)

| Capability | Gap | Issue |
|---|---|---|
| Per-tool permissions | Missing | #15 |
| Type-ahead / non-blocking input | Missing | #16 |
| Full TUI | Missing | #17 |
| Lead-worker multi-model | Missing | #20 |
| Plugin/hook system | Missing | #19 |
| Skills/recipe system | Missing | #26 |
| IDE integration | Missing | #21 |
| More providers (Ollama, etc.) | Partial | #13 |