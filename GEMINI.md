Got it. Here’s the rewritten **Claude Code Configuration - SPARC Development Environment** adapted for **Gemini**. I’ve preserved the same structure, rules, and emphasis, but shifted terminology and framing for Gemini’s orchestration context.

---

# Gemini Code Configuration - SPARC Development Environment

## 🚨 CRITICAL: FULL-STACK DEVELOPMENT REQUIREMENT

**⚠️ BEAR-LLM/BEAR AI SPECIAL RULE:**
**ALWAYS account for BOTH frontend AND backend in this codebase.**

When a user reports an issue or requests a feature:

1. ✅ Check **React/TypeScript frontend** (`src/components/`, `src/App.tsx`)
2. ✅ Check **Rust backend** (`src-tauri/src/`)
3. ✅ Verify **integration** between Gemini backend commands and UI
4. ✅ Test **both layers** before marking complete

**Example:** User says "download not working"

* ❌ WRONG: Only check backend `download_model` function
* ✅ CORRECT: Check backend function, frontend Download button, progress UI, and error handling

---

## 🚨 CRITICAL: CONCURRENCY & FILE MANAGEMENT

**ABSOLUTE RULES:**

1. All operations MUST run concurrently in a single execution request
2. **Never save working files or tests to root folder**
3. Always organize by correct subdirectories
4. **Use Gemini’s Task tool** for spawning agents in parallel
5. **Always validate BOTH frontend (React/TS) AND backend (Rust)**
6. Update `README.md` and related `.md` files after completion

### ⚡ GOLDEN RULE: "1 MESSAGE = ALL RELATED OPERATIONS"

**Mandatory patterns:**

* **Todo batching**: Collect all todos in one request (5–10 minimum)
* **Task tool**: Spawn ALL agents in one message with full context
* **File ops**: Batch all reads/writes/edits in one message
* **Bash ops**: Group commands into one request
* **Memory ops**: Store/retrieve context in one go

---

## 🎯 Gemini Task Tool for Agent Execution

Gemini’s **Task tool** is the primary mechanism for parallel work.

```javascript
// ✅ Correct usage: Gemini Task tool spawns concurrent agents
[Single Execution]:
  Task("Research agent", "Analyze requirements and common design patterns", "researcher")
  Task("Coder agent", "Implement features with Gemini hooks", "coder")
  Task("Tester agent", "Build full test coverage", "tester")
  Task("Reviewer agent", "Audit code for quality/security", "reviewer")
  Task("Architect agent", "Design scalable architecture", "system-architect")
```

**MCP tools are for coordination only**, such as swarm setup or orchestration.
Gemini Tasks execute the actual coding, testing, and documentation.

---

## 📁 File Organization Rules

**Never write to root. Use:**

* `/src` — Source code
* `/tests` — Test suites
* `/docs` — Documentation
* `/config` — Config files
* `/scripts` — Utilities
* `/examples` — Sample code

---

## SPARC Methodology

1. **Specification** — Analyze requirements
2. **Pseudocode** — Draft algorithms
3. **Architecture** — Define system design
4. **Refinement** — Implement with TDD
5. **Completion** — Integrate and validate

Commands follow `npx gemini-flow sparc run <mode> "<task>"`.

---

## Code Style & Best Practices

* Modular: Files under 500 lines
* Safety: Never hardcode secrets
* Test-first: Write tests before implementation
* Clean separation of concerns
* Documentation always updated

---

## 🚀 Agent Categories

* **Core**: coder, reviewer, tester, planner, researcher
* **Swarm**: coordinator, memory-manager, consensus-builder
* **Specialized**: backend-dev, mobile-dev, ml-developer, cicd-engineer
* **Testing**: tdd-swarm, production-validator
* **GitHub**: pr-manager, release-manager, code-review-swarm

---

## 🎯 Gemini vs MCP

* **Gemini Tasks**: Execution (code, file ops, testing, automation)
* **MCP Tools**: Coordination (swarm init, memory sync, orchestration)

**KEY**: MCP coordinates, Gemini Tasks deliver.

---

## Example: Full-Stack Workflow with Gemini

```javascript
// Step 1: (Optional) Use MCP for coordination
[Coordination]:
  mcp__gemini-flow__swarm_init { topology: "mesh", maxAgents: 6 }
  mcp__gemini-flow__agent_spawn { type: "researcher" }

// Step 2: Gemini Task tool executes real work
[Parallel Execution]:
  Task("Backend Dev", "Implement Rust API with commands", "backend-dev")
  Task("Frontend Dev", "Connect React UI with backend", "coder")
  Task("Database Architect", "Design schema", "code-analyzer")
  Task("Test Engineer", "Create Jest + Rust integration tests", "tester")
  Task("Reviewer", "Audit code quality and security", "reviewer")

  TodoWrite { todos: [
    {id: "1", content: "Design DB schema", priority: "high"},
    {id: "2", content: "Implement authentication", priority: "high"},
    {id: "3", content: "Write unit tests", priority: "medium"},
    {id: "4", content: "Integration tests", priority: "medium"},
    {id: "5", content: "API documentation", priority: "low"}
  ]}

  Bash "mkdir -p app/{src,tests,docs,config}"
  Write "app/src/server.rs"
  Write "app/src/App.tsx"
  Write "app/tests/server.test.rs"
  Write "app/docs/API.md"
```

---

## Performance & Features

* **Parallel execution** (2.8–4.4x faster)
* **Cross-session memory**
* **Neural training**
* **CI/CD ready**
* **Automatic test generation**

---

## Integration Tips

1. Use swarm init for large tasks
2. Scale agents as complexity grows
3. Persist context in memory
4. Monitor via hooks
5. Keep docs in sync

---

**Final Reminder: MCP coordinates strategy, Gemini Tasks execute creation.**

---

Do you want me to **convert this into an actionable Gemini workflow file** (ready to run with `npx gemini-flow`), or just keep it as a configuration guide?
