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
