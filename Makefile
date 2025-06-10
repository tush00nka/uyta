SHELL := /bin/bash

release:
	cargo build --release --target=wasm32-unknown-emscripten
	cp ./target/wasm32-unknown-emscripten/release/deps/uyta.data ./docs/uyta.data
	cp ./target/wasm32-unknown-emscripten/release/uyta.wasm ./docs/uyta.wasm
	cp ./target/wasm32-unknown-emscripten/release/uyta.d ./docs/uyta.d
	cp ./target/wasm32-unknown-emscripten/release/uyta.js ./docs/uyta.js