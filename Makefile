.PHONY: clean build dev

# Remove build artifacts (Trunk dist/ and Cargo target/).
clean:
	trunk clean
	cargo clean

# Production build into dist/ (optimized wasm bundle).
build:
	trunk build --release

# Development server with hot reload, opened in the browser.
dev:
	trunk serve --open
