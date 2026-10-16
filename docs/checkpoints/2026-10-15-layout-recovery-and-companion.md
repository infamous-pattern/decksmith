# Layout recovery and Homebridge dependency refresh — October 15, 2026

These changes are installed local development builds with user-accepted paired
Homebridge/native smoke checks. They are not publicly released and do not close
the remaining V1.5 lifecycle, accessibility and resource qualification gates.

## Previous Layout recovery

Each newly committed device profile retains one checksum-verified preceding
layout in its generation. Layout changes replace that predecessor; selection,
brightness, saved-page and partner-device changes preserve it. Restoring a prior
layout is a normal device-addressed save and can itself be undone. A new device
with no previous layout has an explicit known-empty history state.

Normal recovery reads the current verified generation rather than searching a
growing directory of backups. The previous file receives the same private-file,
checksum, size, model and layout validation as the current document. Archive,
interrupted-prune resume and explicit generation restore include that file.
No input/device rendering path performs a history search.

Old generations remain readable. Their first new save imports verified preceding
history before publishing anything. Compatibility lookup inspects bounded small
manifests and fully verifies the selected archived generation. It checks the
opaque ID and private physical association, so a moved backup can recover without
depending on its original absolute path. The one-time compatibility review budget
is 4,096 sibling entries and 8 MiB of metadata per device; an oversized or corrupt
old store still requires explicit review. Newly checkpointed stores do not have
the former 64-archive recovery limit. No old backup is deleted or rewritten to
perform this import.

The profile-store manifest is now version 2; layout documents and the retained
original V1 Plus configuration are unchanged. The pre-fix multi-device development
binary cannot read this new manifest. Reverting that binary requires restoring
its matching pre-update configuration backup, retaining a separate copy of the
new store first. A binary-only downgrade is not an accepted rollback. The legacy
V1 runtime still uses the retained legacy layout and does not merge new per-device
edits automatically. Production installation must protect the original store and
pair binary/configuration recovery before activation.

Verification:

- Full hardware-enabled workspace: **336 Rust tests passed**, five intentional
  ignores; formatting and strict Clippy passed. Five new storage regressions cover
  long history, old-format/moved history import without backup rewrites, independent
  restore/undo, corrupt predecessor rejection and interrupted publication/restore.
- A private-bus test used a copy of the exact **84-archive** store that previously
  returned `profile_archive_review_required`. Recovery, checkpoint migration,
  restore, undo, partner isolation and restart passed. Original evidence and
  existing copied archives remained byte-for-byte unchanged.
- Its first runner attempted another save before the asynchronously updated
  device was ready. The corrected runner waits for input/display readiness between
  changes; the rejected attempt and the passing run are retained. No action was
  automatically replayed against a production device.

## Homebridge companion

The reviewed OpenHomeB adaptation was copied into a new build directory. Only its
`Cargo.lock` changed: `rustls` is **0.23.46** and optional `quinn-proto` is
**0.11.18**, with required transitive resolution retained in the receipt. The
adapter contract and all Rust behavior source remain unchanged. The original
adaptation, original GitHub-associated source and installed binary were preserved.

The build remaps private paths and strips executable symbols. Its receipt binds
all copied source files, updated lock, binary and adapter contract. Candidate
companion identity: `37adab0cf9b2886f`; executable SHA-256:
`80ca8b933ceb5d8ac407ec5f4ee903f4af549390846c23f011f174ed36e6cddd`.
All 75 packaged runtime manifest files verified.

Verification:

- Companion formatting, strict Clippy, Rust tests and optimized build passed.
- **46** current adapter/mock integration and protocol tests passed: brightness,
  clamping/reversal, authentication/re-authentication, credential redaction,
  timeout/lost-acknowledgement behavior, cancellation and uncertain-write fencing.
- The initial broader run mixed pre-contract Host tests and an outdated FakeHost
  with current adapters. Two representative scaffold errors reproduced against
  the accepted old binary. The corrected run uses current helper fixtures and
  adapter-aware tests; original failed evidence remains available.
- Exact-version OSV queries for **179** Rust/Python dependency versions returned
  no advisory matches. Private builder path scans passed. This is not a claim of
  complete exploit testing, physical accessory acceptance or a full TLS audit.
  The existing exploit paths were not demonstrated; updating the affected versions
  removes those matches from the reviewed dependency graph.

The build is repeatable with `scripts/build-openhomeb-companion.py --source
/path/to/approved-adaptation --output /path/to/new-build`. It refuses an existing
output directory, copies/verifies input source, records dependency updates and
checks, and never installs or changes services. An updated runtime must still be
packaged from its verified receipt and pass a live Homebridge acceptance check
before promotion. No credential, server-address or saved-assignment changes are
part of this update.

Sources: [RustSec rustls advisory](https://rustsec.org/advisories/RUSTSEC-2026-0346.html),
[RustSec quinn-proto advisory](https://rustsec.org/advisories/RUSTSEC-2026-0338.html).
Local evidence: `outputs/v15-recovery-20261015`, including final quality logs,
private API test reports, companion build/source receipts, retained fixture errors
and the passing integration/advisory result. Desktop services, real audio,
Homebridge accessories, guest VMs and scheduled jobs were not changed.

## Guarded desktop activation and physical acceptance

After the user saved and closed the editor, app
`1.0.0-dev-recovery-926e9425dc5e` and companion `37adab0cf9b2886f` were installed.
The whole pre-update device store was copied and hash-verified while both services
were stopped. The guarded rollback routine preserves newer generations separately
and restores the complete old store; it does not merge old and new generations.
Five isolated rollback checks covered exact restoration, newer-data preservation,
corrupt/link refusal, stopped-service requirements and interrupted swaps. No
production rollback was needed during this activation. A manual downgrade to the
old multi-device build must likewise replace the complete device-store directory
from its matching backup rather than use a merge restore.

Machine checks passed for both active service processes, fresh/ready companion,
selected device, both saved layouts/pages/brightness, two error-free displays and
unchanged enabled startup preference. Both previous releases and configuration
backups remain available. The installed runtime creates recovery checkpoints on
the next normal profile generation; no artificial production layout edit was made
just to force migration.

The user then confirmed existing Homebridge toggles and ordinary-speed brightness,
Plus native audio and MK.2 page controls all worked, with both Decks remaining
available. A follow-up read confirmed both connected displays without errors and
both background services running. This is user-reported functional acceptance,
not a new latency measurement or physical suspend test.

The exact packaged development archive and immutable source snapshot are retained
with the evidence; this post-install documentation update does not change their
compiled runtime. Detailed receipts: `desktop-install/result.json` and
`physical-acceptance.json` under the local recovery evidence directory.

## Updated-build physical suspend/resume

The user suspended Fedora from the system menu, resumed and unlocked it, then
confirmed both Decks recovered perfectly and all controls worked as expected.
A follow-up read on `1.0.0-dev-recovery-926e9425dc5e` confirmed both saved-device
sessions connected with ready input/displays and no device errors, and both
background services active. No service restart or automated suspend was performed
by the assistant. This supersedes the earlier physical suspend deferral. Exact
sleep duration and display latency were not measured. Evidence:
`physical-suspend-acceptance.json` in the recovery qualification directory.

Next physical gates: permanent-build USB isolation/reconnection in both directions,
quit/stop blanking and startup at login; then editor/accessibility and resources.

## Updated-build USB isolation/reconnection

Both directions passed on the permanent daily service, without an assistant
service restart. With MK.2 disconnected, the user confirmed Plus page/dial
operation; the API confirmed only MK.2 was unavailable. MK.2 reconnected with its
same opaque identity and an advanced connection counter, and both devices were
ready without errors while the Plus connection stayed unchanged.

The reverse check likewise confirmed MK.2 page/key operation during Plus removal.
On reconnection the user accepted all Plus keys, the full strip, dial and mute;
the API confirmed Plus identity retained and connection advanced, MK.2 connection
unchanged, both sessions input/display-ready without errors and both services
active. Offline transport errors were confined to the deliberately removed device
and cleared after reconnect. Visual page/brightness and action correctness are
user-accepted results, not synchronized display-latency measurements. Evidence:
`physical-hotplug-summary.json` and its four phase reports.

Remaining lifecycle checks are quit/stop blanking and start at login.

## Updated-build quit/blanking and relaunch

The user chose Quit Decksmith and confirmed both Decks completely blank, including
the entire Plus strip. Read-only checks while it remained closed confirmed both
services inactive with zero main PIDs, successful zero exit statuses and no owned
control/workspace/editor bus names. The assistant did not stop or start services.

The user then relaunched normally and confirmed saved pages, brightness and both
Decks' controls returned correctly. Follow-up status confirmed the tested build,
both device identities and brightness retained, both sessions input/display-ready
without errors and both services running. Start at login remains enabled; enabled
configuration alone is not acceptance of a new login session. Evidence:
`physical-quit-acceptance.json` and `physical-relaunch-acceptance.json`.

The remaining lifecycle gate is user-operated sign-out/sign-in acceptance,
including verifying controls start automatically without opening the editor.

## Updated-build start at login

The user saved work, signed out/in as requested and confirmed both Decks started
automatically with working pages/controls while the editor remained closed.
Follow-up status confirmed both persistent device identities, ready input/displays
without errors, both background services active, login startup enabled and the
editor bus name unowned. The assistant did not start services or open the editor.
The session transition is user reported; no pre-login session ID was captured.
Evidence: `physical-login-acceptance.json`.

This completes the updated-build suspend, USB isolation/reconnection, quit blanking,
relaunch and login sequence. The dedicated physical lock acceptance belongs to
the prior permanent build; the recovery/dependency update did not change lock
code, and no additional dedicated lock test is claimed here. Native editor/physical
assignment/accessibility, controlled resources and final release qualification
remain open.

## Native editor keyboard and screen reader

The user confirmed both device views work with stable key/dial sizing, visible
Tab/Shift+Tab focus, clear scrolling, reachable Save and Apply and no clipped
controls. The user also confirmed Orca clearly speaks device names, control names
and current values in both views, then was asked to turn it off. These are
user-reported native accessibility results; no independent speech recording or
accessibility-tree capture is claimed. Evidence:
`physical-editor-accessibility.json`. A separate label-only save/restore check is
being performed against private baselines before controlled resource sampling.
