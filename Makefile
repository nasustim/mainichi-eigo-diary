.PHONY: clean build dev test fmt fmt-check lint check

# Public URL the site is served from. Root for local builds; CI overrides this
# with the GitHub Pages project subpath (make build PUBLIC_URL=/mainichi-eigo-diary/).
PUBLIC_URL ?= /

# Remove build artifacts (Trunk dist/ and Cargo target/).
clean:
	trunk clean
	cargo clean

# Production build into dist/ (optimized wasm bundle).
build:
	trunk build --release --public-url $(PUBLIC_URL)

# Development server with hot reload, opened in the browser.
dev:
	trunk serve --open

# Run native unit tests (host-runnable, non-DOM logic).
test:
	cargo test

# Format sources.
fmt:
	cargo fmt

# Verify formatting without modifying files.
fmt-check:
	cargo fmt --check

# Lint; warnings are errors.
lint:
	cargo clippy --all-targets -- -D warnings

# Full verification suite (run before committing / in CI).
check: fmt-check lint test
