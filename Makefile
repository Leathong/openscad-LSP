build: mac windows linux

format:
	cargo fmt && cargo generate-lockfile

mac: format
	cargo build --release --target=x86_64-apple-darwin && \
	cargo build --release --target=aarch64-apple-darwin && \
	lipo -create -output target/release/openscad-lsp target/x86_64-apple-darwin/release/openscad-lsp target/aarch64-apple-darwin/release/openscad-lsp && \
	zip -j target/openscad-lsp-mac.zip target/release/openscad-lsp

windows: format
	cargo build --release --target x86_64-pc-windows-gnu && \
    zip -j target/openscad-lsp-win.zip target/x86_64-pc-windows-gnu/release/openscad-lsp.exe

linux: format
	export CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc; cargo build --release --target x86_64-unknown-linux-musl && \
    zip -j target/openscad-lsp-linux.zip target/x86_64-unknown-linux-musl/release/openscad-lsp

debug: format
	cargo build