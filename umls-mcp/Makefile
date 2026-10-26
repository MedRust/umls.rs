CARGO_FLAGS := -q

build:
	@echo "=> Building MCP server..."
	@cargo build $(CARGO_FLAGS)

test:
	@echo "=> Testing MCP server..."
	@cargo test $(CARGO_FLAGS)

server:
	@echo "=> Running MCP server..."
	@cargo run $(CARGO_FLAGS) -- --transport http
