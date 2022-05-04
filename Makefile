build: mac windows

mac:
	cargo build --release;

windows:
	cargo build --release --target x86_64-pc-windows-gnu