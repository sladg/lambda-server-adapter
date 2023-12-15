TARGET=$(shell pwd)/dist

clean:
	rm -rf target
	rm -rf $(TARGET)
	mkdir -p $(TARGET)

build-layer-x86_64:
	cargo lambda build --release --x86-64
	cd target/lambda && zip -r $(TARGET)/lambda-adapter-x86_64.zip lambda-adapter

build-layer-arm64:
	cargo lambda build --release --arm64
	cd target/lambda && zip -r $(TARGET)/lambda-adapter-arm64.zip lambda-adapter

format:
	cargo fmt --all

build: clean build-layer-x86_64 build-layer-arm64
