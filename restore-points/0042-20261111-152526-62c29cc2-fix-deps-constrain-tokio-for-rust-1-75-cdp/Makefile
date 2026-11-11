.PHONY: build test wasm serve clean

build:
	cargo build --all --release

test:
	cargo test --all

clippy:
	cargo clippy --all -- -D warnings

wasm:
	cd crates/bionetic-wasm && wasm-pack build --target web --out-dir ../../web/pkg

serve: wasm
	cd web && python3 -m http.server 8080

run:
	cargo run --example basic_brain --release

train:
	cargo run --example train_and_sleep --release

clean:
	cargo clean
	rm -rf web/pkg
