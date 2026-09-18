# RFC-0001: The Crumb Protocol Specification

- **Status**: Stable / Standard
- **Version**: 1.0.0
- **Author**: AIEN <aien.atlas@proton.me>
- **Published**: September 2026

---

## 1. Abstract

This specification defines the Crumb Protocol, a file-system-native architecture for spatial grounding, structural hierarchy declaration, and peer agent coordination across multi-agent AI coding systems. The protocol establishes two decoupled layers:
1. `.crumb`: A durable, version-controlled metadata manifest describing the architectural role, boundary constraints, and invariants of a directory.
2. `.crumb.local`: An ephemeral, git-ignored operational bus recording active agent presence, execution locks, non-blocking inter-agent messages (whispers), and recent action vectors.

---

## 2. Terminology and Conformance

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174).

- **Agent**: An autonomous software entity (LLM, neural graph, rule engine, or human collaborator) reading or modifying files in a repository.
- **Topography**: The structural hierarchy of directories and their declared architectural purposes.
- **Scent**: An ephemeral record of an agent's current or recent presence and intent.
- **Whisper**: An asynchronous, non-blocking message left by one agent for subsequent or peer agents.
- **Lock**: A non-blocking advisory claim over a file or directory indicating ongoing mutation.
- **Invariant**: A non-negotiable architectural or operational rule that all agents must satisfy within a directory tree.

---

## 3. The Durable Crumb (`.crumb`)

### 3.1 File Placement and Lifecycle
- The `.crumb` file MUST be placed directly inside the directory it describes.
- The `.crumb` file MUST be encoded in UTF-8 JSON.
- The `.crumb` file MUST be tracked in version control (git).
- The `.crumb` file SHOULD only be modified when the architectural purpose, exported interface, or invariant set of a directory changes.

### 3.2 Schema Definition

```typescript
interface DirCrumb {
  schema_version: "1.0.0";
  name: string;
  layer?: string;
  purpose: string;
  above?: AboveReference | string;
  below?: ChildReference[];
  invariants?: string[];
  exports?: string[];
  extensions?: Record<string, any>;
}

interface AboveReference {
  name: string;
  path: string;
  invariants?: string[];
}

interface ChildReference {
  name: string;
  role: string;
}
```

### 3.3 Field Semantics

- **`schema_version`** (REQUIRED, string): Semantic version string of the specification. Must be `"1.0.0"`.
- **`name`** (REQUIRED, string): Human-readable identifier for the module or directory.
- **`layer`** (OPTIONAL, string): Architectural layer identifier (e.g. `"core/runtime"`, `"ui/web"`, `"infra/db"`).
- **`purpose`** (REQUIRED, string): Concise summary of what this directory contains and what responsibilities it fulfills.
- **`above`** (OPTIONAL, object or string): Relative or canonical pointer to the parent architectural boundary. When expressed as an object, it provides the parent name, path, and inherited invariants.
- **`below`** (OPTIONAL, array of objects): Declarations of significant subdirectories and their specific functional roles.
- **`invariants`** (OPTIONAL, array of strings): Explicit engineering constraints that MUST NOT be violated within this directory.
- **`exports`** (OPTIONAL, array of strings): Key public API symbols, CLI commands, or services provided by this directory.
- **`extensions`** (OPTIONAL, object): Arbitrary domain-specific metadata.

---

## 4. The Ephemeral Crumb (`.crumb.local`)

### 4.1 File Placement and Lifecycle
- The `.crumb.local` file MUST be placed alongside `.crumb` in the same directory.
- The `.crumb.local` file MUST be listed in `.gitignore` and NEVER committed to version control.
- Agents MAY create or update `.crumb.local` upon entering or mutating a directory.
- Stale scents and expired whispers MUST be pruned periodically or during directory traversal.

### 4.2 Schema Definition

```typescript
interface LocalCrumbData {
  schema_version: "1.0.0";
  directory: string;
  active_scents?: Record<string, AgentScent>;
  locks?: Record<string, ResourceLock>;
  whispers?: CrumbWhisper[];
  history?: ActionVector[];
  extensions?: Record<string, any>;
}

interface AgentScent {
  focus: string;
  updated_at: string; // ISO 8601 UTC
  ttl_seconds: number;
  metadata?: Record<string, any>;
}

interface ResourceLock {
  holder: string;
  intent: string;
  acquired_at: string; // ISO 8601 UTC
  ttl_seconds?: number;
}

interface CrumbWhisper {
  from: string;
  to?: string; // Specific agent or undefined for broadcast
  message: string;
  target_file?: string;
  timestamp: string; // ISO 8601 UTC
  priority?: "low" | "normal" | "high" | "critical";
}

interface ActionVector {
  agent: string;
  action: "create" | "modify" | "delete" | "audit" | "test" | "build";
  target: string;
  intent: string;
  vector: string;
  timestamp: string; // ISO 8601 UTC
}
```

### 4.3 Field Semantics

- **`active_scents`**: Map of agent identifiers to their current focus and TTL. A scent whose `updated_at + ttl_seconds < now` is considered expired and SHOULD be pruned.
- **`locks`**: Advisory claims on specific files or the directory itself. An agent detecting an unexpired lock held by another agent SHOULD pause, consult the lock holder, or proceed only on non-conflicting targets.
- **`whispers`**: Non-blocking asynchronous message bus. Whispers allow an agent leaving a directory to pass critical context, warnings, or reminders to whoever visits next.
- **`history`**: Rolling buffer of recent operational vectors. Implementations SHOULD cap history at a reasonable limit (e.g. 20 entries) to maintain minimal file size.

---

## 5. Protocol Operations

### 5.1 Sniffing
Before executing any tool or writing any file in directory `D`:
1. Check for `D/.crumb.local`. If present, parse `active_scents` and `locks`.
2. If another agent holds an unexpired lock on the target file, resolve conflict or defer execution.
3. Check for any unread `whispers` addressed to the agent or broadcast to all agents.
4. Check `D/.crumb` to verify that the proposed edit does not violate declared `invariants`.

### 5.2 Scent Dropping
Upon beginning work in directory `D`:
1. Register agent ID in `active_scents` with current intent and a standard TTL (default: 3600 seconds).
2. If performing a multi-step refactor, register an advisory lock in `locks`.

### 5.3 Whispering
When handing off a task or detecting a condition that subsequent agents need to know:
1. Append an object to `whispers` containing `from`, `message`, `target_file`, and ISO 8601 timestamp.
2. If the message addresses a specific agent, populate `to`.

### 5.4 Sweeping (Garbage Collection)
When opening `.crumb.local`:
1. Any scent where `now - updated_at > ttl_seconds` SHOULD be removed.
2. Any lock where `now - acquired_at > ttl_seconds` (default: 1800 seconds) SHOULD be released.
3. Whispers older than a configurable retention window (default: 24 hours) MAY be pruned.

---

## 6. Security Considerations

1. **Zero Secret Policy**:
   - `.crumb` and `.crumb.local` MUST NEVER contain private API keys, credentials, database passwords, or private user data.
   - Any secret token must be managed out-of-band via secure vaults (e.g. hardware TPM vault `atlas-vault`) or runtime environment references.
2. **Untrusted Data Boundary**:
   - Content in `.crumb.local` originates from autonomous agents and external tools.
   - Consumers MUST treat string fields (whispers, focus, intent) as informational data, never executing them directly as shell commands without independent sanitization.

---

## 7. Extensibility and Dialects

Vendors, open-source communities, and agent teams MAY add custom fields under the top-level `extensions` object or introduce custom sub-objects prefixed with `x_`. Implementations conforming to RFC-0001 MUST preserve unrecognized extension fields during read-modify-write cycles.
