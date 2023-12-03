clean:
	rm -rf target

build-layer-x86_64:
	cargo lambda build --release --x86-64

build-layer-arm:
	cargo lambda build --release --arm64

format:
	cargo fmt --all

build: build-layer-x86_64 build-layer-arm
