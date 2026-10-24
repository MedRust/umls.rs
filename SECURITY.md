# Security Policy

The `umls.rs` project takes the security of its codebase, dependencies, and users seriously. This document outlines our security policies, supported versions, and procedures for reporting vulnerabilities.

---

## 1. Supported Versions

Security updates and patches are provided for the following versions:

| Version | Supported | Notes |
| :--- | :--- | :--- |
| `0.1.x` (Git `main`) | :white_check_mark: | Actively supported and maintained |
| `< 0.1.0` | :x: | Unsupported; please upgrade to the latest release |

We strongly recommend keeping all workspace crates ([`umls-client`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-client), [`umls-downloader`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-downloader), [`umls-downloader-cli`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-downloader-cli), [`umls-http`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-http), and [`umls-mcp`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-mcp)) updated to their latest published versions.

---

## 2. Reporting a Vulnerability

We appreciate the efforts of security researchers and community members in helping us keep `umls.rs` safe.

> [!IMPORTANT]
> **Please do not report security vulnerabilities via public GitHub issues, discussions, or pull requests.**

### How to Report Privately

1. **GitHub Security Advisory (Preferred)**:
   Navigate to the [GitHub Security Advisory page](https://github.com/MedRust/umls.rs/security/advisories/new) and click **"Report a vulnerability"** to open a confidential draft.
2. **Alternative Disclosure**:
   If the GitHub advisory reporting feature is unavailable, contact the project maintainers privately via the email associated with the [MedRust Organization](https://github.com/MedRust).

### What to Include in Your Report

To help us triage and resolve the issue quickly, please provide:
- A clear description of the potential vulnerability and its estimated severity.
- The affected crate(s) and version(s).
- Step-by-step instructions or a minimal reproducible example (PoC).
- Any relevant logs, panic outputs, or network trace details (ensure any real UMLS API keys are redacted).
- Proposed fixes or mitigations, if known.

### Response & Disclosure Process

1. **Acknowledgment**: We aim to acknowledge receipt of security reports within **48 hours**.
2. **Investigation & Triage**: Maintainers will verify the issue, assess the attack vector, and determine the blast radius.
3. **Patch Development & Testing**: Fixes will be developed privately in a temporary security fork.
4. **Coordinated Release & Advisory**: A patched release will be published to [crates.io](https://crates.io) along with a GitHub Security Advisory acknowledging the reporter (if desired).

---

## 3. Security Architecture & Commitments

The `umls.rs` workspace is engineered with the following security guarantees:

### 3.1 Zero `unsafe` Code
All crates in this workspace strictly enforce:
```toml
[workspace.lints.rust]
unsafe_code = "forbid"
```
No `unsafe` blocks are permitted in any crate, preventing memory corruption bugs, buffer overflows, use-after-free conditions, and data races at compile time.

### 3.2 Credential Protection (`secrecy`)
- UMLS API keys represent access credentials to the National Library of Medicine UTS systems.
- Sensitive keys are encapsulated using `secrecy::SecretString`.
- Secrets are zeroized on drop where applicable and explicitly protected against accidental leakage in `Debug`, `Display`, and standard serialization pipelines.
- Secrets are only exposed at the immediate HTTP client header construction boundary using `.expose_secret()`.

### 3.3 Safe File Download & Path Handling ([`umls-downloader`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-downloader))
- Download destinations are strictly validated to prevent directory traversal and arbitrary file write vulnerabilities.
- Downloads stream to staging files that are atomically renamed upon completion or purged immediately if cancelled or interrupted.

### 3.4 MCP Server Security ([`umls-mcp`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-mcp))
- Tools exposed to Model Context Protocol (MCP) clients specify explicit security hints (`read_only_hint = true`, `destructive_hint = false`, `idempotent_hint = true`).
- Error messages returned to LLM clients are sanitized using generic failure descriptions (`failure(...)`) to ensure raw upstream stack traces or configuration secrets are never reflected into model prompts.

### 3.5 Dependency & Supply Chain Auditing
- Dependencies are monitored and audited using `cargo-audit` to detect known CVEs in the Rust ecosystem.
- Releases and builds run on modern Rust (`edition = "2024"`) with minimal external dependencies.

---

## 4. Best Practices for Users and Integrators

When using `umls.rs` in production or applications:

1. **Protect Your UMLS API Key**:
   - Store API keys in environment variables (e.g. `UMLS_API_KEY`) or dedicated secret managers (e.g., AWS Secrets Manager, HashiCorp Vault).
   - Never commit API keys or `.env` files into source control.
2. **Enforce HTTPS**:
   - UTS REST APIs must always be reached over secure TLS (`https://uts-ws.nlm.nih.gov/rest/`).
   - If configuring custom endpoints via `base_url`, ensure they use HTTPS.
3. **Data Licensing & Compliance**:
   - UTS terminology data (UMLS, RxNorm, SNOMED CT) is subject to NLM and respective terminology authority license agreements. Users are responsible for adhering to all usage and distribution restrictions.
