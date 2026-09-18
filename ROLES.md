# The Agent Crumb Role and Behavioral Protocol

This document defines how autonomous agents, coding assistants, and subagents must behave when operating inside a Crumb-enabled codebase.

---

## 1. The Four Operational Phases

Every interaction with a codebase follows a four-step lifecycle:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  1. Sniff    │ ──> │   2. Lease   │ ──> │  3. Execute  │ ──> │  4. Whisper  │
│ (Read Scent) │     │ (Advisory)   │     │ (Mutate/Test)│     │  & Scent     │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

### Phase 1: Sniff
Before opening or mutating files in directory `D`:
1. **Inspect `.crumb`**:
   - Verify directory role and boundary.
   - Read and respect `invariants`. If your planned change violates an invariant, stop immediately.
   - Note the `above` reference to understand parent expectations.
2. **Inspect `.crumb.local`**:
   - Check `locks` for existing claims on target files.
   - Read `whispers` for notes left by prior agents.
   - Read `active_scents` to identify peer agents working concurrently.

### Phase 2: Lease (Advisory Lock)
If performing a multi-step operation or modifying shared files:
1. Open `.crumb.local` (create if absent).
2. Insert an entry under `locks`:
   ```json
   "src/service.rs": {
     "holder": "<AGENT_ID>",
     "intent": "Refactoring connection pool timeout handling",
     "acquired_at": "2026-09-18T20:30:00Z",
     "ttl_seconds": 1800
   }
   ```
3. Update `active_scents` with your current focus.

### Phase 3: Execute
Perform the necessary code generation, refactoring, testing, or debugging while preserving the invariants declared in `.crumb`.

### Phase 4: Whisper and Scent Dropping
Upon task completion, pause, or delegation:
1. **Release Lock**: Remove your lock from `locks` in `.crumb.local`.
2. **Append History Vector**: Record what was done:
   ```json
   {
     "agent": "<AGENT_ID>",
     "action": "modify",
     "target": "src/service.rs",
     "intent": "Refactor pool timeouts",
     "vector": "performance-resilience",
     "timestamp": "2026-09-18T20:42:00Z"
   }
   ```
3. **Leave Whispers for Succeeding Agents**:
   If a test failed, an assumption needs review, or work was partially delegated:
   ```json
   {
     "from": "<AGENT_ID>",
     "to": "ReviewerAgent",
     "message": "Connection pooling timeouts verified under load. Ready for PR review.",
     "target_file": "src/service.rs",
     "timestamp": "2026-09-18T20:42:10Z"
   }
   ```

---

## 2. Agent Conflict Resolution Rules

When an agent sniffs a directory and finds an active lock held by another agent:

1. **Lock Expired (`now - acquired_at > ttl_seconds`)**:
   - The agent may break the stale lock, remove the entry, and proceed.
2. **Lock Active on Same Target**:
   - The agent MUST NOT overwrite the locked file without coordination.
   - The agent SHOULD switch to an independent task, or leave a high-priority whisper requesting release.
3. **Lock Active on Different Target in Same Directory**:
   - The agent MAY proceed on its non-conflicting file, provided it respects the broader directory scent.

---

## 3. Subagent Handoff Protocol

When an orchestrator spawns subagents:
1. The orchestrator leaves a whisper in the root directory specifying subagent IDs and assigned tasks.
2. Each subagent navigates to its assigned target directory, sniffs the `.crumb`, and registers its own scent in `.crumb.local`.
3. When the subagent completes its goal, it writes a completion whisper and terminates cleanly.
4. The parent orchestrator sniffs the target directory to verify the completion receipt.
