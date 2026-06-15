# Versioning policy — pin every dependency to an exact version

**This is a hard rule. Agents must comply.** Every dependency — Rust crates, GitHub
Actions, CI tools, and JS CDN imports — is pinned to a single exact version. No ranges
(`^`, `~`, `*`), no floating major tags (`@v4`), no branch refs (`@master`, `@stable`).
**Renovate** proposes upgrades; do not loosen a pin to "fix" a build.

## How to pin each kind
- **Rust toolchain** — exact `channel` in `rust-toolchain.toml` (plus the
  `wasm32-unknown-unknown` target).
- **Rust crates** — exact `=x.y.z` requirement in `Cargo.toml` (e.g. `yew = "=0.23.0"`);
  `Cargo.lock` is committed.
  - Pin `wasm-bindgen` / `web-sys` / `js-sys` to versions compatible with what Yew already
    resolves so Trunk's auto-fetched `wasm-bindgen-cli` matches (check `Cargo.lock`).
- **GitHub Actions** — exact release tag `@vX.Y.Z` (e.g. `actions/checkout@v6.0.3`). If an
  action publishes no semver release (e.g. `dtolnay/rust-toolchain`, which only tags `v1`),
  pin to the full commit **SHA** with a trailing `# <tag>` comment instead.
- **Tools installed in CI** — exact version (e.g. `taiki-e/install-action` with
  `tool: trunk@0.21.14`).
- **JS / CDN imports** — exact version in the URL (e.g.
  `https://esm.run/@mlc-ai/web-llm@0.2.84`).

When adding any dependency, look up its current exact version (`gh api`, crates.io,
npm registry) and pin it.
