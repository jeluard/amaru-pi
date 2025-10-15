run-simulator:
# If on OSX, do `export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"`
	cargo run --no-default-features --features simulator

build-rpi:
	cross build --release --target aarch64-unknown-linux-musl --features display_hat --no-default-features

upload-rpi: build-rpi
	scp target/aarch64-unknown-linux-musl/release/app ${SCP_TARGET}