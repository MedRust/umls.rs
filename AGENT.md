# AGENT.md — Engineering Guidelines for `umls.rs`

This document defines the architecture, design principles, coding standards, testing practices, and review guidelines for AI agents and human contributors working within the `umls.rs` workspace.

---

## 1. Workspace Overview

`umls.rs` is a high-performance, asynchronous Rust ecosystem for querying medical concepts, atoms, relations, and downloading releases from the **U.S. National Library of Medicine (NLM) UMLS Terminology Services (UTS)**.

### Workspace Crates

| Crate | Purpose | Core Dependencies |
| :--- | :--- | :--- |
| [`umls-client`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-client) | High-level asynchronous client for UMLS REST APIs (concepts, atoms, relations, search, pagination). | `http`, `secrecy`, `serde`, `thiserror`, `tokio`, `url` |
| [`umls-http`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-http) | Abstraction layer for HTTP transport and typestate markers (`WithoutClient`). Feature-gated `reqwest` transport. | `bytes`, `http`, `reqwest` (optional), `tokio` |
| [`umls-downloader`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-downloader) | Library for discovering and streaming downloads of UMLS, RxNorm, and SNOMED CT releases. | `async-trait`, `futures-util`, `umls-http` |
| [`umls-downloader-cli`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-downloader-cli) | Terminal interactive CLI for downloading releases with progress tracking and cancellation. | `clap`, `crossterm`, `indicatif`, `umls-downloader` |
| [`umls-mcp`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/umls-mcp) | Model Context Protocol (MCP) server providing LLMs with tools for UMLS concept and relation retrieval. | `axum`, `rmcp`, `umls-client`, `schemars` |

---

## 2. Rust Toolchain & Workspace Lints

- **Rust Edition**: `2024` across all crates.
- **Resolver**: `3`.
- **Workspace Lints** (configured in root [`Cargo.toml`](file:///Volumes/BitBhandar/Languages/Hosted/Rust/Cargo/umls.rs/Cargo.toml)):
  ```toml
  [workspace.lints.rust]
  unsafe_code = "forbid"
  missing_docs = "warn"

  [workspace.lints.rustdoc]
  broken_intra_doc_links = "deny"
  ```
  - **Zero Unsafe Code**: Never use `unsafe`. If an operation cannot be expressed safely, reconsider the design.
  - **Exhaustive Documentation**: All `pub` items (crates, modules, structs, enums, methods, traits) must have docstrings.
  - **Clean Intra-Doc Links**: Every link in doc comments (e.g. `[`Config::builder`]`) must resolve to a valid item.

---

## 3. API Design Principles

### 3.1 Typestate Pattern for Compile-Time Safety
Use the typestate pattern to enforce lifecycle phases and prevent runtime errors (e.g., executing a request before attaching a client):
- **Uninitialized State**: Requests or builders start in a state such as `Relation<WithoutClient, V>` or `Atom<WithoutClient, ...>`.
- **Initialized State**: Calling `.with_client(client)` transitions the builder to `Relation<reqwest::Client, V>`, which enables execution methods like `.fetch()` or `.get()`.
- **Batch / Pagination States**: Pagination builders encode current page state (`All`, `Batch<T>`) directly into the type parameters.

### 3.2 Decoupling Configuration and HTTP Transport
- **Configuration** (`Config`) owns only authentication credentials, target release versions, and base URLs.
- **Client Ownership**: `Config` does **not** own the HTTP client. An HTTP client (e.g. `reqwest::Client`) is passed or cloned into the request builder.
- **Connection Reuse**: Callers maintain their own client instances to reuse connection and DNS pools across multiple requests.
- **Feature Gating**: All network-executing methods and client implementations must be conditionally compiled under `#[cfg(feature = "reqwest")]`.

### 3.3 Domain Modeling & Type-Safe Identifiers ("Parse, Don't Validate")
Avoid primitive obsession (`String`, `&str`) for domain concepts:
- **Newtypes**: Wrap identifiers in dedicated types like `Cui` (Concept Unique Identifier), `Aui` (Atom Unique Identifier), and `DisplayName`.
- **Strict Validation at Ingestion**: Validate invariants in constructors (`new()`, `from_str()`, `TryFrom`). Once created, a `Cui` is guaranteed valid:
  - Begins with uppercase `'C'`.
  - Followed by exactly 7 ASCII digits (e.g. `C0010200`).
  - Whitespace is trimmed, but non-conforming formats are rejected.
- **Idiomatic Trait Implementations**: Always implement:
  - `Display`
  - `AsRef<str>`
  - `FromStr`
  - `TryFrom<&str>` and `TryFrom<String>`
  - `Deref<Target = str>` (when ergonomic and transparent borrowing is intended)

### 3.4 Secret Management
- Sensitive values (API keys) **must** use `secrecy::SecretString`.
- Never log, format (`Display`, `Debug`), or serialize raw API keys.
- Only expose secrets at the boundary of HTTP request header creation using `.expose_secret()`.

### 3.5 Serde & Wire-Format Handling
The upstream UMLS REST API has quirks (e.g., JSON booleans sent as strings `"true"`/`"false"`, null values represented as `"NONE"`).
- **CamelCase Renaming**: Always specify `#[serde(rename_all = "camelCase")]` on DTO structs matching UTS REST responses.
- **Modular Custom Serde Handlers**: Use dedicated modules with `#[serde(with = "...")]` rather than ad-hoc functions:
  ```rust
  #[cfg(feature = "reqwest")]
  mod bool_as_string {
      use serde::{de, Deserializer, Serializer};

      pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
      where
          S: Serializer,
      {
          serializer.serialize_str(if *value { "true" } else { "false" })
      }

      pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
      where
          D: Deserializer<'de>,
      {
          let value = String::deserialize(deserializer)?;
          match value.as_str() {
              "true" => Ok(true),
              "false" => Ok(false),
              _ => Err(de::Error::custom("expected \"true\" or \"false\"")),
          }
      }
  }
  ```
- **Sentinel `NONE` String Handling**:
  ```rust
  #[cfg(feature = "reqwest")]
  mod optional_string {
      use serde::{Deserialize, Deserializer, Serializer};

      pub fn serialize<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
      where
          S: Serializer,
      {
          serializer.serialize_str(value.as_deref().unwrap_or("NONE"))
      }

      pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
      where
          D: Deserializer<'de>,
      {
          let value = Option::<String>::deserialize(deserializer)?;
          Ok(match value {
              Some(val) if val == "NONE" => None,
              other => other,
          })
      }
  }
  ```

### 3.6 Error Handling
- Use `thiserror` (version 2) for domain error definitions.
- Mark public error enums with `#[non_exhaustive]` to allow future extensions without breaking changes.
- Preserve underlying cause chains with `#[source]` or `#[from]`.
- Provide user-friendly, descriptive error messages.
- Never panic (`unwrap()`, `expect()`) in production library code. Always propagate errors via `Result<T, Error>`.

---

## 4. Documentation Standards

Because `missing_docs = "warn"` and `broken_intra_doc_links = "deny"` are active:

1. **Module & Crate Level Docs**:
   - Every file must start with module-level documentation `//!`.
   - Provide an architectural overview, usage summary, and a working doctest.
2. **Item-Level Docs**:
   - Every public struct, enum, variant, trait, and function must have a docstring `///`.
   - Methods should include `# Errors` sections whenever they return `Result`.
3. **Doctests**:
   - Keep doctests runnable.
   - For code requiring the network feature, hide setup behind `#`:
     ```rust
     /// # Example
     ///
     /// ```no_run
     /// # #[cfg(feature = "reqwest")]
     /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
     /// use umls_client::{config::Config, core::cui::Cui};
     ///
     /// let config = Config::builder("api-key").build()?;
     /// let cui = Cui::new("C0010200")?;
     /// # Ok(())
     /// # }
     /// ```
     ```
   - Use `no_run` for doctests requiring network or API keys.

---

## 5. Examples Guidelines (`examples/`)

Runnable examples live under crate-level `examples/` directories (e.g. `umls-client/examples/`).

1. **Feature Gate Registration**:
   Add every example to `Cargo.toml`:
   ```toml
   [[example]]
   name = "fetch_concept_relations"
   required-features = ["reqwest"]
   ```
2. **CLI & Environment Parameter Support**:
   - Use `clap` to accept arguments with fallback to standard environment variables:
     - `UMLS_BASE_URL` (`--base-url`)
     - `UMLS_API_KEY` (`--api-key`)
     - `UMLS_VERSION` (`--version`)
   - Share common parsing logic across examples using an internal `common.rs` module.
3. **Self-Documenting Headers**:
   Include a markdown table and command line usage instructions in the doc comment at the top of the example file.

---

## 6. Unit & Integration Testing Guidelines

### 6.1 Unit Tests
- Place unit tests inside a nested `mod tests` block at the bottom of the source file under `#[cfg(test)]`.
- Test domain boundaries exhaustively:
  - Parsing valid and invalid variants.
  - Leading/trailing whitespace trimming.
  - Rejection of invalid characters (e.g. non-ASCII digits, unexpected prefixes).
  - Empty strings or malformed encodings.
- Test standard trait implementations (`FromStr`, `TryFrom`, `Display`, `AsRef`).

### 6.2 Integration Tests
- Place integration tests in the `tests/` directory of each crate.
- **Mocking**: Always use `httpmock` for HTTP mocking.
- **Async Execution**: Use `#[tokio::test]`.
- **Feature Flags**: Add `#![cfg(feature = "reqwest")]` to tests that depend on network client features.
- **Determinism**: Never hit the live UTS API in CI tests. Mock endpoint paths, query parameters, auth headers, and realistic JSON payloads.
- **Round-Trip Serialization Tests**: Verify that types serialize and deserialize cleanly against realistic UMLS wire payloads.

---

## 7. MCP Server Guidelines (`umls-mcp`)

When adding or updating tools in `umls-mcp`:
1. **Tool Annotations**: Always define annotations for Model Context Protocol tools:
   ```rust
   #[tool(
       annotations(
           read_only_hint = true,
           destructive_hint = false,
           idempotent_hint = true,
           open_world_hint = true
       ),
       description = "Retrieve UMLS concept information by CUI"
   )]
   ```
2. **Schemas**: Derive `schemars::JsonSchema` on all tool argument structs alongside `Deserialize`.
3. **Structured Tool Responses**:
   - Use `success(&value)` which serializes the response into a JSON text content block.
   - Use `failure("clear error message")` for validation or upstream errors. Do not leak credentials in error messages.

---

## 8. Code Review & Verification Checklist

Before opening a PR or completing a coding task, run and verify the following commands from the workspace root:

```bash
# 1. Compile workspace with all features
cargo check --all-targets --all-features

# 2. Run all unit and integration tests
cargo test --all-targets --all-features

# 3. Verify doctests compile and pass
cargo test --doc --all-features

# 4. Run Clippy with zero warnings allowed
cargo clippy --all-targets --all-features -- -D warnings

# 5. Check formatting
cargo fmt --all -- --check
```

### Reviewer Checks
- [ ] **No `unsafe`**: Zero unsafe blocks in the codebase.
- [ ] **No Unused Code**: No unused imports, variables, or dead code (`warn(unused)`).
- [ ] **Doc Comments & Links**: All public items documented; no broken intra-doc links.
- [ ] **Secret Safety**: API keys wrapped in `SecretString` and never leaked in logs.
- [ ] **Serde Symmetry**: Fields with custom serialization round-trip correctly.
- [ ] **Errors Handled**: No `.unwrap()` or `.expect()` in non-test library code.
- [ ] **Code Coverage**: Use unit and integration tests to find the what portion of the code is not tested and notify it.
