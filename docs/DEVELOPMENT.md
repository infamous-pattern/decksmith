# Decksmith

**Open Control, Forged for Linux.**

Linux-native control platform, with Stream Deck + as the first hardware target.
This repository is the initial local Phase 0 foundation, not a completed application.

## Baseline status

The original **Decksmith v0.4 documentation package** is imported and verified.
The root README, PRD, architecture, repository strategy, and 11 ADRs are unchanged
from the user-supplied ZIP. The archive and SHA-256 manifest are under `baseline/`.
The approved **Maker’s Mark** reference sheet is imported separately under `brand/`;
individual production icon exports remain to be prepared.
See [baseline provenance](baseline-status.md) and [reconciliation](baseline-reconciliation.md).

## Run locally

```sh
cargo build --workspace --locked
cargo run -p decksmithctl -- devices
cargo run -p decksmithctl -- virtual info
cargo run -p decksmithctl -- virtual emulate < examples/input.jsonl
cargo run -p decksmithctl -- virtual resolve < examples/triggers.jsonl
cargo run -p decksmithd -- --virtual-once
```

`devices` enumerates only USB VID 0x0fd9 / PID 0x0084 and checks read-only
access. It sends no device commands. JSON includes any access error. Detection
is not proof of event decoding, firmware queries, display output, or reconnect.
Sandbox restrictions can prevent access even when the logged-in user has it.

`virtual emulate` consumes timestamped raw events as JSON lines and emits them
through the shared device interface. Indices are zero-based. Timestamps are
caller-provided monotonic milliseconds. This is an in-process test device, not
IPC into a running daemon. For semantic triggers, use `virtual resolve` with the explicit-clock script
format in [trigger-resolver.md](trigger-resolver.md).

## Workspace

- `decksmith-core`: geometry and raw-event domain types.
- `decksmith-device`: DeckDevice interface, VirtualDeck, Linux discovery and read-only probe; physical transport pending.
- `apps/decksmith-cli` (`decksmithctl`): JSON discovery, geometry, and event emulation.
- `apps/decksmith-daemon` (`decksmithd`): one-shot virtual harness; persistent runtime pending.

GTK4/libadwaita Studio, SQLite, D-Bus, PipeWire integration, hot-plug handling,
trigger resolution, login startup, and GNOME companion remain future implementation.
No background service has been installed or enabled.

## Check

```sh
./scripts/check.sh
```

`just check` delegates to the same gate if just is installed. Requires cargo-deny
0.20.2 on PATH, or DECKSMITH_CARGO_DENY pointing at its executable. The gate runs
formatting, Clippy, unit/integration tests and dependency policy. See [CI setup](ci.md).
Rust 1.97.1 was used locally. On Codex's restricted filesystem, the Cargo advisory
cache requires host execution; ordinary terminal use has no such task restriction.

Apache-2.0 was approved by the project owner on 2026-09-10; see
[ADR-0012](adr/0012-project-license.md) and the root LICENSE. Packages remain
marked `publish = false` during private incubation.

See [Phase 0 checkpoint](phase-0-checkpoint.md) for evidence and next steps.

## Optional hardware adapter

See [hardware-adapter.md](hardware-adapter.md) for the native build prerequisites,
prepared live commands and remaining physical validation. The full quality gate now
builds all features; this workstation uses ignored local/hardware-env.sh for headers.
