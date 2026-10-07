# Matched physical resource comparison — protocol

The October 7 reference-desktop comparison was authorized, initially interrupted
by playback/capture, then completed one quiet A/B pair before the user paused the
repeat runs. On continuation the remaining B/A pair completed with fresh
preflight; see the [outcome checkpoint](checkpoints/2026-10-07-v1-matched-resources.md).
All four measured phases completed across two sessions, not a continuous A/B/B/A
experiment. The protocol requires
saved editor drafts, a closed editor, paused playback and an unlocked awake Fedora
session. It briefly interrupts Decksmith background controls at build switches.
No power operation, audio command or temporary dial assignment is part of it.

## Targets and preservation

- A: retained previous bundle `0.1.0-257113d71d41`, source
  `1a0101da0f38d9c5d90c9131d5238d9ee74ef0e9`.
- B: signed candidate `1.0.0-c5e4fea53afe`, source
  `68c42421ceaa65bb033993b56f0babdde2a14a6e`.
- Final state: B running and connected, original saved configuration/artwork,
  active page, brightness, audio routing and login preference restored/preserved.

Read-only preflight found all 379 A and 389 B listed installed files match their
manifests. B has no extra files. A also contains an unlisted generated Python
cache, so its installed directory is **not** admitted as a clean archive. Use the
retained previous archive for admission and record its hash. Preserve the generated
cache before removing it if clean installed-directory admission requires this;
never silently ignore unexpected files or modify the signed candidate.

Take a managed configuration/artwork backup and record hashes, integration and
current-release state before any switch. Retain both original archives, installer
and backup paths privately. Use the reviewed installer with explicit activation
for recoverable switches; no hand-edited service unit or generic raw symlink swap.
The old bundle is an accepted prior comparison artifact, not a newly signed V1
download. Do not publish it as part of the candidate's provenance.

## Measurement

1. Verify candidate identity, archive admission, current connection/display/lock,
   saved-file hashes and absence of active playback or non-meter capture streams.
   Require the same saved layout, monitored targets, power profile and companion
   policy in every run. Record process count and observer cadence.
2. Run A, B, B, A. Start each phase from a fresh deliberately started daemon, even
   for the repeated middle build. Warm up for 90 seconds, then observe for 300
   seconds at five-second intervals. Expected duration is about 26 minutes plus
   switching and verification. Log any startup audio outside the measured phase;
   require a quiet graph after warmup and throughout all 61 measurement points.
   Keep the editor closed and Deck untouched; moving
   the mouse to prevent screen lock is allowed.
3. Read daemon summed-thread CPU, service-cgroup CPU including exited-child work,
   companion CPU separately, RSS, cgroup memory, handles, restart counts and
   connected/display/lock state. Use the same method in every phase. Record the
   service/executable identity, exact warmup, sample count and elapsed time.
4. Require no input/action records or playback during quiet phases. Label and
   reject any invalid phase rather than blending it into a quiet mean.
5. Compare paired means and phase variation against the reference observations.
   Two phases per artifact do not prove statistical equivalence or establish a
   universal CPU guarantee. Investigate actual sustained increases together with
   memory, handles and responsiveness; do not explain them by assertion.
6. In success **or failure** finalization, restore B through the reviewed installer,
   verify it reconnects, compare saved-data/integration/startup hashes and return
   the original active page. Retain the recoverable backup and previous build.
   Evaluate restoration separately from the quiet-sample gate: external playback
   invalidates a measurement but does not by itself mean restoration failed.

## Stop conditions

Stop sampling on lost connection, a lock transition, failed build admission,
unexpected restart, unavailable helper, changed saved data/routing, playback,
unplanned input or an installer/readiness error. Preserve logs, restore B and
report the failed condition. Do not force-reset the host, change permissions,
retry uncertain actions or adjust audio to make a sample valid. If restoration
does not converge, stop further testing and use the retained recovery evidence.

This protocol does not measure loaded dispatch or visible pixels. Numerical
visible-response qualification was deferred beyond V1 on October 7; internal
dispatch/action coverage and the final publication decision remain separate.
