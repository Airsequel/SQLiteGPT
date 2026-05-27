.PHONY: help
help: makefile
	@tail -n +4 makefile | grep ".PHONY"


.PHONY: format
format:
	cargo clippy --fix --allow-dirty
	cargo fmt
	# nix fmt  # TODO: Reactivate when it's faster


.PHONY: build
build:
	cargo build --release


.PHONY: debug
debug:
	cargo build


.PHONY: test
test:
	cargo test -p sqlitegpt-core --release


.PHONY: demo
demo: build
	sqlite3 :memory: \
	  ".load $(CURDIR)/target/release/libsqlitegpt" \
	  "SELECT gpt('Name of the tallest mountain, no explanation');"


.PHONY: clean
clean:
	cargo clean
