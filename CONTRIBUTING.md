# Contributing during private incubation

Use short-lived branches and keep main buildable. Original code is Apache-2.0.
The unchanged v0.4 documents are the baseline; later accepted ADRs supersede
individual decisions. Submit a numbered ADR with Context, Decision and Consequences
before changing a boundary or product decision. Record status and date.

Keep core independent of GTK, HID, D-Bus and persistence implementation types.
Device access belongs in decksmith-device. Test independent domain behavior without
hardware. Never claim a simulator test certifies real-device behavior.

Run ./scripts/check.sh (or just check). Requires Rust stable with rustfmt and Clippy,
plus cargo-deny 0.20.2 on PATH. Cargo.lock is committed; build/check scripts use
--locked. Every dependency-policy exception needs a written reason and scope.

Use typed errors at library boundaries and propagate recoverable failures. Avoid
panic/unwrap for external input. Local invariant assertions must be justified.
CLI stdout carries results; tracing diagnostics go to stderr. Log stable operation
and error_code fields. Never log raw CLI input, secrets, clipboard contents or
credentials. The CLI accepts at most 16 KiB per JSON line.

Keep tests focused on public behavior, timing boundaries, cancellation, invalid
input, independent devices and failures. Maintain explicit timestamp ordering in
replay tests. Run physical tests only in an agreed window after OpenDeck is stopped.
