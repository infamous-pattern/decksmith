# Signed V1 quiet resource observation — 2026-10-07

The reference Fedora Workstation 44 desktop completed a valid 300.001-second
quiet observation of signed candidate `1.0.0-c5e4fea53afe`, public source
`68c42421ceaa65bb033993b56f0babdde2a14a6e`. Its archive SHA-256 remains
`4334a2df091ccdb0fac2431e416fa51cf4dabe0da1dfe4ccc372f60029a3dda2`.
See the [candidate checkpoint](2026-10-06-v1-signed-candidate.md) for signing,
installation, VM and bounded physical acceptance evidence.

## Conditions and results

The user paused playback. Preflight confirmed the editor was closed, no uncorked
playback or non-meter capture streams were present, and the device was connected,
display-ready and unlocked. Other applications could remain open. The observer
collected 61 resource points five seconds apart, with audio/device checks every
30 seconds. It issued no control, volume, layout or service-change commands.

The daemon retained the same process identity and zero restarts. Every sampled
connection/display/lock check passed; every audio check was inactive. The service
journal contained zero input, queued-action, page-request or action-result records
during the measurement. Those checks establish this finite quiet sample, not
continuous observation between sample points.

CPU percentages below express one logical CPU core as 100%. Daemon CPU sums its
threads; it is not a measurement restricted to a single core. Core service CPU
includes its helpers and exited-child work accounted by the service cgroup.

| Measurement | Result |
| --- | --- |
| Main daemon mean CPU | 2.46% |
| Core service/helpers mean CPU | 4.42% |
| Core five-second CPU interval p95 | 5.26% |
| Experimental companion mean CPU | 0.17% |
| Core plus companion mean CPU | 4.59% |
| Main daemon mean RSS | 17.94 MiB |
| Main daemon RSS range | 17.934–17.949 MiB |
| Main daemon open handles | 16–17 |
| Core cgroup memory | Mean 65.52 MiB; range 64.29–66.87 MiB |
| Companion cgroup mean memory | 34.53 MiB |

The companion retained its process identity. Main RSS grew by 16 KiB across the
finite sample and remained below the 100 MB idle daemon target. Cgroup memory
includes helpers and file-cache accounting and must not be confused with daemon
RSS. These observations do not exclude a long-term leak or qualify configurator
memory, interactive resources, physical dispatch or visible-pixel timing.

## Comparison and investigation boundary

The reference baseline in the [V1 scope](../v1-release-scope.md) is 2.11% daemon
and 3.75% full core-service CPU. This candidate's observation is approximately
16% and 18% higher, respectively. The closer previous-build quiet observation,
bundle `0.1.0-257113d71d41`, measured 1.63% daemon and 2.91% core-service CPU:
the new observations are approximately 51% and 52% higher. Its core memory mean
was 64.68 MiB, close to this run's 65.52 MiB; daemon RSS was 18.56 MiB.

These are different-day/build/service-lifetime observations, not a matched A/B
benchmark. The CPU increase is recorded for investigation and is **not explained
or accepted as a regression-free performance result** by this checkpoint.

A subsequent ten-second read-only attribution observed approximately 2.10% daemon,
0.30% audio-target reader, 0.30% control-health helper and 1.00% audio-meter helper
CPU. The current power profile was Balanced. Core memory accounting at that point
included 45.79 MiB anonymous, 15.51 MiB file and 2.62 MiB kernel memory. This short
observation omits children that exit between snapshots and does not replace the
five-minute measurement or establish a cause. An absent optional power-profile
CLI was handled by querying the existing desktop API; no profile was changed.

The daemon already uses separate threads for device work, audio jobs, meters,
session lock and background monitoring; 18 threads were sampled. The editor has
bounded background executors with GTK updates returned to its main thread.
Adding threads alone does not reduce summed CPU time. No threading, scheduling,
polling, runtime or installed-artifact changes were made during this work.

The approved below-1% idle daemon CPU target remains post-V1. The requirement to
review unexplained resource increases still applies. This checkpoint completes
the deferred quiet measurement, while loaded resources, CPU comparison, lifecycle,
accessibility, visible-response and remaining installation/companion/release gates
stay open. Raw samples, journal and attribution details remain private.

The subsequent [physical resource checkpoint](2026-10-07-v1-physical-resources.md)
records the completed ten-minute mixed-page run and user acceptance. CPU
comparison remains open, and one observed Homebridge page visit means it is not
an uninterrupted four-target certification.

The later [CPU investigation](2026-10-07-v1-cpu-investigation.md) records observed
steady-state costs and a constrained paired VM comparison. It does not establish
the cause of the different-day physical CPU increase.
