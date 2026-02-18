.PHONY: dev

dev:
	cargo build --release --bin openswarm
	cargo install --path . --bin openswarm --force
