# Performance baseline — September 17, 2026

This bounded diagnostic pass measured the accepted polish checkpoint at Git
`237b142`. Installed interface: `0.1.0-f0ae15327cb2`; running daemon came from
`0.1.0-ba281d7236ac` (the later update changed only the interface).
No device actions, audio routing changes, service restarts or layout saves were
performed. The user confirmed audio was playing during the run.

## Workloads and observations

The physical layout retained three audio dials (output, input and application)
and a brightness dial. A 180-second read-only service cgroup sample included the
daemon and its helper processes. It averaged **44.7% of one CPU core**. The first
two 30-second intervals, before the isolated editor workload, measured 40.4% and
44.0%. Anonymous service memory ranged from **38.5 to 58.9 MiB**, finishing at
40.4 MiB; this metric excludes file-backed pages and is not editor RSS.

A separate 30-second attribution sample measured daemon CPU at **6.1% of one
core**, with **37.8%** charged to reaped child processes. The samples have different
time boundaries and should not be treated as an exact additive breakdown of the
180-second average.

A separate native GTK editor copied the saved layout in memory and substituted
four audio panels. It used real read-only daemon preview rendering, while page
switching was simulated and all action/persistence calls were blocked. The fourth
panel used cached system state; this does **not** certify four independent live
meter subscriptions. The test used the Cairo renderer, so editor CPU figures are
not measurements of the user's normal GPU rendering configuration.

| Editor phase, about 30 seconds each | CPU (% of one core) | End RSS | Preview latency p95 |
| --- | ---: | ---: | ---: |
| Visible four-panel preview | 7.9% | 166.8 MiB | 6.4 ms |
| 100 isolated page changes | 62.0% | 170.3 MiB | 10.9 ms |
| About tab, preview hidden | 0.8% | 175.7 MiB | One in-flight completion |
| Visible preview again | 7.4% | 176.2 MiB | 6.3 ms |

The 100-ms main-loop heartbeat's p95 excess delay during page changes was 2.4 ms.
No preview errors occurred. File descriptors remained bounded (23–27 in the
samples). Threads dropped from 48 to 22 in the hidden phase and rose again when
visible; no thread-leak conclusion is drawn from this short run. RSS rose during
warm-up and page switching, then remained nearly flat during the final visible
phase. This is not a long-duration leak certification.

## Priority improvement

Background helper-process CPU is the clearest performance concern for a
lightweight utility. `audio_worker.rs` separately reads named targets and system
volume. `audio_target.rs` starts a Python process for each read; each
`audio_targets.py` snapshot invokes `pactl` commands. Audio state is eligible for
refresh every 250 ms. Child CPU attribution supports investigating this path;
it does not establish that every child CPU cycle came from these reads.

Recommended next work:

1. Share one inventory snapshot for named targets and system volume.
2. Reuse a bounded helper connection rather than repeatedly starting Python.
3. Prefer change-driven updates or a carefully bounded cache for volume/device
   state, while retaining independent fast peak-meter updates and immediate
   refresh after user actions.
4. Repeat the same before/after workloads, including control latency, external
   volume/mute changes, missing targets, reconnect and helper failure recovery.

## Still required before V1

- A longer soak with four independent active audio sources and normal GPU rendering.
- Repeat editor-open/closed measurements after optimizing helper reads.
- Larger layouts and extended page-switching/history stress.
- Large-text, high-DPI, keyboard and screen-reader acceptance.
- Coordinated repeated suspend/reconnect tests; no disruptive lifecycle test was
  performed in this pass.

Local raw evidence and probe scripts remain under `local/performance/` and are
excluded from Git. Summarized results intentionally omit user targets and layouts.

## Refresh optimization follow-up

Installed `0.1.0-414375fe1bda` with configuration backup and a brief service
restart. Volume/mute reads share a fresh snapshot through one persistent reader
every 500 ms; actions invalidate the wait immediately. Meter publication and
visible touch preview requests now run at up to 20 Hz. Discovery reconciles every
two seconds; visible pickers retain topology-event refresh.

The repeated 180-second service sample averaged **21.3% of one core**,
versus 44.7% before (about 52% lower). Anonymous memory
ranged from 3.7 to 60.6 MiB and ended at 54.9 MiB.
Playback was confirmed active; the same physical layout and isolated editor
workload were used, but playback content and workload overlap were not identical.
These are indicative workstation measurements, not a controlled benchmark.

| Editor phase | CPU (% of one core) | End RSS (MiB) | Preview p95 (ms) |
|---|---:|---:|---:|
| visible | 14.96 | 166.93 | 3.25 |
| page_changes | 69.8 | 177.23 | 6.72 |
| hidden | 0.2 | 177.2 | no requests |
| visible_again | 12.32 | 177.64 | 3.34 |

The software-rendered visible editor costs more CPU at 20 Hz than at 10 Hz;
hidden previews made zero requests. No long-duration leak claim is made.
The reader was deliberately terminated once and automatically replaced in
0.99 seconds; the replacement remained alive, with the device
ready and page preserved. No volume, routing or playback actions were issued.

Verification: 102 Rust tests passed (one hardware-only test ignored), 110 Python
tests passed, strict Clippy and formatting passed, and the native GTK workspace
check passed. Tests cover shared snapshots, fresh subsequent reads, malformed and
oversized requests, broken response channels, and immediate refresh invalidation.
External audio control latency and repeated suspend recovery remain manual
acceptance checks for this update. Raw before/after measurements remain local.
