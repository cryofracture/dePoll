prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cd dePoll_v* && cargo build --release --target wasm32-unknown-unknown
	wasm-strip depoll_v*/target/wasm32-unknown-unknown/release/dePoll_v*.wasm 2>/dev/null | true

test: build-contract
	mkdir -p tests/wasm
	cp dePoll_v*/target/wasm32-unknown-unknown/release/dePoll_v*.wasm tests/wasm
	cd tests && cargo test

clippy:
	cd dePoll_v* && cargo clippy --all-targets -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd dePoll_v* && cargo fmt -- --check
	cd tests && cargo fmt -- --check

lint: clippy
	cd dePoll_v* && cargo fmt
	cd tests && cargo fmt

clean:
	cd dePoll_v* && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm
