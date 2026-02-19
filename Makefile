.PHONY: dev docs docs-deps docs-venv docs-serve docs-build

DOCS_BUNDLE_STAMP := .bundle/.docs-deps.stamp

dev:
	cargo build --release --bin openswarm
	cargo install --path . --bin openswarm --force

$(DOCS_BUNDLE_STAMP): Gemfile
	@mkdir -p .bundle
	@bundle install --path .bundle/gems
	@touch "$(DOCS_BUNDLE_STAMP)"

docs-deps: $(DOCS_BUNDLE_STAMP)

docs-venv: docs-deps

docs: docs-deps
	@bundle exec jekyll serve --source docs --destination site --livereload

docs-serve:
	@$(MAKE) docs

docs-build: docs-deps
	@bundle exec jekyll build --source docs --destination site
