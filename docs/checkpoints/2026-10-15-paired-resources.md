# Permanent paired desktop resources — October 15, 2026

Build `1.0.0-dev-emoji-4551ce366af5` completed a 300.002-second read-only paused-desktop observation: 61 resource points at five-second cadence, with both physical Stream Deck + and MK.2 on their saved Home pages. The editor was closed, Fedora stayed unlocked, both service identities/restart counters remained stable, and both saved layouts matched their pre-test hashes afterward. There were no service, audio, routing or layout changes by the observer.

## Audio limitation and admission

Preflight found two uncorked speech-dispatcher playback streams (`speech-dispatcher-dummy` and `speech-dispatcher-espeak-ng`), although GNOME Screen Reader was disabled and no Orca process was observed. The user explicitly chose to measure this desktop state with that limitation documented. Those two stream names persisted in the playback inventories; the only capture streams belonged to Decksmith peak levels. No PCM/audio was recorded.

**This is not an admitted clean quiet-idle measurement.** Uncorked streams do not alone establish audible sound, and the observer did not measure their signal. This sample neither proves the below-1% target nor closes clean-idle or regression qualification. It is not a matched comparison with the earlier single-device V1 results: device count, layouts, runtime, observer cadence, service lifetime and audio conditions differ.

## Measured resources

CPU percentages sum all threads, with one logical core equal to 100%. Service cgroup CPU includes helpers and exited-child work. Cgroup memory includes helper/cache accounting and is not daemon RSS.

| Measurement | Result |
| --- | --- |
| Main daemon mean CPU | 5.737% |
| Core service and helpers mean CPU | 7.436% |
| Homebridge companion service mean CPU | 0.637% |
| Combined services mean CPU | 8.073% |
| Main daemon RSS | 51.34 MiB throughout |
| Core cgroup mean memory | 108.32 MiB |
| Companion main RSS | 38.58 MiB throughout |
| Companion cgroup mean memory | 56.10 MiB |
| Main daemon open descriptors, first/last | 24 / 24 |
| Main daemon threads, first/last | 31 / 31 |
| Automatic service restarts | 0 |

Both device sessions were connected and input/display ready without errors at every observed point, with unchanged page/brightness/connection values. The main daemon RSS was below the 100 MB target in this finite sample, without observed growth; this does not exclude longer-term leaks. User instructions requested no Deck input; no added HID instrumentation proves absence of every possible input. Point observations cannot rule out short events between samples.

The observer ran outside the measured service cgroups and performed two systemd property reads, two Decksmith status reads, an editor ownership check and two audio graph reads at each point. Its API requests create some work inside Decksmith; observer overhead is not subtracted. Exact process identities, layouts, audio inventories and raw samples remain private in `outputs/v15-emoji-20261015/resources/paused-desktop-01` in the enclosing workspace.

## Remaining qualification

The ten-minute loaded observation below is complete and physically accepted. A controlled clean-idle observation is now complete below; performance regression review, final signed-artifact admission and release publication remain open. The loaded interaction result does not turn the paused sample above into clean-idle evidence.

## Ten-minute saved-layout loaded interaction

The same installed build completed 600.003 seconds with 121 five-second observations. Brave playback was uncorked at every observed point. A Chromium stream and the two speech-dispatcher streams also appeared in the playback inventories; all observed captures belonged to Decksmith peak levels. This is a mixed saved-layout workload, not a fixed four-independent-active-meter experiment. No temporary profiles, automatic volume/media commands or service changes were used.

The user confirmed: “Tested both Decks; everything worked correctly.” The mid-run physical navigation/ordinary/quicker-dial prompt was separately accepted, and sampled states included Home and Media on both devices. User acceptance covered correct controls, labels, icons and meters without reported freezes, blanking, missed input or wrong-target effects. Numerical visible timing and an action-by-action trace are not established here.

| Measurement | Loaded result |
| --- | --- |
| Main daemon mean CPU | 5.205% of one core |
| Core service/helpers mean CPU | 7.253% of one core |
| Companion service mean CPU | 0.652% of one core |
| Combined service mean CPU | 7.905% of one core |
| Main daemon RSS | 51.59 MiB throughout |
| Core cgroup mean memory | 108.65 MiB |
| Companion main RSS | 38.58 MiB throughout |
| Companion cgroup mean memory | 56.71 MiB |
| Main daemon descriptors, first/last | 24 / 24 |
| Main daemon threads, first/last | 31 / 31 |
| Service restart delta | 0 for both units |

Both sessions were connected, input/display ready and error-free at every sampled point; no display-transition rechecks were needed. Process identities and connections stayed stable, and both saved layout hashes matched the original pre-test baselines at the end. User-driven runtime page/audio changes were retained normally; there were no temporary assignments to restore. Stable RSS over ten minutes does not exclude longer-term leaks. The loaded mean being below the preceding paused-desktop mean is not evidence of a performance improvement: this is not a controlled matched comparison.

Private evidence: `outputs/v15-emoji-20261015/resources/loaded-desktop-01`, including raw observations, binary/process metadata, interval statistics, playback coverage and user acceptance. The recorded companion main executable hash identifies its Python launcher, not the nested OpenHomeB native binary. Release artifact admission must separately verify the bundled companion files.

## Clean five-minute paired idle observation

With explicit user approval, the leftover speech-dispatcher process was gracefully stopped. Firefox's existing client then reactivated it through its socket. The first strict-quiet preflight rejected that state before taking any measurement points; it is retained as a rejected preflight, not an application failure. For the admitted retry, the speech service and activation socket were temporarily stopped. Their original active and enablement states were captured and restored in finalization. A separate post-test check verified both units active, with their original disabled/static enablement values, Screen Reader still disabled and no Orca process. No accessibility settings or audio routing were changed.

After a 15-second settling period, the same installed development build completed 300.001 seconds and 61 five-second points. No playback streams appeared at any observed point; only Decksmith peak-level capture clients were observed. The editor stayed closed and Fedora unlocked. Both Decks remained input/display ready and error-free, with unchanged pages, brightness, connection identities and saved layouts. Both service PIDs remained unchanged and their restart deltas were zero.

| Measurement | Clean idle result |
| --- | --- |
| Main daemon mean CPU | 5.757% of one core |
| Core service/helpers mean CPU | 7.360% of one core |
| Companion service mean CPU | 0.645% of one core |
| Combined service mean CPU | 8.005% of one core |
| Main daemon RSS | 51.59 MiB throughout |
| Core cgroup mean memory | 108.48 MiB |
| Companion main RSS | 38.58 MiB throughout |
| Companion cgroup mean memory | 57.89 MiB |
| Main daemon descriptors, first/last | 24 / 24 |
| Main daemon threads, first/last | 31 / 31 |
| Service restart delta | 0 for both units |

This closes collection of the clean paired-idle baseline. It does not meet the deferred below-1% CPU goal, establish a matched single-device V1 comparison, or close resource regression review by itself. CPU remained near the earlier paused-desktop observation with speech streams; removing those clients did not materially reduce the observed Decksmith CPU in these finite samples, but this sequence is not a controlled causal experiment. The historical V1 reference (2.11% daemon, 3.75% full service, 17.90 MiB daemon RSS) used a different runtime/device workload, so the increased current values must remain visible in release review rather than be described as regression-free. The separate thread-attribution evidence identifies discovery/coordinator costs for follow-up without changing runtime polling during qualification.

Daemon RSS was below the 100 MB target, with no observed descriptor/thread growth. Five minutes cannot exclude long-term leaks, and point sampling cannot exclude activity between observations. Observer/API overhead remains included as described above. No runtime code, Decksmith service configuration, volume, playback, routing or saved-layout change was made for this measurement.

Private evidence: `outputs/v15-emoji-20261015/resources/quiet-desktop-02`, `speech-stop.json`, `speech-unit-pause.json` and `post-quiet-state.json`. The rejected `quiet-desktop-01` preflight and socket-client attribution are retained separately. Speech service restoration is complete; no paused speech service remains to recover.

## Subsequent discovery update

The later hardware-discovery-only development package `1.0.0-dev-discovery-d5e84a632d77` passed physical controls, bidirectional USB isolation/recovery and a new clean five-minute paired-idle observation. Combined service CPU was 6.181% of one core versus 8.005% above, an observed 22.79% reduction; the combined 5% target remains unmet. Daemon CPU was 3.953% with flat 42.61 MiB RSS. Both devices, service identities and saved layouts remained stable, and the speech-service pause was fully restored. See the [discovery optimization checkpoint](2026-10-15-hardware-discovery-optimization.md) for matching conditions, raw evidence, thread attribution and limits. The loaded result above belongs to the preceding emoji build; no new ten-minute loaded resource result is implied.

## Subsequent coordinator update

The next development package, `1.0.0-dev-coordinator-42ddae185374`, coalesced unchanged authority synchronization while retaining input polling and microphone maintenance. Both-device controls, Homebridge and observed bidirectional USB recovery passed. Its 300.001-second clean paired-idle sample averaged 2.497% combined service CPU of one core, 0.897% daemon CPU and flat 45.38 MiB daemon RSS. This is 59.60% lower combined CPU than the preceding 6.181% discovery-build reference, meeting the combined 5% goal in this finite quiet observation. Both Decks/services/layouts remained stable and speech states were restored. See the [coordinator checkpoint](2026-10-15-coordinator-optimization.md) for full evidence and sequential-comparison limits. Earlier loaded measurements still belong to their exact preceding build; new final-candidate loaded qualification remains separate.

## Optimized ten-minute loaded follow-up

The coordinator development package subsequently completed 600.002 seconds/121 points and received final physical acceptance for both Decks. Combined service CPU averaged 4.507% of one core (2.007% daemon), with flat 45.63 MiB daemon RSS, no restarts, unchanged original layouts and stable ready sessions. Brave was active throughout sampled points; an uncorked speech dummy stream and a single corked Chromium point are documented. Independent page changes were sampled and user-accepted. See the [coordinator checkpoint](2026-10-15-coordinator-optimization.md) for the complete workload, counts, limits and separate physical acceptance receipt. This is the latest development loaded result; final signed-artifact qualification is still separate.
