# V1 action safety and physical push-to-talk — October 7, 2026

**Scoped checks complete; publication and release acceptance remain separate.**
The tested core is the frozen signed bundle `1.0.0-c5e4fea53afe`, public source
`68c42421ceaa65bb033993b56f0babdde2a14a6e`. No runtime source, signed download or
normal saved assignment was changed by these checks.

## Exact-package VM checks

The Fedora Workstation 44 guest admitted all 389 candidate manifest files and
imported the packaged modules. Fifteen focused tests passed: six system-control,
four media-selection and five push-to-talk guardian tests. These cover explicit
power confirmation, inhibitors, nonforced authorized power methods, unavailable
or invalid commands, media-player ownership, exact media methods and microphone
release/error safety. Unit tests use isolated settings and fixtures; they do not
execute host power operations or establish physical action behavior.

Five additional live guardian cases passed using an owned synthetic microphone
in the guest: no heartbeat never opens it; input EOF, heartbeat expiry and
termination mute it; a generic default-device target is rejected. Existing guest
sources, volume/mute and default routing were preserved. The owned audio fixture
and staged files were removed, child processes reaped and the guest shut down.
All four test VMs were confirmed off afterward.

## Physical acceptance

With the user's approval, a temporary **V1 Mic Check** page bound Key 1 and the
leftmost dial press to the same named microphone. Other controls on that page
were inactive. The original microphone state was muted; an automatic timeout
restored the original layout, page and muted state. No audio was recorded.

The user confirmed all three physical cases:

1. Hold/release Key 1: green Live while held, red Muted after release.
2. Hold/release the leftmost dial: the same behavior.
3. Hold both, release the key first: Live remains until the last hold is released,
   then returns to Muted.

The first sampler captured only the initial muted state. It supports restoration
but does **not** independently corroborate those three transitions. A separate
sustained physical key hold was therefore checked with direct Fedora mute
readback as well as the packaged reader. The user confirmed **Live then Muted**;
the observer recorded Muted → Live → Muted, with about 7.44 seconds between its
live and muted observations. No disagreement between readers was recorded.
Polling at 200 ms does not measure input or visible-response latency.

Both temporary runs restored the original saved configuration and artwork bytes,
active page and muted microphone state, with the daemon PID unchanged. A final
independent check confirmed the signed installed/running build, connected and
display-ready device, original saved state/default routing, enabled login
preference and muted microphone. Physical overlap acceptance is human evidence;
it was not independently instrumented in this pass. Touch-strip tap behavior was
not part of these temporary hold tests.

## Coverage reconciliation

| Action family | Evidence and scope |
| --- | --- |
| Page/navigation, audio/mute, media, brightness and normal feedback | Existing exact-candidate physical interaction, resource and lifecycle checkpoints; sampled dispatch timing covers audio dials only |
| Application launch | Exact-candidate physical application keys accepted, including the recorded loaded sample |
| Website launch | Earlier reference-user acceptance carried forward; the `launch.py` adapter is byte-identical between canonical `481a622a63144f32fe61166b5685d3f576e6a6d0` and the candidate; no new physical website trial in this pass |
| System controls | Earlier guest/host results in [system actions](../system-actions.md), plus current packaged-module tests above; `system_controls.py` is unchanged over that same source interval |
| Power confirmation | Earlier guest confirmation/cancel and confirmed reboot/shutdown; current tests require confirmation, inhibitors and nonforced authorization. The confirmation UI changed only by gettext wrapping over the compared interval; no new host shutdown/reboot execution requested |
| Push-to-talk | Current physical key/dial/overlap acceptance and sustained mute-readback confirmation above; current guardian unit/live failure checks |
| Missing devices, absent capabilities and media isolation | Current package tests and retained exact-candidate helper-failure/health checks; absence is not silently redirected to another target |
| Experimental Homebridge | Separate [fault/recovery and live coexistence checkpoint](2026-10-07-v1-homebridge-faults.md); not covered by the core archive's attestation |

This ledger combines exact-candidate checks and explicitly identified historical
acceptance; it is not an exhaustive new physical execution of every assignment,
touch behavior, power profile, hardware combination or failure mode. The maintainer
must reconcile that coverage and the other documented limits in the
[release review](../v1-release-review.md) before publication. Private raw layouts,
device identifiers, sampled mute states and logs remain in ignored local storage.
