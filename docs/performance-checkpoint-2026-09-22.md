# Background-service optimization — September 22, 2026

Installed development build `0.1.0-e4129e376ba0` reduces repeated helper startup
and Homebridge snapshot parsing without changing polling or meter rates.

## Changes

- Reuse the read-only control-health Python helper. Each request still creates
  fresh status checks. Requests and responses are bounded; timeout, malformed
  output, mismatched checks and shutdown discard the child. Subsequent checks
  start a fresh child rather than accepting a late response.
- Parse each unchanged private Homebridge snapshot once per reading thread.
  Every access still checks file identity, ownership, permissions and the existing
  three-second freshness limit. Replacement, deletion and expiry cannot keep an
  old snapshot available. No network work was added to rendering.

The initial 20-second process sample observed 17 control-health helper launches.
The installed update keeps one helper available between checks.

## Measurements

Two 60-second workstation samples include all main-service child processes via
cgroup CPU accounting. Percentages refer to one logical CPU core, not the whole
machine. These are indicative sequential samples, not a controlled benchmark.

| Main service plus helpers | Before | After |
| --- | ---: | ---: |
| Average CPU | 18.354% | 6.579% |
| Summed RSS at end | 64.04 MiB | 89.19 MiB |
| Sampled peak summed RSS | 100.80 MiB | 94.71 MiB |
| Cgroup memory at end | 47.80 MiB | 44.85 MiB |

CPU was about 64% lower. Keeping Python alive increases steady summed RSS;
shared pages are counted more than once in that metric. Cgroup memory was lower
in this sample, but the restart also resets allocations and caches, so this is
not evidence of a lasting memory reduction. After-sample RSS rose from 88.94 to
89.19 MiB; a longer soak remains necessary. Companion CPU stayed near 0.56%.
Brave reported no playback stream at the post-install check; this pass does not
certify performance with four independent active meters.

## Verification and recovery

- Hardware-enabled workspace tests: 115 passed, one ignored.
- Studio Python suite: 136 passed.
- Focused coverage includes persistent response matching/cancellation, fresh
  per-request checks, request bounds, snapshot replacement and stale/private-file
  validation. A real helper returned two missing-player responses without exiting.
- Installed daemon reports connected and display ready. All five saved config
  files match the installer backup byte for byte. No live accessory action was
  sent during this optimization pass.
- Prior version: `0.1.0-dbf9d465765c`; installer backup:
  `20260922-104503-847483.tar.gz`. Startup preference was preserved.

Raw measurements are under `local/cpu-2026-09-22/` (excluded from Git).
The user confirmed that everything works normally after installation.
Next acceptance work: longer active-audio sampling, helper-failure recovery under load, and repeated
suspend/reconnect checks. Further optimization can target repeated audio inventory
subprocesses without slowing visible feedback. This work is not a V1 performance
or leak certification.
