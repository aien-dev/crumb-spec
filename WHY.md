# Why Crumbs: The Case for Filesystem-Native Agent Grounding

## 1. The Core Limitation of Prompt Stuffing

When an AI coding assistant starts a session, modern systems attempt to solve codebase orientation through two common strategies:

1. **Massive File-Tree Ingestion**: The agent is provided a flattened list of hundreds of files in its system prompt.
2. **Global Semantic Search / Vector Databases**: Code snippets are retrieved based on embedding similarity.

Both approaches break down under real-world autonomous multi-agent workloads:

### Problem A: Context Attenuation and Distance Decay
Attention mechanisms in Large Language Models do not treat 100,000 tokens uniformly. Information placed in the middle of massive context prompts suffers from "lost in the middle" degradation. When an agent is deep inside `services/billing/tax/rates.rs`, having a 10,000-line global directory tree in the top prompt provides virtually zero structural constraint. The agent forgets that `services/billing` enforces an invariant like "Never use floating point numbers for currency calculations."

### Problem B: Semantic Similarity Misses Architectural Topology
Embedding search matches keywords and concepts, but it has no understanding of hierarchical authority. Searching for "authenticate user" will pull snippets from frontend auth forms, database models, and test mocks simultaneously. It does not inform the agent: "You are currently in the infrastructure layer, and the core domain layer above you dictates that all auth tokens must be signed with Ed25519."

---

## 2. The Multi-Agent Race Condition

As AI engineering evolves from single-turn chat into swarms of specialized agents (planners, coders, reviewers, auditors, debuggers), coordination becomes the central failure point.

### The Problem of Blind Overwrites
Agent 1 (Coder) starts modifying `src/parser.rs` to support streaming tokens.
Agent 2 (Optimizer) simultaneously inspects the repository, notices slow parsing in `src/parser.rs`, and rewrites the function using SIMD intrinsics.
Neither agent knows the other is actively working there. When Agent 1 writes its changes, it destroys Agent 2's work.

### Centralized State Silos Fail
Storing this state in a centralized database or proprietary SaaS platform creates friction:
- Every agent framework needs custom API tokens and network access.
- Human developers cannot inspect the state using standard shell tools (`cat`, `ls`, `git diff`).
- If the repository is cloned to another machine, the memory vanishes.

---

## 3. The Crumb Solution: Pheromone-Style Filesystem Anchoring

In nature, ant colonies coordinate massive collective construction projects without a centralized brain. They do this through **stigmergy**: individuals modify the physical environment (laying down pheromone trails), and peer individuals respond to the modified environment.

The Crumb Protocol applies stigmergy to software engineering:

```
[Agent A] ──Writes File──> [Directory] ──Leaves Scent──> [.crumb.local]
                                                              │
                                                        [Agent B Reads]
                                                              │
                                                    [Acts with Context]
```

### Benefit 1: Zero-Shot Architectural Grounding
When an agent `cd`s or opens a file in a subdirectory, reading `.crumb` immediately answers:
- What is this directory responsible for?
- What are the absolute rules and invariants here?
- What is the parent domain (`above`)?
- What subcomponents exist (`below`)?

This takes less than 200 tokens of context, yet provides 100% precision.

### Benefit 2: Decentralized, Non-Blocking Synchronization
Through `.crumb.local`:
- Agent A registers a lock on `src/parser.rs`.
- Agent B sniffs the directory, sees the active lock and Agent A's intent, and chooses a different task or leaves a whisper: *"Agent A: ensure you maintain zero-copy slices for the tokenizer."*
- When Agent A finishes, it removes the lock and appends an action entry to `history`.

### Benefit 3: Version Control Native
Because `.crumb` is stored in git, architectural boundaries evolve alongside code. When a senior architect commits a `.crumb` with an invariant like `"Zero plaintext secrets: hardware TPM vault only"`, every autonomous agent that touches that directory in the future inherits that invariant automatically.
