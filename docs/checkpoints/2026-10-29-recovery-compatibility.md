# Recovery and compatibility checkpoint — October 29, 2026

Recovery work was authorized before broader Ubuntu/Debian acceptance. The
installed Fedora candidate remains 1.5.2-7478100d4b04 until a separate verified
candidate is installed; physical Wave:3 reconnection and the cause of USB timeouts
remain open. The user's desktop settings and devices were not changed in these
AI-server exercises.

## Evidence and limits

- A disposable staged store built by 1.5.2-73f516e5e660 contained simulated Plus
  and MK.2 profiles, 17 configuration/store files and a synthetic icon. Upgrade to
  1.5.2-7478100d4b04 and rollback preserved config/icon hashes; both distinct
  candidate daemons reopened the paired virtual store. An earlier full store was
  copied into place with its matching previous build after retaining the newer
  store. This synthetic exercise does not prove arbitrary older-schema, real-user
  data, interruption or physical-device compatibility.
- In an isolated Fedora VM, a deliberately unavailable executable caused a
  separate user service to fail. The manager restored a corrupted synthetic
  layout without the editor; the candidate then returned a status response on a
  private bus. The guest used a temporary disk overlay and shut down normally;
  the retained Fedora baseline remained off and its domain definition unchanged.
- Ten layout-store, nine permanent-service, 14 lifecycle/execution, 24 original
  installer and 25 revised-installer unit tests passed. A separate private
  PipeWire fixture using the signed current candidate passed ten simulated
  re-enumerations with mute-first recovery, identity mismatch rejection and
  audio-server replacement refusal. Two Rust native fixture tests were ignored
  in the separate microphone-filtered suite; physical reconnection remains open.
- An earlier invalid-layout service attempt remained active while awaiting
  discovery; its result is retained and its cause is not established. A subsequent
  fixture path error repeated that attempt, and a direct missing executable was
  refused before systemd created a service. The corrected fixture produced the
  intended isolated service failure. Active status alone is not a readiness proof.

The installer now reports layout validation failure in plain language, preserves
current data and suggests a verified compatible backup. A real staged rollback
with a corrupted sample confirmed the message and preserved both the sample bytes
and current release selection. This does not diagnose the cause of every validator
failure. Tests and local evidence are retained under
`outputs/recovery-rehearsal-20261029/`; server evidence is under
`/home/james/.local/share/decksmith-test-host/manual-runs/recovery-rehearsal-20261029/`.
The manager's ordinary restore merges files, so full downgrade recovery still
requires a matching build and complete earlier store rather than a merge alone.
