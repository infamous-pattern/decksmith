# Homebridge reliability and resource checkpoint - September 22, 2026

This is a bounded development acceptance pass, not an overnight soak or V1
certification. Tests use the pinned experimental OpenHomeB binary and current
Decksmith companion. Disruptive tests use loopback simulated accessories; the
real Homebridge server and real plugin process were not deliberately crashed.

## Findings and corrections

1. After a simulated read outage, discovery could recover while the legacy HB TEST
   controls remained paused. Recovery previously accepted discovery readiness as
   sufficient. It now confirms a fresh accessory read and clears the read-failure
   gate without restarting a healthy child. Manual/unknown-result guards remain
   in place: an uncertain command is never automatically retried.
2. The first sustained assignment test stopped after 1,338 successful adjustments:
   continuous input could starve the catalog refresh because it skipped every busy
   interval. Catalog freshness then expired and input was refused. Catalog reads
   now continue during action activity. A completed read from a replaced/removed
   connection cannot repopulate its state. Focused regression tests cover both
   scheduling and late-read isolation.

## Resource measurements

Live service sampling used cgroup CPU accounting, including short-lived children,
and summed process RSS at one-second intervals over 120 seconds. CPU percentages
are fractions of one logical core. Summed RSS can count shared pages more than
once; it is not unique physical memory. Other desktop/audio controls remained active.

| Scope | Average CPU | Memory observation |
| --- | ---: | --- |
| Homebridge companion plus plugin, idle | 0.56% | RSS 86.50 MiB at start/end and peak |
| Main Decksmith service and helpers, current background workload | 22.24% | RSS 64.75 to 64.39 MiB; sampled peak 102.08 MiB |

A separate 30-second attribution sample measured the main daemon at 8.73% of a
core and reaped helpers at 10.03%. Different sample windows are not directly
additive. This indicates a separate main-service optimization opportunity; it
is not evidence that Homebridge caused the main service's CPU use. The earlier
post-optimization sample also attributed material CPU to helpers.

The corrected simulated run used 20 seconds warm idle, 120 seconds alternating
one-percent adjustments, and 30 seconds settled idle. It exercised the current
schema-2 Unix socket assignment path with automatic recovery enabled. The harness
also contains the simulated HTTP server and measurement bookkeeping, so its CPU
and memory numbers are not directly comparable to the installed companion.

- 1,444 adjustments completed with no failures in the active interval.
- 95th-percentile confirmation latency: 46.49 ms on loopback, excluding the
  40 ms producer pacing delay. This is not physical-device/network latency.
- Active CPU: 5.91% for the harness/host/mock server; 0.91% for the plugin.
- Active combined RSS: 40.88 to 43.78 MiB. Initial growth includes warmed caches
  and measurement samples; this short run cannot establish absence of all leaks.
- Settled idle combined RSS was 43.78 to 43.84 MiB; CPU returned to 0.30%
  for the harness and approximately 0% for the plugin over that short window.
- Child host contexts remained bounded at four, with two host tracking tasks.

## Failure and queue coverage

Focused integration checks cover zero writes at startup, the Off guard,
EOF cancellation after an applied write, no replay after a lost acknowledgement,
child crash recovery, simulated network outage/recovery, malformed reads and a
full manager restart after a completed write. Removal/disabled persistence,
capability checks and read-only sensors also retain their regression coverage.

Five daemon queue/transport tests cover bounded coalescing, direction reversal,
stale/page-context invalidation, cancellation and discarded failed input. The
soak tests companion transport at a paced rate; these queue tests separately cover
bursts before that transport. Simulated restart coverage is not a full desktop
logout/suspend/reboot test.

## Evidence and remaining work

Machine-readable samples and the live check are retained under
`local/homebridge-reliability-2026-09-22/`. The isolated harness and loopback tests
are in `outputs/plugin-testing/host-prototype/`; runtime regression sources are
also preserved under `plugins/openhomeb/runtime/`.

Continue with main-service/helper profiling before claiming low idle CPU for the
whole application. Longer runs, authenticated-session expiry, multiple target
loads, additional physical accessories and suspend/logout recovery remain broader
acceptance work. Keep the V1 usability pass after the current reliability findings
are reviewed. No general plugin compatibility or full sandbox claim is added.

## Installed fix and live check

Companion release `1a3248c5c40ef86e` is installed, with rollback backup
`20260922-092743-745264`. The main daemon and editor were not restarted or replaced.
The live Main_LED’s test briefly enabled the originally-off light, confirmed
100% -> 99% -> 100%, then restored Off. Homebridge reads confirmed original
Hue 216 and Saturation 96, and the saved layout matched exactly. The two level
confirmations took 647 and 675 ms; this is API readback, not a user visual check.
No other real devices were actuated. Changes are not committed or pushed.

A final 30-second post-install sample measured companion CPU at 0.57% of one
core. Summed RSS warmed from 55.84 to 60.67 MiB after the restart; this short sample
is not evidence of a memory plateau or a leak. The main service measured 18.19%
in that window. Compare the longer live baseline and simulated settled interval
separately rather than treating fresh-process and warmed-process RSS as equivalent.
Installed source matches the corrected runtime; all 61 functions remain available.
