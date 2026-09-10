CARGO_FLAGS := -q

build: build-without-features build-with-reqwest-feature

build-without-features:
	@echo "=> Building without features"
	@cargo build $(CARGO_FLAGS)

build-with-reqwest-feature:
	@echo "=> Building with 'reqwest' feature"
	@cargo build --features reqwest $(CARGO_FLAGS)

test: test-without-features test-with-reqwest-feature

test-without-features:
	@echo "=> Testing without features"
	@cargo test $(CARGO_FLAGS)

test-with-reqwest-feature:
	@echo "=> Testing with 'reqwest' feature"
	@cargo test --features reqwest $(CARGO_FLAGS)
