# Signed V1 CPU investigation — 2026-10-07

This follows the [quiet measurement](2026-10-07-v1-quiet-resources.md) and
[physical interaction sample](2026-10-07-v1-physical-resources.md). The desktop
remains on signed candidate `1.0.0-c5e4fea53afe`, public source
`68c42421ceaa65bb033993b56f0babdde2a14a6e`. No runtime code, installed build,
polling setting, audio setting or service configuration was changed.

The investigation identifies existing CPU costs and a useful post-V1 optimization
path. A limited paired guest comparison did not reproduce the larger desktop
increase. It does **not** establish the cause of that historical increase or
convert different-day desktop observations into a matched regression test.

## Read-only desktop observations

After the user paused playback, a sixty-second diagnostic sampled thread CPU,
scheduling and wait states twice per second, and cached touch frames every five
seconds. The device was connected and unlocked; its service identity and runtime
layout stayed unchanged. All thirteen frame observations were identical in each
of the four slots. The microphone was muted. No observed meter animation accounts
for this diagnostic's steady-state work.

| Observation | Result |
| --- | --- |
| Main daemon CPU | 2.35% of one logical core |
| Full core service/helpers CPU | 4.13% of one logical core |
| Main-thread CPU | 0.87% of one logical core |
| Display/device worker CPU | 0.88% of one logical core |
| Meter helper CPU | 0.92% of one logical core |
| Running process count | 857–877 |
| Main-thread / display-worker run-queue wait | 1.71 / 1.87 ms total over sixty seconds |

Most sampled waits were ordinary timers, input polling or blocked synchronization.
The two largest daemon threads were not persistently runnable or starved for CPU.
This short diagnostic includes observer overhead, omits helpers that exit between
snapshots and does not replace the five-minute quiet qualification.

In a separate ten-second observation, the main thread made approximately **1,742
read calls per second**, reading about 13.7 KB per second without disk-read bytes.
Source inspection shows the physical-session main loop scans process names once
per second to detect a competing OpenDeck process. The counters support this as
an existing source of work that depends on desktop process count; they do not
attribute every syscall or prove its contribution to yesterday's difference.

## Sampled comparison hotspot

A sixty-second, requested user-space cycle profile of the existing daemon and its
helpers found repeated byte comparisons to be the largest sampled location in
the display/device worker. The sampled caller was identified as
`display::Display::pending`, which compares desired and sent image buffers. The
worker checks pending display state while scheduling input and updating status,
even when the nine display slots have settled. `flush_one` also scans for changed
buffers. These checks preserve fairness and duplicate suppression, but repeatedly
traverse image data while idle.

The previous and signed binaries both record Rust 1.97.1 and the same compiler/
linker versions. The previous unstripped binary's ELF build ID matches its
packaged reference binary. A unique routine-prologue match identified the signed
candidate's pending-display routine: all **703 bytes** match after normalizing
the nine indirect-call address displacements. This supports an existing hotspot,
not a newly introduced implementation of that routine. It is not proof that
every instruction in both complete executables is identical.

The profile is sampled execution, not a complete function-time census. The
stripped signed binary limits named stacks; kernel/off-CPU work and unresolved
locations are not fully attributed. Raw stacks stay private. Fedora's profiling
tool was downloaded, verified against installed Fedora trust keys and extracted
only in the ignored test directory; no host package or profiling permission was
changed. No debugger stop or daemon restart was used.

## Paired Fedora 44 VirtualDeck comparison

The unused retained Fedora Workstation 44 VM ran the exact previous and signed
daemon binaries, verified by their recorded SHA-256 digests. Both used identical
pages, resources, static fake audio/health helpers and a private D-Bus/XDG test
environment. A ten-second warmup preceded each sixty-second measurement, in
old/signed/signed/old order. The guest has four vCPUs and 8 GiB RAM.

| Run | Binary | Main-daemon CPU, one core |
| --- | --- | --- |
| 1 | Previous | 1.317% |
| 2 | Signed V1 candidate | 1.300% |
| 3 | Signed V1 candidate | 1.450% |
| 4 | Previous | 1.400% |
| Mean | Previous | 1.358% |
| Mean | Signed V1 candidate | 1.375% |

The mean difference was 0.017 percentage points of one core, approximately 1.23%,
smaller than the variation among these short runs. Two runs per binary do not
support a statistical equivalence claim. Every run produced the same steady touch
frame hash and kept fourteen descriptors. Main RSS ranges were 11.438–11.766 MiB
across previous-binary runs and 11.832–12.137 MiB across signed-binary runs.

This measures the shared engine under a constrained fixture. It excludes physical
USB, real audio/health helpers, the physical competing-process scan, full-service
CPU and visible pixels. Private-bus screen-saver activation messages are fixture
environment noise, not native GUI or lock acceptance. These limits prevent
calling it a matched physical-desktop A/B test or a replacement release gate.

The normal guest's saved data, current-release link and startup setting were
verified unchanged. Temporary test files were removed, and the guest shut down
normally. Final verification found all four test VMs off with autostart disabled.
The desktop retained its original daemon process, zero restarts, a ready display
and the same hashes for all five saved JSON files.

## Follow-up and acceptance boundary

Keep the signed candidate unchanged while completing its remaining qualification.
The observed CPU costs are concrete profiling leads, and this guest comparison
does not show the large engine increase suggested by the unmatched desktop
measurements. The physical difference remains unproven; any later physical A/B
would need a separately coordinated recoverable build switch and matched conditions.

Post-V1 CPU work should first evaluate tracking dirty display slots or frame
revisions, avoiding repeated full-buffer comparisons once settled. Preserve
latest-wins coalescing, duplicate suppression, fair input polling, failed-write
retry, reconnect repaint, Auto-Lock and complete shutdown blanking. Profile the
process-name scan before changing its cadence or competing-application protection.
Measure idle and loaded full-service CPU, memory and latency together; do not
hide work by moving it into more threads or helpers.

The below-1% idle-daemon goal remains post-V1. This investigation adds evidence
and a bounded optimization plan; it does not close precise visible-response,
clean dependency provisioning, lifecycle/accessibility, optional-companion or
release-acceptance gates.
