# Coordinator synchronization optimization — October 15, 2026

Development package `1.0.0-dev-coordinator-42ddae185374` coalesces unchanged authority synchronization in the permanent paired runtime. Previously each tick rebuilt its authority map three times and woke action providers even when their scopes had not changed. The new path compares current device contexts, connection/binding generations, lease epochs and models, then rebuilds/notifies only for an actual authority change. Every decision still reads live device leases; a failing/replaced owner cannot remain authorized through a cached entry.

Input polling and the coordinator's five-millisecond loop are unchanged. Explicit page, profile, foreground, lock, sleep and disconnect revocation paths still cancel/fence immediately. Microphone hold pruning and maintenance retain their existing per-sync cadence, including on the unchanged-view path. Submission still wakes its own action provider. Hardware discovery retains the separately qualified topology cache. No dependency, file format, layout assignment, audio routing or microphone policy changed.

## Verification and installation

The hardware-enabled workspace passed 342 tests with zero failures and six intentionally ignored diagnostics. Strict workspace/all-target Clippy and the optimized bundle build passed. A new two-model test covers action submission after idle synchronization, immediate lease fencing, page/lock/foreground fences, partner isolation and replacement connection rejection. The full existing microphone, lifecycle, editor and renderer regressions passed.

The new test initially omitted the owner redraw step after directly invoking a low-level manager fence and timed out waiting for that virtual device. The original pre-optimization synchronization failed with the same fixture. Correcting the fixture to complete the normal owner transition resolved it; the failed compile/fixture diagnostic and original-baseline reproduction remain retained rather than presented as clean first attempts. No desktop update was made until the corrected suite passed.

An isolated debug-build synthetic diagnostic over three rounds of 10,000 turns reduced coordinator wall time from approximately 187–197 ms to 87–91 ms. It uses private virtual adapters with no native effects and is not a production CPU or latency result.

The exact package then passed AI-server source/package/font provenance, executable hardening, relocated self-check/VirtualDeck/CLI and staged paired backup/restore after 24 subsequent saves. No server services, VMs, schedules or native audio changed. The user saved and closed the editor before a guarded desktop installation. Both Decks became ready with exact original saved layouts, selected pages, brightness and startup enablement preserved; the previous discovery package and complete device/configuration backups remain available. The Homebridge companion stays `37adab0cf9b2886f`.

Physical controls, Homebridge/native audio/media, independent pages and both-direction USB recovery were user-accepted. The read-only half-second observer captured both disconnect/reconnect sequences and confirmed the partner kept its connection. Final readiness verified both original layouts and stable service PIDs/restart counters. The repeated clean-idle observation completed below. The reference discovery-build clean idle was 6.181% combined CPU and 3.953% daemon CPU of one core, with 42.61 MiB daemon RSS.

Private evidence is under `outputs/v15-coordinator-20261015` in the enclosing workspace. This is a local development qualification, not a signed/public V1.5 release. Final-build resource/reliability review, source/version synchronization, signed candidate/download verification and explicit publication approval remain separate.

## Completed clean-idle observation

After the user paused playback and closed the editor, the package completed 15 seconds of settling followed by 300.001 seconds and 61 five-second samples. Both Decks remained on saved Home pages at MK.2 50% and Plus 55% brightness, matching the discovery-build reference. No playback streams were observed; the only captures belonged to Decksmith peak levels. Both sessions stayed connected, input/display ready and error-free, with unchanged process identities, zero restarts and exact original layout hashes.

CPU percentages sum all threads, with one logical core equal to 100%. Service cgroup costs include helpers; cgroup memory is distinct from daemon RSS. The observer ran outside the measured cgroups, with the same cadence and read-only API/audio queries as the reference; its requests create some included service work. A separate pair of thread-counter reads overlapped the observation without adding runtime instrumentation.

| Measurement | Discovery build | Coordinator build |
| --- | --- | --- |
| Main daemon mean CPU | 3.953% | 0.897% |
| Core service/helpers mean CPU | 5.551% | 1.952% |
| Companion service mean CPU | 0.630% | 0.545% |
| Combined service mean CPU | 6.181% | 2.497% |
| Daemon RSS mean | 42.61 MiB | 45.38 MiB |
| Core cgroup mean memory | 83.98 MiB | 83.41 MiB |
| Daemon descriptors, first/last | 24 / 24 | 24 / 24 |
| Daemon threads, first/last | 31 / 31 | 30 / 30 |

The observed combined reduction is 59.60% from the preceding discovery build. The combined 5% target was met in this finite quiet sample. This sequential comparison matches the desktop, device count, saved layouts, sampled pages/brightness, quiet policy and observer cadence; service age, preceding interaction history and USB epochs differ. It is not repeated randomized A/B qualification, a guarantee for other workloads, or matched historical V1 equivalence. Stable sampled RSS/handles do not exclude long-term leaks; point sampling cannot exclude short events between points. Lower memory after restart must not be treated as a proven permanent saving.

The overlapping thread-counter observation attributed about 0.214% of one core to the coordinator and 0.154% to hardware discovery. The preceding discovery build's separate clean-window coordinator attribution was approximately 2.500%; these overlapping windows differ in duration and service lifetime, so retain them as supporting attribution rather than a repeated matched thread benchmark. The daemon's below-1% value in this one quiet sample does not establish the normally-below-1% milestone across layouts and sessions.

The authorized temporary speech-service/socket pause was restored to its original active and disabled/static enablement states. Post-test verification confirmed those states and unchanged disabled Screen Reader/no Orca. The measurement made no volume, routing, saved-layout, accessibility-setting or Decksmith service configuration change.

The physical acceptance covered both Decks' independent pages, audio/media controls, ordinary/quicker Plus dials, mute/unmute and existing Homebridge controls. The half-second observer recorded MK.2 and Plus disconnect/reconnect sequences, each with the connected partner retaining its connection and continuing to respond. Final readiness and layout hashes matched the original saved profiles. No numeric USB recovery time, visible response latency or new physical microphone-hold case is claimed for this optimization; the unchanged microphone maintenance and fencing paths retain their automated coverage. The subsequent ten-minute loaded observation is recorded below.

Detailed private evidence includes `resources/quiet-desktop-01`, its sequential comparison, speech restoration/post-test receipt, thread attribution and `recovery-watch.json`. The optimized development build remains installed; no commit, push, signed candidate or public release was performed. Final version/source synchronization, final candidate resource/reliability review, signed artifact/download verification and explicit publication approval remain.

## Ten-minute loaded physical qualification

The unchanged installed coordinator package completed 600.002 seconds and 121 five-second points with Brave uncorked at every point. A speech-dispatcher dummy stream was also uncorked throughout; Chromium appeared corked at one point only. All observed captures belonged to Decksmith peak levels. This was ordinary playback on the user's saved layouts, not a fixed four-independent-active-meter experiment. No speech services, volume, routing, assignments or Decksmith services were modified by this read-only observer; user-driven controls were retained normally.

The user accepted the specific mid-run physical page/dial/key check, then confirmed after completion: “Tested both Decks during the sample; everything worked correctly.” Coverage includes both physical Decks, independent pages, audio/media keys, ordinary/quicker Plus dial turns and correct controls/labels/icons/meters without reported freezes, blanking, missed input or wrong-target effects. Sampled MK.2 pages were 0/1 and Plus pages 0/3; no transient readiness rechecks were needed. This is physical acceptance plus point observations, not an action-by-action HID trace or numeric visible latency result.

| Loaded measurement | Result |
| --- | --- |
| Main daemon mean CPU | 2.007% of one core |
| Core service/helpers mean CPU | 3.863% of one core |
| Companion service mean CPU | 0.644% of one core |
| Combined service mean CPU | 4.507% of one core |
| Daemon RSS | 45.63 MiB throughout |
| Core cgroup mean memory | 99.09 MiB |
| Companion main RSS | 39.95 MiB throughout |
| Companion cgroup mean memory | 49.79 MiB |
| Daemon descriptors, first/last | 26 / 26 |
| Daemon threads, first/last | 31 / 31 |
| Automatic service restart delta | 0 for both units |

Both device sessions and connection identities remained stable, with ready input/displays and no errors at every sampled point; both original saved layout hashes matched at completion. A separate final check verified both Decks ready and unchanged service PIDs/restart counters. Stable RSS/handles over ten minutes does not exclude longer-term leaks. This mean includes user interaction and differs from quiet idle; it is not a controlled matched comparison with the earlier emoji-build loaded run. The companion main executable hash in the raw observation identifies its Python launcher, not the nested native binary independently verified by package admission.

Private evidence: `resources/loaded-desktop-01`, including raw samples, resource/coverage summary and separate final physical acceptance. The raw observer result retains its original `completed_pending_physical_acceptance` state; the acceptance receipt and resource/master summaries record the later user acceptance. The sampler allowed at most two seconds of readiness settling during a page/render transition, while rejecting disconnects/errors/connection changes immediately; no settling rechecks were actually needed.

This closes this installed development build's finite idle/loaded observations and physical loaded interaction check. Final version/source synchronization and signed-candidate/download qualification remain separate; no code or desktop update, commit, push or publication was performed during this loaded test.
