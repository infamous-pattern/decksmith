# Sustained physical qualification — 2026-10-06

The reference Fedora Workstation 44 desktop completed a ten-minute physical
interaction and resource sample on bundle `0.1.0-257113d71d41`, built from clean
source `1a0101da0f38d9c5d90c9131d5238d9ee74ef0e9`. The archive SHA-256 is
`4de46d33b1da65cd9b76cf8d136eda0c068c7c248a11ae0db44f2b3e36ac6703`.
This remains a development qualification artifact, not the frozen V1 release.

The user exercised physical dials at ordinary and quicker speeds and switched
pages, confirming: “Tested both speeds and pages; everything worked.” No freeze,
missed input, incorrect label/icon/meter or wrong-target change was reported.
The daemon stayed connected and unlocked, with its process identity unchanged
and zero automatic restarts. All 148 recorded actions succeeded.

## Workload and coverage

The sample lasted 600.004 seconds and collected 121 five-second observations.
The four target assignments were output, microphone, Brave and Chrome. Both
browser playback streams remained uncorked at the thirty-second inventory checks.
The output meter includes the browser mix; these are four target identifiers,
not four independent playback sources.

Chrome temporarily replaced Dial 4 only through overrides on Home, System and
Media. The Homebridge test page retained its existing fan override. One observation
captured a brief visit to that page, so four expected meter targets were present
in **120 of 121 observations**, rather than throughout every observation. All 120
four-target preview checks showed live meter frames without unavailable markers;
each of the four activity bars changed across the sample.

The sampler issued no volume, mute, media or accessory commands. Physical user
actions supplied the workload. Observer reads may contribute some daemon work,
and periodic checks can miss faults between observations. Expected unsupported
previous/next capabilities reported by the browser are not evidence of an audio
target disconnect or a failed recorded action.

## Resource results

CPU percentages use one logical core as 100%. Core-unit figures include its
helpers; the optional experimental Homebridge companion is reported separately.
Cgroup memory includes helpers and file-cache accounting, and is not main-process
RSS.

| Measurement | Result |
| --- | --- |
| Main daemon mean CPU | 2.54% |
| Core service and helpers mean CPU | 4.66% |
| Core five-second CPU interval p95 | 8.11% |
| Experimental companion mean CPU | 0.56% |
| Core plus companion mean CPU | 5.22% |
| Main daemon RSS | 28.324–28.348 MiB |
| Main daemon open handles | 16–20 |
| Core cgroup memory | Mean 72.60 MiB; range 70.13–94.99 MiB |
| Companion cgroup memory | Mean 56.27 MiB; range 55.25–57.44 MiB |

Main RSS grew by approximately 24 KiB across this finite run. Core anonymous
memory changed from 54.18 to 55.67 MiB; file memory was 12.16 MiB at both ends.
Core process count ranged from four to six during short-lived action helpers.
The companion retained the same process identity and zero automatic restarts.
These results support bounded resource use for this workload, not a long-term
leak exclusion or a matched comparison to earlier quiet measurements. The approved
below-1% idle CPU enhancement remains post-V1.

## Physical dispatch evidence

The 148 successful actions included 133 physical dial adjustments: 30 output,
30 microphone, 33 Brave and 40 Chrome. Dial dispatch median was 0.023 ms,
p95 **15.745 ms**, and maximum 19.378 ms. The remaining actions were two
application launches, two key mute actions, two media play/pause actions and eight
touch mute actions. Their observed dispatch maxima were below 0.052 ms. Four
physical page-render invocations had a maximum of 0.074 ms.

The timestamp boundary is the pinned hardware library's report return through
backend entry or page-render invocation. It excludes USB/firmware delay, backend
completion, display transport and visible pixels. The observed routes met the
25 ms dispatch target; this does **not** close precise visible-response timing or
prove every action route meets its release target.

## Restoration and remaining work

The original runtime layout and active page were restored. All five saved JSON
configuration files were verified byte-for-byte against their pre-test backups.
Dial 4 again uses System sounds, with the original brightness, Auto-Lock preference
and companion readiness retained. User-operated volume changes were retained.
No service restart was needed for restoration.

Raw configurations, recordings and samples remain in ignored local evidence;
they are excluded from public source exports. Remaining V1 work includes precise
visible-pixel timing, release authenticity, a frozen final artifact and its
complete installer, lifecycle, regression and security qualification. This
checkpoint adds sustained physical acceptance and resource evidence without
replacing those gates.
