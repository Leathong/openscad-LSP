build: mac windows linux

mac:
	cargo build --release

windows:
	cargo build --release --target x86_64-pc-windows-gnu

linux:
	export CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc; cargo build --release --target x86_64-unknown-linux-musl

debug:
	cargo build