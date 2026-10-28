# October 28 code review and testing

The source and test review covers the application, device and shared-action
boundaries, editor, packaging and Homebridge integration. Review changes include
resume and device-generation fencing, retrying failed state persistence, bounded
audio helper output and imports, archive limits, and editor recovery. Rendering
cleanup avoids repeated work without claiming a measured performance improvement.

Regression coverage includes a delayed pre-resume property reply, failed state
write followed by recovery, audio helper size and timeout handling, archive
limits, import validation, invalid metadata and device-context changes. Run the
complete Rust workspace tests, formatting and strict Clippy; the Studio and
installer Python suites; dependency policy; and isolated VirtualDeck checks.
Run those checks on the AI server against the exact committed source as well.
Record any failures and do not promote an unverified candidate.

Desktop acceptance requires the verified candidate, a verified full settings
backup and retained prior build, followed by paired-device readiness and user
checks of controls, editor behavior and relevant audio functions. These tests
do not prove long-term USB stability or microphone reconnection. Ubuntu and
Debian qualification remains paused; no support promotion is implied.

## Verification and desktop candidate

On AI_server, the committed source passed the complete Rust checks, Studio and
installer Python suites, dependency policy and VirtualDeck replay. Isolated
editor failure/close fixtures passed on both machines. Private PipeWire tests
passed microphone removal, repeated same-device reconnection, wrong-device
refusal and changed-server refusal without accessing physical hardware.

Gitea source `2347aadbd0da9b5e3925d23c73fee5d805e21be6` and GitHub source
`bb0f9b8c36a6d2e1c8be749592aa1909c5996852` have the same source tree. GitHub
Quality passed; the Fedora candidate workflow built and signed candidate
`1.5.2-7478100d4b04`. Independent attestation, checksum, reviewed helper-byte,
packaged self-check and virtual-launch checks passed.

The first desktop attempt hit a verification harness error: it passed the
normal `current` symlink to the newly hardened directory reader. The guarded
rollback restored the prior candidate and both services. After resolving and
verifying the selected release directory explicitly, a second attempt backed
up settings and the previous signed build, installed the candidate, and passed
machine checks for both device sessions, saved profiles, page/brightness,
companion readiness, startup state and audio settings. Physical interaction
acceptance was confirmed by the maintainer: both Decks changed pages and their
website, application and safe system controls worked; Plus audio dials, mute,
media and an existing Homebridge control also worked. Long-term USB stability
and physical microphone unplug/reconnect remain separate checks.
