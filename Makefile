default: build

test: build
	cargo test --all --tests

build:
	mkdir -p target/wasm32-unknown-unknown/optimized

	soroban contract build

	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/pool_admin_transfer.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/pool_admin_transfer.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean