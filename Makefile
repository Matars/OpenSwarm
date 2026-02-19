.PHONY: dev docs-serve docs-build

dev:
	cargo build --release --bin openswarm
	cargo install --path . --bin openswarm --force

docs-serve:
	python3 -m mkdocs serve

docs-build:
	python3 -m mkdocs build --strict
