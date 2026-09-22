# Active-audio reliability checkpoint — September 22, 2026

## Result

The ten-minute reference-desktop sample completed without service restarts or
sampled connection/display failures. All 17 focused simulated recovery, helper and
queue tests passed afterward. No application-code change was indicated by this
bounded pass. This is not an overnight soak, a leak certification, or acceptance
of all audio targets and hardware configurations.

## Environment and method

- Fedora Workstation 44; kernel `7.2.6-200.fc44.x86_64`; 32 logical CPUs.
- Running daemon and installed application: `0.1.0-739b30218d15`; executable hashes matched.
- Installed companion: `5912e6fda323ef3e`; tested runtime modules matched the repository byte for byte.
- Source checkout: `f8d7c6ec0920561836d3bf3f8d3806f1c14a185b`.
- 600 seconds, 121 resource samples at five-second intervals.
- 41 read-only status/audio checks at fifteen-second intervals; one uncorked playback stream was present at every check. The user confirmed playback and was invited to use audio dials normally; dial-event frequency was not recorded.
- CPU uses cgroup accounting and includes service children. Percentages are of one logical CPU core. Memory is cgroup memory, including accounted cache, rather than summed RSS.
- Recovery tests ran after the resource sample to avoid contaminating the workload. No service was deliberately restarted, no physical accessory commands were sent by the harness, and the desktop was not suspended or locked.

## Resource results

| Scope | Average CPU, one core | Start memory | End memory | Sampled peak |
|---|---:|---:|---:|---:|
| Decksmith plus helpers | 9.87% | 63.44 MiB | 64.02 MiB | 64.81 MiB |
| Companion plus plugin | 0.56% | 55.71 MiB | 55.05 MiB | 56.77 MiB |

Both services retained their main process IDs and zero restart counts. All 41
status checks reported connected and display ready with an empty attention list.
Checks between samples could miss very brief failures.

Persistent daemon/helper processes remained present in all 121 samples. Main
daemon RSS changed from 17.64 to 17.66 MiB, with 18 threads throughout. Its three
persistent Python helpers each grew by less than 0.03 MiB. Their file-descriptor
ranges remained bounded (8–10, 10–12 and 8–10). The companion manager grew by
about 0.08 MiB RSS; plugin RSS ended below its initial value. No accumulating
thread/handle trend was observed in this interval.

The earlier 6.58% CPU result used a different short workload without confirmed
Brave playback; it is not a like-for-like regression benchmark for this active
sample. These results do not justify lowering polling or meter update rates.

## Focused checks

Seven companion integration tests used the installed runtime and loopback mock
accessories. A socket guard rejected non-loopback connections. Coverage included:

- Read-outage recovery without writes.
- Full manager restart without replaying completed commands.
- Malformed reads without writes.
- Lost acknowledgement after an applied write, without replay.
- Plugin crash/reconnect with read-only recovery.
- Catalog refresh during busy action processing.
- Late catalog reads unable to restore removed/replaced state.

Four daemon health-helper tests passed: reuse with response matching, shutdown
cancellation, stale session/page/binding rejection, and distinct failure feedback.
Six daemon plugin-runtime tests passed: bounded burst coalescing, failure/stale
discard, direction reversal, page-context invalidation, transport cancellation
and private snapshot freshness. These are isolated checks, not injection of
faults into the user’s live services.

## Next acceptance work

- Continue V1 keyboard, high-contrast, enlarged-text and scaling review.
- Retain longer memory observation, four independent active-meter workloads,
  authentication/session expiry and varied physical USB/suspend recovery as open.
- A dedicated repeatable high-rate dial workload is still needed for combined
  end-to-end latency/resource acceptance; the queue tests cover bursts separately.

Raw samples, environment/build identities, summary, scripts and test logs are
retained in `local/sustained-2026-09-22/` and excluded from publication.
No installed build or saved assignment was changed by this pass.
