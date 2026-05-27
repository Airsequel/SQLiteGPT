.PHONY: help build debug clean test demo

help:
	@echo "Targets:"
	@echo "  build  - Release build of libsqlitegpt"
	@echo "  debug  - Debug build of libsqlitegpt"
	@echo "  test   - cargo test"
	@echo "  demo   - Load extension and run a sample query (needs OPENAI_API_KEY)"
	@echo "  clean  - Remove target/"

build:
	cargo build --release

debug:
	cargo build

test:
	cargo test --release

demo: build
	sqlite3 :memory: \
	  ".load $(CURDIR)/target/release/libsqlitegpt" \
	  "SELECT gpt('Name of the tallest mountain, no explanation');"

clean:
	cargo clean
