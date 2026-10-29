#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# DECKSMITH_CARGO_DENY may name a task-local checker binary.
checker="${DECKSMITH_CARGO_DENY:-cargo-deny}"
if ! command -v "$checker" >/dev/null 2>&1; then
    echo 'Install cargo-deny 0.20.2, or set DECKSMITH_CARGO_DENY to its executable.' >&2
    exit 1
fi
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
# Lock-latency regressions must use a private bus, never the desktop session.
if ! command -v dbus-run-session >/dev/null 2>&1; then
    echo 'Install dbus-run-session to run the isolated lock-latency checks.' >&2
    exit 1
fi
dbus-run-session -- cargo test -p decksmithd --all-features --locked session_lock::tests::slow_ -- --ignored
"$checker" --all-features --locked check
