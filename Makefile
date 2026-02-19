.PHONY: dev docs docs-deps docs-venv docs-serve docs-build

DOCS_BUNDLE_STAMP := .bundle/.docs-deps.stamp
BREW_RUBY33_BIN := /opt/homebrew/opt/ruby@3.3/bin
BREW_RUBY_BIN := /opt/homebrew/opt/ruby/bin

ifeq ($(wildcard $(BREW_RUBY33_BIN)/bundle),$(BREW_RUBY33_BIN)/bundle)
export PATH := $(BREW_RUBY33_BIN):$(PATH)
BUNDLE := $(BREW_RUBY33_BIN)/bundle
else ifeq ($(wildcard $(BREW_RUBY_BIN)/bundle),$(BREW_RUBY_BIN)/bundle)
export PATH := $(BREW_RUBY_BIN):$(PATH)
BUNDLE := $(BREW_RUBY_BIN)/bundle
else
BUNDLE := bundle
endif

dev:
	cargo build --release --bin openswarm
	cargo install --path . --bin openswarm --force

$(DOCS_BUNDLE_STAMP): Gemfile
	@mkdir -p .bundle
	@$(BUNDLE) config set --local path .bundle/gems
	@$(BUNDLE) install
	@touch "$(DOCS_BUNDLE_STAMP)"

docs-deps: $(DOCS_BUNDLE_STAMP)

docs-venv: docs-deps

docs: docs-deps
	@$(BUNDLE) exec jekyll serve --source docs --destination site --livereload

docs-serve:
	@$(MAKE) docs

docs-build: docs-deps
	@$(BUNDLE) exec jekyll build --source docs --destination site
