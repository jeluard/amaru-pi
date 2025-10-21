run-simulator:
# If on OSX, do `export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"`
	cargo run

build:
	rustup target add aarch64-unknown-linux-gnu
ifeq ($(shell uname -s),Linux)
	cargo build --no-default-features --features display_hat --release --target aarch64-unknown-linux-gnu
else ifeq ($(shell uname -s),Darwin)
	cargo zigbuild --no-default-features --features display_hat --release --target aarch64-unknown-linux-gnu
endif

upload: build
	@if [ -z "$${SCP_TARGET}" ]; then \
		echo "Error: SCP_TARGET is not set!"; \
		exit 1; \
	fi
	scp target/aarch64-unknown-linux-gnu/release/amaru-pi ${SCP_TARGET}
