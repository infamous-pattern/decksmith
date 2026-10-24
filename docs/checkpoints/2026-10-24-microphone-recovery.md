# Microphone reconnection and shutdown — October 24, 2026

This is an authorized reliability exception during the development pause. The
maintainer requested completing these repairs and testing on the AI server.
The maintainer subsequently authorized committing/pushing the fixes and notes
to Gitea/GitHub and building/verifying a signed Fedora V1.5.2 candidate.
Ubuntu/Debian qualification remains paused. Desktop installation, public-release
publication and support promotion remain separate decisions. Ask before installing
the candidate on the desktop.

## Incident and limits

The desktop lost USB devices on their former shared hub branch. Kernel logs
recorded reset, descriptor and enumeration failures involving both Decks and the
Wave:3. The devices are now on motherboard ports. The shared unpowered hub is a
plausible contributor, but the exact electrical, cable, firmware or controller
trigger is not established. A software recovery correction does not prove or
repair that underlying USB failure.

Two software faults were reproduced independently. A retained microphone pin
continued referencing the old audio connection after USB re-enumeration. The
helper correctly refused to write to the replacement connection, but could not
recover within that daemon session. After the maintainer attempted Stop/Restart,
the microphone worker's ten-second cleanup window exhausted the entire service's
ten-second stop allowance. The resulting dump was a stop-timeout abort, not proof
of a spontaneous native memory fault. Private logs, dumps and complete verified
configuration/installed-build backups remain outside public source.

## Correction

- Pins additionally retain USB bus, device serial, vendor ID and product ID when
  all are available. These are physical matching evidence, distinct from the
  per-connection source index and `object.serial`. Raw identities stay private.
- A **mute-only** helper may replace a stale connection pin after the named
  source and all four USB identity fields match, on the same audio-server
  socket. It checks the new connection again and atomically replaces its private
  pin. Live/hold and toggle helpers cannot rediscover a stale connection.
- The coordinator retires the old writer before any replacement helper starts.
  Failed holds and uncertain toggles become a pending explicit mute, cancelling
  all leases and pending manual actions for that target. Recovery never replays
  an unmute or old toggle. Downs consumed during recovery require a fresh press.
- While a hold remains active, the helper verifies the pinned connection at most
  every 250 ms after a received heartbeat. Loss now ends the old helper even if
  both Decks continue sending heartbeats. The independent 750 ms missing-heartbeat
  watchdog and pre-write connection checks remain.
- After three initial mute attempts, a reconnectable native target gets at most
  one further mute attempt per 5.25 seconds (5-second reconnect interval plus
  250 ms failure spacing). The coordinator sleeps between exhausted attempts;
  other targets keep their own ownership. There is still one writer per target
  and an eight-target maximum. Stop never schedules these background probes.
- Exhausted mute recovery with no remaining writers exits promptly on shutdown,
  retaining its blocked/unconfirmed state and a fixed diagnostic. Closing
  guardians retain the existing ten-second cleanup limit. Forced child exit
  confirmation now polls for at most 250 ms rather than waiting indefinitely;
  an unconfirmed writer prevents replacement. The packaged service allows 30
  seconds for microphone cleanup and remaining service teardown.

A missing/incomplete USB identity, different hardware under the same name,
changed server socket, malformed pin or failed initial discovery stays blocked.
The existing pin is never discarded just to make a lookup succeed. An initial
lookup that never established identity and an audio-server restart require
separate deliberate recovery/restart after the intended microphone is present.
No default-device fallback or automatic unmute is introduced. Normal manually
committed unmute and normal overlapping holds keep their existing semantics.

## Validation and reproducibility

Run the ordinary Rust formatting, strict Clippy, workspace tests, Python tests,
VirtualDeck replay and dependency audit on an isolated AI-server source copy.
The additional native test is opt-in:

```sh
python3 -B tests/private_microphone_recovery.py /absolute/source /absolute/evidence
```

That runner creates its own `/tmp/ds-mic-*` PipeWire/Pulse sockets with synthetic
sources. It never passes through USB or accesses host microphones. It exercises
real libpulse reads/writes, source removal and re-enumeration, retained pins,
same-name/different-serial rejection, changed-server rejection, unrelated-source
mute/volume preservation and ten successive reconnects. Two opt-in Rust tests
exercise the production coordinator/native guardian with paired-owner holds,
manual toggles, lease cancellation, fresh-input recovery and shutdown while the
microphone is absent. Every owned fixture process is retired at completion.

Final AI-server results: **354 Rust workspace tests passed**, with seven ordinary
opt-in/platform tests skipped by the normal suite; both selected native
microphone integration tests were then explicitly enabled and passed. The
Python suites ran **122 script checks and 258 Studio checks**, with one documented
skip in each suite. The 20 focused Python microphone tests all passed. Formatting,
strict Clippy, VirtualDeck replay and the dependency audit passed. All 673 source
files matched the uploaded manifest before testing and remained unchanged during
the run. Subsequent changes record the results/publication authorization and
make the native test-runner paths portable; production runtime bytes are unchanged.
The native runner was repeated on the AI server after that portability edit.

The final native component joins measured approximately **2.63 ms** after normal
recovery and **0.47 ms** with the microphone unavailable after exhausted attempts.
The latter retained blocked/unconfirmed mute state; it did not fabricate success.
These are microphone-component timings, not measurements of full systemd service
teardown or physical USB recovery. No server packages were installed and no guest
was started. The normal recurring timers remained enabled and active.

The first native run exposed the missing identity check during continued hold
heartbeats; that production defect is corrected above. Earlier fixture attempts
also exposed missing default metadata, unavailable optional `pw-*` utilities,
module-argument quoting and volatile Pulse modules after a private-server
restart. The final fixture uses existing `pactl` modules; no server packages were
installed. Retain failed-attempt receipts rather than counting them as passes.

Machine results and source hashes are in the private workspace directory
`outputs/microphone-recovery-20261024/`, with server receipts beneath
`~/.local/share/decksmith-test-host/manual-runs/microphone-recovery-20261024/`.
These fixtures establish software recovery behavior, not physical Wave:3/USB
acceptance or long-term desktop stability. A signed Fedora build and separately
approved desktop installation/physical checks remain later gates. Keep Ubuntu
and Debian paused; do not infer broader distribution acceptance from host tests.


## Signed Fedora delivery and approved desktop installation

The reviewed fixes and testing notes were committed to canonical/Gitea
`67a3b4d84ae16fdd5bd8fa3c5d82a7e9263bdd60` and exported to sanitized GitHub
`a5f6e7d8442bd9c4e08deca6c32fc354821ba314`, with identical tree
`e073f6dd2c5aaa55ba0365293319b81577b7dc29`.
[Quality run 40124370873](https://github.com/infamous-pattern/decksmith/actions/runs/40124370873)
and [Fedora build/signing run 40124386651](https://github.com/infamous-pattern/decksmith/actions/runs/40124386651)
completed successfully. No other distribution candidate workflow was started.

Candidate **1.5.2-73f516e5e660** passed independent verification of all six signed
subjects, exact workflow/source identity, checksums, archive metadata and all
**503 manifest files**. Archive SHA-256:
`ae4579944e407f038e6c38df4ef939300f1d14d73f0483466360317672e5867c`.
On the AI server, the exact signed package passed runtime self-check/VirtualDeck,
20 packaged-helper tests and native private-audio removal/re-enumeration checks,
including ten successive muted recoveries, different-device/server refusal and
unrelated microphone state preservation. Its helper and unit bytes matched the
tested source, including the 30-second service stop allowance. No guest or
physical host microphone was used for those checks.

After the maintainer explicitly confirmed saved/closed Studio and approved the
installation, this candidate replaced desktop **1.5.2-f49eb0631053**. A fresh
complete **568-file** settings snapshot and archive were hash-verified before
replacement; the previous signed build was copied and byte-verified and remains
retained. Managed integrations, icon data, startup state and rollback receipts
are preserved privately. Installation left saved layouts, pages, brightness,
selection and audio devices/defaults/volume/mute unchanged, with the existing
Homebridge companion retained and ready. All 503 installed manifest hashes
matched; both Decks reported input/display ready, the effective stop allowance
was 30 seconds, and both services had zero automatic restarts.

The maintainer then confirmed: **“Both Decks and controls work normally”** after
page changes on each Deck and a Plus volume/media check. This is immediate basic
physical acceptance of the installed candidate, not physical microphone unplug/
reconnect acceptance or long-term USB stability. Do not deliberately disconnect
an active microphone or unmute it automatically to broaden that result.

Private evidence includes `delivery-result.json`, `candidates/fedora44/`,
`signed-package-verification/`, `desktop-install/`,
`desktop-runtime-verification.json` and `physical-acceptance.json` under
`outputs/microphone-recovery-20261024/`. Keep the current/previous builds and
complete backups. Public release remains V1.5.1; no release or support promotion
occurred. Ubuntu/Debian qualification and other development stay paused.
The following documentation-only acceptance record does not change the signed
candidate's source commit or runtime bytes.
