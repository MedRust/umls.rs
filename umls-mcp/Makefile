RUST_FLAGS := -q

build:
	@echo "=> Building MCP server..."
	@cargo build $(RUST_FLAGS)

test:
	@echo "=> Testing MCP server..."
	@cargo test $(RUST_FLAGS)

server:
	@echo "=> Running MCP server..."
	@cargo run $(RUST_FLAGS) -- --transport http
