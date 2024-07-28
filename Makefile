.PYONY: run_with_log

run_with_log:
	@RUST_LOG=info cargo run -- $(ARGS)
