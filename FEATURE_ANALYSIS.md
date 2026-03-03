# Feature Analysis: What to Build vs What to Leave to MCP

*Generated 2026-03-02 — For Koda as a personal coding agent*

---

## Architecture Decision: Built-in vs MCP

### The Tradeoff

| | Built-in Tools | MCP Servers |
|---|---|---|
| **Latency** | ~0ms overhead | 5-50ms per call (JSON-RPC + subprocess IPC) |
| **Reliability** | Always works, no process management | Server can crash, timeout, fail to start |
| **Binary size** | Compiled in, grows the binary | Zero binary cost, spawned on demand |
| **UX** | Zero config needed | Requires `.mcp.json` setup |
| **Maintenance** | You maintain it | Community maintains it |
| **Flexibility** | Fixed at compile time | User adds/removes at will |
| **Context cost** | Tool defs always in prompt | Only when server is connected |

### The Rule

**Built-in if ALL of these are true:**
1. Used in >50% of coding sessions (daily driver)
2. Latency matters (called frequently in loops)
3. Small implementation (<200 lines)
4. No external dependencies (no npm, python, docker)

**MCP if ANY of these are true:**
1. Domain-specific (databases, cloud, Slack, GitHub API)
2. Requires external runtime (node, python)
3. Large implementation (>500 lines)
4. Already exists as a quality community MCP server

### Koda's Sweet Spot

Koda's strength is being a **single compiled binary with zero runtime dependencies**.
Every npm/python dependency we add erodes that value prop. The answer is clear:

> **Lean hard on built-in for core coding workflow. Lean on MCP for everything else.**

---

## Feature-by-Feature Analysis

### ✅ YES — Build as Built-in

#### 1. Todo / Task Tracker (from Goose)

**What it does:** The agent maintains a checklist of what it's working on.
Goose's implementation: the agent writes/reads a todo list that persists in the session.

**Why built-in:**
- Used constantly — the agent checks/updates the list on every turn in complex tasks
- Trivial implementation (~80 lines: read/write a string to session state)
- Zero external dependencies
- Makes the agent dramatically better at multi-step tasks
- Latency-sensitive (checked every inference loop iteration)

**How to implement in Koda:**
- New tool: `TodoUpdate` — writes the current task list (markdown checklist)
- Store in DB as a session-scoped metadata row (not a message)
- Inject into system prompt: "Your current task list: {todo}"
- ~80 lines in `src/tools/todo.rs`

**Impact: HIGH.** This is the single highest-value feature on this list.
Goose's agents are noticeably better at complex multi-file tasks because of it.

---

#### 2. Top of Mind / Context Injection (from Goose's `tom`)

**What it does:** Injects custom context into every turn via environment variables
or a file. Users put project-specific instructions that persist across sessions.

**Why built-in:**
- Koda already has this! It's called `MEMORY.md` + `/memory`
- But Goose's version is more automatic — reads from env vars and files every turn
- Could be improved: auto-inject `.koda-rules` or `KODA.md` from project root

**What to do:** Not a new feature — just improve existing memory.
Consider auto-loading a `.koda-rules` file if present (like Cursor's `.cursorrules`).
~20 lines of change to `inference::build_system_prompt()`.

**Impact: MEDIUM.** Koda already has memory. This is a UX polish.

---

### ⚠️ MAYBE — Worth Evaluating

#### 3. Code Analyzer / Tree-sitter (from Goose's `analyze`)

**What it does:** Three modes:
- Directory → structure overview (file tree with function/class counts)
- File → semantic details (functions, classes, imports, call graph counts)
- Symbol → call graph (who calls this, what does it call)

**The case FOR built-in:**
- Used frequently in coding sessions (understanding codebases)
- Provides structured data that's much better than raw `grep` output
- No external dependencies (tree-sitter has Rust bindings)
- Would make Koda significantly better at large codebase navigation

**The case AGAINST:**
- tree-sitter adds ~2-5MB to binary size (grammar files for each language)
- Goose's implementation is ~800 lines across 4 files — not trivial
- Koda's `List` + `Grep` + `Read` combo already handles most cases
- The LLM is pretty good at understanding code with just raw file contents

**Verdict:** Not now. Koda's existing tools (List, Grep, Glob, Read) cover 80% of the
use case. The remaining 20% (call graphs, symbol resolution) is nice but not essential
for a personal agent. Revisit when Koda hits a wall on large codebase tasks.

**Impact: MEDIUM.** Nice to have, but high implementation cost for marginal gain.

---

#### 4. Chat Recall (from Goose)

**What it does:** Search past conversations by keyword/date. Load session summaries
for cross-session memory.

**The case FOR built-in:**
- Koda already stores all sessions in SQLite — the data is there
- Would be ~100 lines (FTS5 search over messages table)
- Useful for "what did I do last week?" or "how did I fix that bug before?"

**The case AGAINST:**
- For a personal agent, you usually know what you did recently
- Koda's `/sessions` command already lets you resume past sessions
- Full-text search over past messages has questionable ROI

**Verdict:** Low priority. Add FTS5 index to the messages table later if needed.
The data is already there — it's a query, not an architecture change.

**Impact: LOW.** Nice for power users, not essential.

---

### ❌ NO — Leave to MCP or Skip

#### 5. Auto Visualizer (from Goose)

**What it does:** Generates HTML charts (Chart.js, D3, Mermaid, Sankey, treemaps)
from structured data. Opens in browser.

**Why NOT built-in:**
- 4.4MB of bundled JS assets (chart.min.js, d3.min.js, mermaid.min.js)
- Would double Koda's binary size
- Used rarely — most coding sessions don't need data visualization
- The LLM can already generate HTML+Chart.js via `Write` + `Bash open`
- Domain-specific: better as an MCP server or just LLM-generated code

**What to do:** Nothing. If a user wants charts, they ask Koda to write HTML.
Koda already has `Write` + `Bash` to create and open files.

**Impact: LOW.** Cool demo, bad for a lean CLI binary.

---

#### 6. Computer Controller / PDF+DOCX+XLSX (from Goose)

**What it does:** Read PDFs, Word docs, Excel files. Run automation scripts.
Take screenshots (macOS).

**Why NOT built-in:**
- PDF/DOCX/XLSX parsing requires heavy dependencies (200+ lines each in Goose)
- Used infrequently in a coding agent context
- Excellent community MCP servers exist for these
- Screenshots are macOS-only and niche

**What to do:** Document in README that users can add doc-reading MCP servers.
Example `.mcp.json` for common ones.

**Impact: LOW** for a coding agent. Higher for a general assistant.

---

#### 7. Tutorial System (from Goose)

**Why NOT:** Self-explanatory. Koda is a tool, not a tutorial platform.

---

#### 8. Apps / Sandboxed Windows (from Goose)

**Why NOT:** Koda is a CLI agent. Building HTML apps in sandboxed windows
is a desktop-app feature. Users can `Write` HTML and `Bash open` it.

---

#### 9. Improved Filesystem / Version Control / Web Browsing

**Current state:**
- **Filesystem:** `Read`, `Write`, `Edit`, `Delete`, `List`, `Glob`, `Grep` — comprehensive
- **Version Control:** `Bash` handles `git` commands. `/diff` provides review workflow
- **Web Browsing:** `WebFetch` strips HTML, returns text. Has SSRF protection.

**What competitors add:**
- Goose: PDF/DOCX/XLSX reading, web scraping with save-to-file, Puppeteer/Playwright
- Code Puppy: Puppeteer/Playwright MCP servers in catalog

**Should we improve?**
- **Filesystem:** Already strong. No gaps for a coding agent.
- **Git:** Could add a dedicated `Git` tool (status, diff, log, commit) instead of
  going through `Bash`. But `Bash` works fine — YAGNI.
- **Web:** `WebFetch` is good for docs. Full browser automation (Puppeteer)
  is better as MCP — it requires Node.js which violates the zero-dependency principle.

**Verdict:** No changes needed. The current tools cover the coding workflow well.

---

## Summary: Priority Order

| # | Feature | Type | Lines | Impact | Do It? |
|---|---|---|---|---|---|
| 1 | **Todo/Task Tracker** | Built-in tool | ~80 | HIGH | ✅ Yes, next |
| 2 | **Auto-load .koda-rules** | System prompt tweak | ~20 | MEDIUM | ✅ Yes, easy |
| 3 | **Code Analyzer** | Built-in tool | ~800 | MEDIUM | ⏳ Later |
| 4 | **Chat Recall (FTS)** | DB query | ~100 | LOW | ⏳ Later |
| 5 | Auto Visualizer | Skip | — | LOW | ❌ No |
| 6 | PDF/DOCX/XLSX | MCP | — | LOW | ❌ MCP |
| 7 | Tutorial | Skip | — | NONE | ❌ No |
| 8 | Apps | Skip | — | NONE | ❌ No |
| 9 | Better FS/Git/Web | Already good | — | LOW | ❌ No |

## Architecture Conclusion

**Koda should stay lean and built-in-first for the core coding loop.**

The built-in tools (Read, Write, Edit, Delete, List, Grep, Glob, Bash, WebFetch,
Memory, Agents) cover ~95% of personal coding agent tasks. MCP fills the remaining
5% for domain-specific needs (databases, cloud APIs, specialized tools).

The one feature that would meaningfully improve Koda's capabilities is a **todo/task
tracker** — it's the difference between an agent that loses track of multi-step tasks
and one that methodically works through them. Everything else is either already covered
or too niche to justify the binary bloat.
