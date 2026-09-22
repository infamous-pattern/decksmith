# v0.4 reconciliation — 2026-09-10

## Corrected in this checkpoint

- Imported all 15 supplied documents unchanged and preserved their source archive/hashes.
- Replaced the inferred root README with the authoritative README; moved practical
  scaffold instructions to DEVELOPMENT.md so planned features do not imply working code.
- Moved binaries to apps/decksmith-cli and apps/decksmith-daemon, preserving the
  decksmithctl and decksmithd executable names.
- Renamed decksmith-hid to decksmith-device. Moved the device contract and VirtualDeck
  out of core into that crate; core now holds only shared domain types.
- Named the project-owned interface DeckDevice per ADR-0004. Its methods remain an
  initial subset; typed descriptors and region writes are pending; poll_event now returns errors.
- Corrected milestone tracking: full hardware developer preview belongs to Phase 1.

## Remaining differences and Phase 0 gaps

- Press Trigger Resolver skeleton and deterministic timing-policy tests are implemented;
  explicit-clock semantic CLI replay is implemented. Persistent daemon scheduling
  and binding-context routing remain pending.
- Structured tracing, typed device/trigger errors, coding standards and ADR workflow
  are in place. Production daemon/journal integration remains pending.
- The shared check script now passes formatting, linting, tests and cargo-deny.
  Gitea workflow is prepared as manual-only; remote execution awaits an isolated runner.
- Renderer/store/Linux/IPC/plugin-host/testkit and Studio boundaries are documented
  but not yet scaffolded as crates. No empty implementation claims are made for them.
- VirtualDeck currently supports raw input, RGB buffers and brightness. It does not
  yet expose identity/capability descriptors, injected faults,
  production renderer/binding integration or device-manager lifecycle routing.
  Disconnect/reconnect and public frame inspection now exist; trigger integration is tested.
- The physical adapter now uses the pinned upstream HID library behind the device
  interface. Synthetic adapter tests pass, and live firmware plus partial input coverage
  are verified (see hardware-adapter.md). Short taps and live key/strip writes are now verified, with user confirmation of
  placement and orientation. All four dial pushes are also verified. Brightness and
  production daemon integration remain pending. A bounded dedicated worker with
  session-aware reconnect is implemented; see device-worker.md for evidence and limits.
- Apache-2.0 is accepted in ADR-0012. Gitea server is confirmed as
  https://github.com/infamous-pattern/decksmith and commits are pushed.
  Individual production icon exports remain pending.
  The approved brand reference sheet is now imported under brand/.

The original v0.4 documentation is authoritative. This checkpoint reconciles the
existing foundation; it does not declare Phase 0 complete or change the approved scope.
