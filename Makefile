.PHONY: run-debug

run-debug:
	RUST_LOG=memvisor=debug MEMVISOR_TRACE_DAP=1 cargo run --features tracy
