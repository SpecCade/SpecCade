#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo test (workspace, all features)"
if [ -d "editor/dist" ]; then
  cargo test --workspace --all-features
else
  echo "==> editor/dist missing; excluding speccade-editor-app"
  cargo test --workspace --all-features --exclude speccade-editor-app
fi

echo "==> cargo clippy"
if [ -d "editor/dist" ]; then
  cargo clippy --workspace --all-features
else
  echo "==> editor/dist missing; excluding speccade-editor-app"
  cargo clippy --workspace --all-features --exclude speccade-editor-app
fi

echo "==> cargo doc"
if [ -d "editor/dist" ]; then
  cargo doc --workspace --no-deps --all-features
else
  echo "==> editor/dist missing; excluding speccade-editor-app"
  cargo doc --workspace --no-deps --all-features --exclude speccade-editor-app
fi

echo "==> coverage generate"
cargo run -p speccade-cli -- coverage generate

echo "==> feature coverage test"
cargo test -p speccade-tests --test feature_coverage -- --nocapture

echo "==> golden: build release CLI"
cargo build --release -p speccade-cli

if [ -d "golden/speccade/specs" ]; then
  echo "==> golden: validate specs"
  while IFS= read -r -d '' spec; do
    echo "Validating: $spec"
    ./target/release/speccade validate --spec "$spec"
  done < <(find golden/speccade/specs -name "*.json" -type f -print0)
else
  echo "==> golden: no specs directory found, skipping"
fi

echo "==> golden: hash verification test"
cargo test -p speccade-tests --test golden_hash_verification -- --nocapture

echo "OK"
