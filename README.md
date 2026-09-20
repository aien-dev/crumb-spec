# The Crumb Protocol (RFC-0001)

An open, vendor-neutral filesystem standard for autonomous AI agent spatial grounding, architectural memory, and peer pheromone coordination.

Free and open for anyone to adopt, implement, extend, and embed across all agent frameworks, IDEs, and autonomous developer swarms.

```
       ┌────────────────────────────────────────────────────────┐
       │                   The Crumb Paradigm                   │
       ├────────────────────────────┬───────────────────────────┤
       │     Durable Topography     │     Ephemeral Scent       │
       │          (`.crumb`)        │      (`.crumb.local`)     │
       ├────────────────────────────┼───────────────────────────┤
       │ • Checked into git         │ • Ignored by git          │
       │ • Defines directory purpose│ • Real-time agent scents  │
       │ • Spatial parent (`above`) │ • Inter-agent whispers    │
       │ • Architecture invariants  │ • Read/write leases       │
       │ • Exported interfaces      │ • Rolling action vectors  │
       └────────────────────────────┴───────────────────────────┘
```

---

## The Problem: Agent Amnesia and Context Collapse

Existing AI software development agents suffer from three structural flaws:

1. **Context Collapse Across Subdirectories**: LLM context windows degrade when stuffed with arbitrary file trees. When an agent enters a nested subdirectory, it loses awareness of parent boundaries, domain invariants, and global architecture.
2. **Multi-Agent Race Conditions**: When multiple autonomous agents (coder, reviewer, verifier, tester) operate simultaneously in a repository, they have no shared physical presence. Agents blind-overwrite each other's work, duplicate tasks, or break assumptions established by a peer.
3. **Database Disconnect**: External vector databases or centralized state stores create an out-of-band dependency. When a human developer or another machine clones the repository, that external memory is absent.

---

## The Solution: Filesystem-Native Memory

The Crumb Protocol solves these flaws by anchoring agent intelligence directly into the filesystem itself using two lightweight JSON files placed in relevant directories:

- **`.crumb` (Durable Repository Topology)**: Checked into source control. Establishes the purpose of the directory, its architectural layer, explicit non-negotiable invariants, and pointers to parent and child domains.
- **`.crumb.local` (Ephemeral Agent Pheromones)**: Kept in `.gitignore`. Acts as a live pheromone trail tracking which agents are currently active, what intent vectors they are executing, non-blocking inter-agent whispers, and operational locks.

---

## Core Specification Overview

Full details are documented in [SPEC.md](SPEC.md).

### 1. The Durable Crumb (`.crumb`)

Located at directory roots. Must be valid JSON matching [schema/crumb.schema.json](schema/crumb.schema.json).

```json
{
  "schema_version": "1.0.0",
  "name": "spark-adapters",
  "layer": "core/runtime",
  "purpose": "Provider adapters for frontier LLMs and local MAX distillation",
  "above": {
    "name": "aien-sovereign-core",
    "path": "../../",
    "invariants": ["Pure native Rust", "Zero disk secrets"]
  },
  "below": [
    { "name": "providers", "role": "API translation modules for third-party endpoints" },
    { "name": "distill", "role": "Dual-rollout consensus engine" }
  ],
  "invariants": [
    "All network calls must support cancellation tokens",
    "Secrets must resolve dynamically via hardware TPM vault, never .env files"
  ],
  "exports": [
    "spark_adapters::router::route_completion",
    "spark_adapters::distill::run_dual_rollout"
  ]
}
```

### 2. The Ephemeral Crumb (`.crumb.local`)

Located alongside `.crumb`. Never committed to git. Must match [schema/crumb-local.schema.json](schema/crumb-local.schema.json).

```json
{
  "schema_version": "1.0.0",
  "directory": "crates/spark-adapters",
  "active_scents": {
    "AIEN": {
      "focus": "Refactoring OpenAI adapter error handling",
      "updated_at": "2026-09-18T20:30:00Z",
      "ttl_seconds": 3600
    }
  },
  "locks": {
    "src/router.rs": {
      "holder": "AIEN",
      "intent": "Adding Anthropic Claude 3.7 hybrid reasoning token schema",
      "acquired_at": "2026-09-18T20:28:15Z"
    }
  },
  "whispers": [
    {
      "from": "VerifierAgent",
      "message": "Verify test_anthropic_thinking passes before merging router changes",
      "target_file": "src/router.rs",
      "timestamp": "2026-09-18T20:25:00Z"
    }
  ],
  "history": [
    {
      "agent": "AIEN",
      "action": "modify",
      "target": "src/router.rs",
      "intent": "Implement adaptive temperature scaling",
      "vector": "inference-stability",
      "timestamp": "2026-09-18T20:20:00Z"
    }
  ]
}
```

---

## The Four Axioms of the Crumb Rule

Any autonomous agent operating in a Crumb-enabled repository must adhere to four axioms:

1. **Sniff Before Touch**: Before opening or modifying any file, the agent must check for `.crumb` and `.crumb.local`. If a peer agent holds an active lock on the file, the agent must yield, collaborate, or wait.
2. **Respect Invariants**: The `invariants` block in `.crumb` is authoritative. An agent must never introduce code, dependencies, or patterns that violate declared directory invariants.
3. **Leave a Scent**: When starting a non-trivial task, the agent registers its presence in `active_scents`. When finishing or pausing, it appends an action entry to `history`.
4. **Whisper on Transition**: When delegating work to a subagent or completing a step that requires review, the agent leaves an explicit message in `whispers`.

---

## How to Adopt and Extend

The Crumb Protocol is intentionally simple and dependency-free. You do not need any specific library to read or write crumbs:

- **Any language can parse it**: It is standard JSON.
- **Reference Clients**: See [examples/reference-clients/](examples/reference-clients/) for zero-dependency Python, TypeScript, Shell, and Rust implementations.
- **CLI Tooling**: A compiled native Rust implementation is available at [spark-crumbs](https://github.com/aien-dev/spark-crumbs).
- **Custom Extensions**: Teams can add custom keys under `extensions` or create new dialects. See [EXTENDING.md](EXTENDING.md).

---

## Documentation Roadmap

- [SPEC.md](SPEC.md): Formal RFC-0001 specification.
- [WHY.md](WHY.md): Deep architectural rationale comparing crumbs to vector search and prompt stuffing.
- [ROLES.md](ROLES.md): Complete operational manual for autonomous agents adopting the Crumb role.
- [EXTENDING.md](EXTENDING.md): Guide for creating custom crumb schemas and tool integrations.
- [SECURITY.md](SECURITY.md): Guidelines on credential isolation and zero disk secrets.

## License and Governance

Licensed under the **Sovereign Resource Commons License 1.0 (SRCL-1.0)** (Apache-2.0 WITH LLVM-exception).
Architected by AIEN (Autonomous Cognitive Architecture operating on the Atlas Framework) and sovereign ecosystem contributors. See [LICENSE](LICENSE) for full legal terms and copyright notices.

All downstream distributions, derivative works, and commercial deployments are governed exclusively by the terms of [LICENSE](LICENSE). [CONSTITUTION.md](CONSTITUTION.md) defines the internal architectural charter and development doctrine for upstream engineering.
