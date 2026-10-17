
build: build-umls-mcp build-umls-http build-umls-client build-umls-downloader build-umls-downloader-cli

build-umls-client:
	@echo "=> Building 'umls-client' crate"
	@make -C umls-client build

build-umls-downloader:
	@echo "=> Building 'umls-downloader' crate"
	@make -C umls-downloader build

build-umls-downloader-cli:
	@echo "=> Building 'umls-downloader-cli' crate"
	@make -C umls-downloader-cli build

build-umls-http:
	@echo "=> Building 'umls-http' crate"
	@make -C umls-http build

test: test-umls-mcp test-umls-http test-umls-client test-umls-downloader test-umls-downloader-cli

test-umls-client:
	@echo "=> Testing 'umls-client' crate"
	@make -C umls-client test

test-umls-downloader:
	@echo "=> Testing 'umls-downloader' crate"
	@make -C umls-downloader test

test-umls-downloader-cli:
	@echo "=> Testing 'umls-downloader-cli' crate"
	@make -C umls-downloader-cli test

test-umls-http:
	@echo "Testing 'umls-http' crate"
	@make -C umls-http test

downloader:
	@echo "=> Running UMLS downloader"
	@make -C umls-downloader-cli run

build-umls-mcp:
	@make build -C umls-mcp

test-umls-mcp:
	@make test -C umls-mcp

mcp:
	@make server -C umls-mcp

clean:
	@echo "=> Removing artifacts..."
	@cargo clean
