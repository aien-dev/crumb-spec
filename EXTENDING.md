# Extending the Crumb Protocol

The Crumb Protocol is designed to be fully extensible. Anyone can create custom metadata schemas, domain-specific extensions, and agent tool bindings without breaking standard compatibility.

---

## 1. Using the `extensions` Block

Both `.crumb` and `.crumb.local` reserve an optional top-level `extensions` dictionary for custom metadata. Standard parsers will preserve these fields unchanged during read-modify-write operations.

### Example: Security and Compliance Extension
```json
{
  "schema_version": "1.0.0",
  "name": "payment-vault",
  "purpose": "Encrypted credit card token storage",
  "invariants": ["Zero plaintext cards"],
  "extensions": {
    "security": {
      "pci_dss_level": 1,
      "audit_required_on_mutation": true,
      "sanitization_routine": "crate::vault::sanitize_payload"
    }
  }
}
```

### Example: ML and Weights Extension
```json
{
  "schema_version": "1.0.0",
  "name": "vision-encoder",
  "purpose": "SigLIP2 vision encoder weights and forward pass",
  "extensions": {
    "ml": {
      "framework": "Modular MAX",
      "model_type": "bfloat16",
      "kv_cache_dim": 128,
      "supported_devices": ["cuda", "grace-blackwell"]
    }
  }
}
```

---

## 2. Creating Custom Scent Types in `.crumb.local`

You can extend `active_scents` with custom telemetry:
```json
{
  "active_scents": {
    "BenchmarkBot": {
      "focus": "Running continuous inference latency sweeps",
      "updated_at": "2026-09-18T20:45:00Z",
      "ttl_seconds": 600,
      "metadata": {
        "batch_size": 32,
        "current_throughput_tok_per_sec": 482.5
      }
    }
  }
}
```

---

## 3. Integrating with Existing Agent Frameworks

### In LangChain / LangGraph
Add a pre-tool hook that reads `.crumb` from the current working directory and prepends `invariants` into the agent's scratchpad.

### In Claude Code / Cursor / Windsurf
Add a prompt rule or system instruction:
> "Before editing any file, read .crumb and .crumb.local in the enclosing folder. Obey all declared invariants and do not modify files held under an active lock."

### In Custom Rust / Python Pipelines
Use the reference readers in `examples/reference-clients/` to integrate sniffing directly into tool calling loops.
