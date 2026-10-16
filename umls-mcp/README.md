# UMLS MCP Server

An MCP server exposing the existing `umls-client` crate to Rig and other MCP
clients over stdio or Streamable HTTP.

## Build and run

Use Rust 1.95 or newer. If your default toolchain is older, prefix the Cargo
commands below with `cargo +1.95.0` (for example,
`cargo +1.95.0 build -p umls-mcp`). The current client pagination implementation
requires this newer toolchain.

```sh
cargo build -p umls-mcp
export UMLS_API_KEY="your-UTS-key"
cargo run -p umls-mcp
```

## Streamable HTTP

```sh
export UMLS_API_KEY="your-UTS-key"
cargo run -p umls-mcp -- --transport http
```

Connect your MCP client to **`http://127.0.0.1:3000/mcp`**. The HTTP server runs
until Ctrl+C, which shuts down active sessions. To select another port:

```sh
cargo run -p umls-mcp -- --transport http --bind 127.0.0.1:8080
```

The endpoint supports MCP initialization, tool discovery, and tool calls using
the SDK’s Streamable HTTP transport, including SSE responses and legacy client
sessions. Configure a URL-based MCP client with this server URL (the exact
configuration format depends on your client):

```json
{ "mcpServers": { "umls": { "url": "http://127.0.0.1:3000/mcp" } } }
```

HTTP defaults to loopback and has no client authentication. Host validation
accepts loopback names and the explicitly bound IP. Browser origins must match
the bound HTTP address or `localhost` at the same port; requests without an
Origin header are accepted. For access beyond your machine, put the endpoint
behind an authenticated reverse proxy and configure its forwarded Host to the
bound address.

## Stdio

Stdio remains the default. Select it explicitly with `--transport stdio`.

The server waits for MCP messages on stdin. Stdout is reserved for protocol
messages; startup errors go to stderr. Configure your MCP client to launch the
built binary rather than entering requests in the terminal.

Example MCP client configuration (replace the path and key):

```json
{
  "mcpServers": {
    "umls": {
      "command": "/absolute/path/to/umls.rs/target/debug/umls-mcp",
      "env": { "UMLS_API_KEY": "your-UTS-key" }
    }
  }
}
```

Keep configuration containing credentials out of Git. A Rig application can
connect to this executable using its MCP client integration and discover these
tools; the server itself does not depend on Rig.

## Tools

| Tool | Arguments | Result |
| --- | --- | --- |
| `search_concepts` | `query`, optional `page`, `page_size` | Concept summaries and pagination |
| `get_concept` | `cui` | Name, status, semantic types, counts, dates |
| `get_concept_atoms` | `cui`, optional `page`, `page_size` | Source vocabulary atoms and pagination |

CUIs must be `C` followed by seven digits. Pages start at 1; page size defaults
to 25 and must be between 1 and 100. Tools return JSON text. Paginated responses
include `page`, `page_size`, `total_pages`, `has_next`, and `version`.
Unknown search totals are returned as `null`; `has_next` uses the client’s
pagination behavior. Each call fetches one page, never the entire result set.
Invalid inputs and upstream failures return MCP tool errors. Upstream error
details are withheld because request URLs contain API keys.

## Configuration and checks

- `UMLS_MCP_TRANSPORT`: `stdio` (default) or `http`; overridden by `--transport`.
- `UMLS_MCP_BIND`: HTTP socket address, default `127.0.0.1:3000`; overridden by `--bind`.
- `UMLS_API_KEY`: required, nonempty UTS API key.
- `UMLS_VERSION`: optional `current` (default) or release such as `2024AB`.
- `UMLS_BASE_URL`: optional REST base URL for a proxy or local tests; its path
  must end with `/`.

Requests use a shared HTTP connection pool and a 30-second timeout.

```sh
cargo test -p umls-mcp
cargo clippy -p umls-mcp --all-targets
```

Tests use local mock HTTP servers and require no real credentials.
