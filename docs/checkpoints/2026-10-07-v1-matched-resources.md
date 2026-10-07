# V1 matched physical resource comparison — October 7, 2026

**Two quiet pairs complete across a user-requested pause. Signed candidate
restored. This is not a continuous A/B/B/A experiment or a statistical equivalence
claim; final resource/release acceptance remains separate.**

The user saved edits, closed the editor, paused playback and authorized the
[A/B/B/A protocol](../v1-matched-resource-protocol.md), including temporary build
switches and restoration of the signed candidate. Managed configuration/artwork
backups and saved-state/routing fingerprints were retained privately.

## Attempts and valid evidence

The first attempt stopped during previous-build warmup when playback became
active; no measured phase completed. The second logged warmup audio separately
and required a quiet graph after warmup and throughout each measured phase.
Its first previous-build phase passed those conditions, but transient playback
interrupted the candidate phase. Neither interrupted phase is a quiet CPU result.

The user then confirmed all audio was stopped and requested a retry. The third
attempt completed another previous-build phase, but stopped during the candidate
phase on two active non-Decksmith capture streams. An immediate read-only
observation found two GNOME Settings (`gnome-control-center`) input-monitor
streams and Decksmith's separately allowed meter streams, with no playback.
This identifies a non-quiet graph consistent with the stop counts; paused music
alone does not guarantee inactive external capture. No audio settings were changed.

| Completed phase | Observation |
| --- | --- |
| Bundle / source | `0.1.0-257113d71d41` / `1a0101da0f38d9c5d90c9131d5238d9ee74ef0e9` |
| Sample | 300.003 seconds, 61 points, after at least 90 seconds of warmup |
| Daemon / core cgroup CPU | 1.707% / 3.025% of one logical core |
| Companion CPU, measured separately | 0.550% of one logical core |
| Daemon RSS / file descriptors | 17.379–17.391 MiB / 16–18 |
| Core cgroup mean memory | 47.299 MiB |
| System process count / power profile | 797–811 / balanced |
| Automatic restarts / unplanned action records | 0 / 0 |

At the end of the second attempt there was only one valid A phase, no valid B
phase and no complete A/B/B/A result.
These numbers cannot establish a candidate regression, equivalence, improvement
or sustained below-1% CPU use. Existing finite candidate resource observations
remain unchanged; comparison acceptance remains open.

The third attempt's valid A phase measured 1.447% daemon / 2.555% core / 0.539%
companion CPU over 300.000 seconds and 61 points. Daemon RSS was 17.207–17.219
MiB, descriptors stayed at 16, mean core memory was 46.399 MiB, and process
counts ranged from 689 to 723. There were no automatic restarts or unplanned
actions. It has no valid B counterpart; do not pool A phases from different
attempts into a completed A/B/B/A result.

## Restoration and observation limits

After each stop the reviewed installer restored `1.0.0-c5e4fea53afe`. The second
runner reused its quiet assertion in finalization: playback made it report
`restored: false` after activation had succeeded. This is a harness reporting
limitation, not evidence that the runtime failed to recover. A separate read-only
follow-up confirmed all 389 manifest files, the expected running executable,
connected/display-ready state, original active page, preserved configuration,
artwork and audio routing, enabled login preference and zero automatic restarts.
The original generated cache in the retained previous build was also preserved.
Future finalization must assess readiness and state separately from sample quiet.
The third runner separated those checks and correctly recorded restoration as
successful despite the invalid sample: candidate connection, original page,
saved files/routing and login preference were verified after restoration.

A subsequent 120-second audio-event observation recorded one new corked stream
with application name `Chromium` and process binary `ChatGPT`. It did not identify
the earlier uncorked stream or prove the interruption's source. No audio was
captured, media titles collected, audio routing/volume changed or apps paused by
the test. All four testing VMs remain off. Raw evidence stays in ignored local
qualification storage; signed files were not modified or repacked.

## Fourth attempt: quiet pair completed, repeats paused

The user closed GNOME Settings and authorized another retry. Quiet preflight and
all nine warmup checks passed for each completed phase. The same saved layout,
active page, routing, balanced power profile and core/companion policy were used.
Each phase had a fresh explicitly started daemon, at least 90 seconds of warmup
and 61 points over 300 seconds. All sampled playback/capture checks were quiet;
no physical action records, automatic restarts or connection/lock failures were
observed in either completed phase.

| Metric | A: retained previous | B: signed candidate |
| --- | --- | --- |
| Daemon CPU, percentage of one logical core | 1.457% | 1.533% |
| Core service cgroup CPU, including helpers/exited-child work | 2.583% | 2.766% |
| Separate companion CPU | 0.523% | 0.541% |
| Daemon RSS | 17.133–17.145 MiB | 16.793–17.020 MiB |
| File descriptors | 16 | 16–17 |
| Mean core cgroup memory | 46.282 MiB | 46.420 MiB |
| System process count | 692–710 | 686–733 |

The candidate was higher by 0.077 percentage points of one core for the daemon
(about 5.3%) and 0.183 points for the core service (about 7.1%). This is a single
ordered pair, with overlapping but varying host process counts. It does not
establish statistical equivalence, a repeatable regression, a cause of the older
different-day increase or a universal CPU guarantee. Runtime daemon/device source
is unchanged between the two accepted source commits; version and release-tool
changes do not prove complete binary or performance identity.

The second candidate phase had begun when the user requested a pause. It retained
seven points over about 30 seconds before cancellation; it is **not** a completed
five-minute phase and is excluded from the pair above. The fourth baseline phase
was not started. The runner's generic `stopped-invalid` status records
`KeyboardInterrupt: Cancellation`; this attempt stopped at the user's request,
not because an audio, lock or connection gate failed.

Cancellation ran the recovery finalizer and restored the signed candidate.
Independent read-only verification confirmed all 389 manifest files, expected
running executable, connected/display-ready state, original page, unchanged saved
data and audio routing, and enabled login startup. No further measurement was
started. Raw evidence remains private under the fourth attempt directory.

## Resumed B/A pair completed

The user subsequently requested continuation. Fresh preflight confirmed quiet
playback/capture, a closed editor, connected/unlocked/ready device, and unchanged
saved configuration/artwork, active page and default audio routing relative to the
first pair. Both resumed phases used balanced power policy, a fresh deliberately
started daemon, at least 90 seconds of warmup and 61 points over 300 seconds.
Every sampled graph was quiet. Neither phase recorded automatic restarts,
unplanned inputs/actions, connection loss or a lock transition.

| Metric | B repeat: signed candidate | A repeat: retained previous |
| --- | --- | --- |
| Daemon CPU, percentage of one logical core | 2.010% | 2.003% |
| Core service cgroup CPU | 3.717% | 3.702% |
| Separate companion CPU | 0.603% | 0.606% |
| Daemon RSS | 16.863–16.875 MiB | 17.125–17.148 MiB |
| File descriptors | 16 | 16 |
| Mean core cgroup memory | 46.504 MiB | 46.250 MiB |
| System process count | 724–752 | 716–738 |

This second pair's candidate difference was 0.007 percentage points for the
daemon and 0.014 for the core service. Both builds measured higher than in the
first session. Host process counts also varied; the measurements do not establish
which environmental factor caused the difference between sessions.

For descriptive reporting only, the two completed phases per artifact average
1.730% previous / 1.772% candidate daemon CPU and 3.143% / 3.241% core CPU.
Separate companion means were 0.565% / 0.572%. The candidate daemon difference
is 0.042 percentage points (about 2.4%), and core difference is 0.098 points (about
3.1%). These means do not erase the pause, provide a confidence interval or turn
two ordered pairs into statistical equivalence. The larger increase suggested
by earlier different-day observations was not reproduced in these pairs; a
universal regression-free guarantee or cause attribution remains unsupported.

All four complete phases were independently recalculated from raw CPU counters;
the incomplete cancelled phase is excluded. No unplanned actions or restart
growth was observed, and memory/handles remained bounded. Below-1% idle daemon
CPU is still a post-V1 goal, not an achieved target.

The resumed runner restored the signed candidate successfully. Independent
read-only verification again admitted all 389 files, matched the running daemon,
confirmed connected/display-ready state and the original page, and verified
saved data, routing and enabled startup. All four test VMs were off. Raw evidence
and the descriptive split-session summary remain private. No runtime source or
signed artifact changed, and no public release or push was performed.

The requested measurements are complete with the pause and finite-sample limits
recorded. Final resource and release acceptance must consider these observations
together with the existing interactive sample and known supported-platform limits.
