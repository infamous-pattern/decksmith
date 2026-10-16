# Hardware discovery optimization — October 15, 2026

The installed development package `1.0.0-dev-discovery-d5e84a632d77` replaces repeated full HID enumeration on the permanent service's discovery worker with a bounded cache. The one-second discovery request cadence remains unchanged. On Linux, the worker checks sysfs HID topology/properties and device-node identity/permissions; an observed change triggers full enumeration. It also enumerates at least every 30 seconds. Probe failures and systems without the Linux probe fall back to the original full enumeration on every request. A failed enumeration clears the cached inventory and retries on the next request.

This cache is only an inventory optimization. Opens use the exact sequence-tagged inventory from the last successful scan and retain all existing unique-identity/path revalidation and post-open serial checks. Device input polling, coordinator wakeups, meters, rendering, saved profiles and microphone policies are unchanged. No new runtime dependency was added. Topology paths and serial metadata remain private and are not serialized in status.

## Verification before desktop installation

- Hardware-enabled Rust workspace: 341 tests passed, zero failed, five intentionally ignored. The three new discovery tests cover unchanged reuse/periodic refresh, topology/probe failures with immediate retries, and Linux replug/node permission/property changes. Final device tests and strict workspace/all-target Clippy passed after the probe refinement.
- A read-only desktop microbenchmark took 0.899 seconds for 30 full inventory calls and 0.105 seconds for 300 warm cached calls, with the same two supported candidates. This is about 85 times lower wall cost per call; it is not a whole-service CPU result. No device was opened or written by that benchmark.
- The exact package passed isolated AI-server provenance, manifest/font checks, executable hardening, relocated self-check/VirtualDeck/CLI checks and two-profile staged backup restoration after 24 subsequent saves. No AI-server services, VMs, schedules or native audio were changed by those checks.

The user saved and closed the editor before installation. A guarded update retained the previous emoji package, unchanged companion, complete per-device store and configuration/device-state backups. Both physical Decks became input/display ready, with exact original saved layouts, pages, brightness and startup enablement preserved. The user accepted normal page/audio/media/dial controls and displays on both devices. Bidirectional physical USB isolation/recovery then passed: each disconnected Deck recovered its saved page/brightness and working controls, while its connected partner retained its connection and kept responding. Both service identities remained unchanged, with no restarts, and original layout hashes were preserved. The repeated clean-idle observation is complete below.

## Evidence and release limits

Private evidence is under `outputs/v15-discovery-20261015` in the enclosing workspace: immutable source snapshot/manifest, package receipt, tests, microbenchmark, AI-server checks, guarded installation and physical phase observations. Package SHA-256: `feda63271b19fea121f4b7185dfe4227a3770b2c3d03c46a57e5b77a387b3246`. The unchanged Homebridge companion is `37adab0cf9b2886f`.

The pre-change clean paired-idle reference was 5.757% daemon CPU and 8.005% combined service CPU of one core, with 51.59 MiB daemon RSS. Do not infer the optimized whole-service result from the microbenchmark. This is a local development update, not a public V1.5 release; final version/source synchronization, signed-artifact admission and publication approval remain separate.

## Clean paired-idle comparison and physical outcome

After the user paused playback and closed the editor, the installed build completed a 15-second settling period followed by 300.002 seconds and 61 five-second samples. Both Decks stayed on their saved Home pages at 50% MK.2 and 55% Plus brightness, matching the pre-change sample. No playback streams were observed; both samples had two Decksmith peak-level capture clients and no unexpected captures. Both sessions remained input/display ready and error-free. Saved layout hashes were unchanged, both process identities stayed fixed and restart deltas were zero.

| Measurement, one core = 100% CPU | Before | Optimized |
| --- | --- | --- |
| Daemon mean CPU | 5.757% | 3.953% |
| Core service/helpers mean CPU | 7.360% | 5.551% |
| Companion service mean CPU | 0.645% | 0.630% |
| Combined service mean CPU | 8.005% | 6.181% |
| Daemon RSS throughout each sample | 51.59 MiB | 42.61 MiB |
| Core cgroup mean memory | 108.48 MiB | 83.98 MiB |
| Daemon descriptors, first/last | 24 / 24 | 24 / 24 |
| Daemon threads, first/last | 31 / 31 | 31 / 31 |

The observed combined reduction is 1.824 percentage points, or 22.79%; daemon CPU was 31.34% lower. **The combined 5% target was not met.** These are sequential finite observations with matching saved layouts, device count/page/brightness, quiet policy and sampler cadence; service ages, prior interaction history and connection epochs differ. They are not repeated randomized A/B evidence or a matched comparison with historical single-device V1. Lower RSS after an update/restart is not proof of a permanent memory saving or absence of long-term leaks.

A separate pair of thread-counter reads overlapping 295.6 seconds of this clean observation attributed about 0.162% of one core to the discovery worker and 2.500% to the coordinator. The earlier short diagnostic's discovery value was about 3.1%, under different audio/measurement conditions; those thread values must not be presented as a matched pre/post benchmark. The new result supports further investigation of coordinator work without changing the input polling loop in this update.

The authorized temporary speech socket/service pause ended in successful restoration of their original active and disabled/static enablement states. A separate post-test check confirmed both units active, Screen Reader still disabled and no Orca process. No accessibility/audio settings, volume, routing, layout or Decksmith service configuration were changed by the measurement. The existing core/companion service PIDs remained stable throughout both physical reconnect tests and the resource observation.

Both-direction physical recovery was user-accepted and machine-verified: MK.2 disconnected/reconnected while Plus kept its connection, then Plus disconnected/reconnected while MK.2 kept its connection. Each recovered its saved page, brightness and working controls; the Plus also recovered all keys, full touch strip, audio dial and mute. Normal physical page/audio/media and ordinary/quicker Plus dial checks passed before USB recovery. No numeric USB recovery deadline, visible-response latency or new ten-minute loaded resource result is claimed.

Raw resource samples, sequential comparison, speech restoration/post-check and thread counters are retained in `resources`; physical states/user confirmations are in `physical-phases` and `result.json`. The desktop remains on the optimized development package. Coordinator optimization, final-build resource/reliability review, source/version synchronization, signed candidate/download verification and release approval remain distinct next steps. No commit, push or public release was performed in this optimization.
