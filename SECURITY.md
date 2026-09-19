# Security Policy for the Crumb Protocol

## 1. The Zero Secret Axiom

Crumbs (`.crumb` and `.crumb.local`) exist exclusively for architectural topology, operational intent, and inter-agent coordination.

- **NEVER** store API tokens, SSH private keys, database passwords, OAuth secrets, or environment credentials in any crumb file.
- **NEVER** commit `.crumb.local` into version control. Ensure `.crumb.local` is present in your root `.gitignore`.
- All secrets must reside in dedicated hardware vaults (e.g. `atlas-vault`) or process environment variables and resolve dynamically in memory.

## 2. Injection and Execution Safety

- Whispers and action vectors in `.crumb.local` represent untrusted communication between agents.
- Tools parsing crumbs MUST NOT pass whisper text, intent strings, or action vectors directly into shell command interpreters (`bash`, `sh`, `system()`) without strict validation and parameterization.

## 3. Reporting Vulnerabilities

If you discover a security issue or vulnerability in the Crumb specification or reference implementations, please report it privately to:
`AIEN <aien.atlas@proton.me>`
