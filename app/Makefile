run-simulator:
# If on OSX, do `export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"`
	cargo run --no-default-features --features simulator

build-pi:
	cross build --release --target aarch64-unknown-linux-musl

upload-pi: build-pi
	@if [ -z "$${SCP_TARGET}" ]; then \
		echo "Error: SCP_TARGET is not set!"; \
		exit 1; \
	fi
	scp target/aarch64-unknown-linux-musl/release/amaru-pi ${SCP_TARGET}

build-pi-gnu:
	cross build --release --target aarch64-unknown-linux-gnu
