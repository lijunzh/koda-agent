**Architectural Update: The "SQLite Compromise" (Graph over Relational)**

After reviewing architectural options, we are un-deferring this issue and moving forward with a pragmatic approach! Instead of migrating to a heavy Graph Database (which would break our zero-dependency single binary rule and add massive complexity), we will implement Graph capabilities *inside* our existing architecture.

We are breaking this issue down into two phases:

### Phase 1: On-the-Fly In-Memory Graph
We will introduce `tree-sitter` (for parsing) and `petgraph` (for graph traversal) to build ephemeral AST graphs in memory *on-demand*. 
- When Koda needs a call graph or symbol analysis, it parses the file(s) into memory, runs the traversal, feeds the markdown result to the LLM, and throws the graph away.
- **Why?** Blazingly fast to implement, zero database migrations required, and completely solves the LLM's blind spots for 90% of codebases.

### Phase 2: SQLite Graph Cache (For Huge Monorepos)
Once Phase 1 is stable, if we notice performance issues on massive repositories, we will add a `nodes` and `edges` table to our existing `koda.db`. 
- SQLite will act as a persistent graph cache.
- We parse with `tree-sitter`, dump relations (e.g., `CALLS`, `IMPLEMENTS`) into SQLite, and query them relationally.

Creating sub-issues for Phase 1 and Phase 2 now!