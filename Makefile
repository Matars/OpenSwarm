.PHONY: dev docs docs-venv docs-serve docs-build

VENV_DIR := .venv
VENV_PY := $(VENV_DIR)/bin/python
VENV_PIP := $(VENV_PY) -m pip
MKDOCS := $(VENV_PY) -m mkdocs
DOCS_STAMP := $(VENV_DIR)/.docs-deps.stamp

dev:
	cargo build --release --bin openswarm
	cargo install --path . --bin openswarm --force

$(DOCS_STAMP): requirements-docs.txt
	@if [ ! -x "$(VENV_PY)" ]; then python3 -m venv "$(VENV_DIR)"; fi
	@$(VENV_PIP) install --upgrade pip
	@$(VENV_PIP) install -r requirements-docs.txt
	@touch "$(DOCS_STAMP)"

docs-venv: $(DOCS_STAMP)

docs: docs-venv
	@$(MKDOCS) serve

docs-serve:
	@$(MAKE) docs

docs-build: docs-venv
	@$(MKDOCS) build --strict
