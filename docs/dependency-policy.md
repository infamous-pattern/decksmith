# Dependency and diagnostics policy

Commit Cargo.lock and use --locked for checks. scripts/check.sh runs cargo-deny
0.20.2 against deny.toml, including refreshed RustSec advisories, license allowlist,
duplicate version bans, wildcard bans, and crates.io-only dependency sources.
All four checks passed on 2026-09-10. This is a point-in-time check, not a guarantee
against future advisories. No ignored advisories or blanket license exceptions exist.

Tracing and tracing-subscriber are initialized only at executable boundaries.
JSON diagnostics go to stderr with operation and error_code fields. CLI stdout
remains JSON result data. The one-shot daemon emits startup/shutdown logs; future
systemd deployment can capture stderr in the user journal. No service is installed.

Device and trigger APIs use typed errors. CLI converts failures to stable diagnostic
codes, without echoing raw input or command arguments. Error messages need not include
sensitive context. Domain libraries do not install global subscribers.

Review new native dependencies against v0.4 and keep OS/library types behind adapters.
Current license allowlist: Apache-2.0, MIT, Unicode-3.0. Dependencies retain their
own licenses; new exceptions require a documented scoped decision.

Reference: https://embarkstudios.github.io/cargo-deny/
