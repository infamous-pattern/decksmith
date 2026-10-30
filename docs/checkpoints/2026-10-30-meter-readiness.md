# Plus meter and display readiness — October 30, 2026

## Observation and diagnosis

During the ordinary-use soak, the installed Fedora candidate
`1.5.2-13e2b24be597` repeatedly reported the Plus display as pending. Two
attempts stopped early under the original readiness rules; in the separate
ten-minute observation, three of 44 rechecks did not observe a settled
display within two seconds. The maintainer confirmed normal physical display,
key, dial and audio/media operation during the observations. Both devices
remained connected, neither service restarted and no matching USB error was
observed. These samples do not prove uninterrupted pending work or identify
which exact internal stage caused each physical observation.

An isolated coordinator regression reproduced one concrete mechanism: a
meter-only invalidation marked a settled page unready before a hardware write.
The device actor also used any unsent touch frame to gate new key and dial
presses. Continuous replacement of live meter frames could therefore make
readiness fluctuate and potentially suppress an input, even when the displayed
page and control assignments were unchanged. The measured behavior is not by
itself proof of missed physical input or an identified USB fault.

## Change and safeguards

Meter-only touch updates are now distinct from page, key, brightness and
guarded touch-state changes. A replaceable meter frame on an already painted
page does not by itself close the page/input readiness gate. A complete page
still waits for its required surfaces, and a guarded touch-state change still
waits for its own completion. A frame revision prevents a write selected before
a replacement page from satisfying that page's barrier. Meter renders cannot
supersede in-flight full-page or guarded state renders. Connection, lock,
input-drain, ownership, failed-write and microphone hold fences remain in
place. Readiness now means the page and guarded state have been transmitted;
it does not promise that every superseded meter sample was transmitted.

## Verification and remaining work

- An isolated coordinator test failed before the change and passes afterward.
- Device-actor tests cover a stalled meter write, queued key delivery during
  that write, guarded touch updates, a superseding full page, a meter replacing
  the new page's first strip, stale writes, input drains, transport failures and
  partner isolation. The coordinator test covers meter bursts during a full
  page render.
- The local workspace all-feature suite passed sequentially; the two private
  D-Bus slow-lock checks and Clippy with warnings denied passed. A parallel
  workspace run had a sleep-inhibitor timing test fail with `WouldBlock`; that
  test passed alone and in the sequential suite. The cause was not established.
- The actor and focused meter suites passed in an isolated AI-server source
  copy, without starting a VM or installing the build. Its newer Rust toolchain
  emitted existing `fetch_update` deprecation warnings.
- A non-installing local Fedora build completed its built-in daemon self-check,
  virtual-device smoke check and CLI check. All 509 manifest files matched.
  The resulting `1.5.2-e25f4c8289b3` artifact is unsigned and records a dirty
  source tree; it is only build evidence, not a candidate for desktop installation.

No signed candidate has been built or installed for this change. Physical
readiness, key and dial behavior under active meters, page/lock/reconnect safety,
resource use and a new ordinary-use soak remain to be checked on a verified
candidate after desktop installation is separately approved. The previous
soak's 5.86% combined CPU observation is not a matched performance comparison.
