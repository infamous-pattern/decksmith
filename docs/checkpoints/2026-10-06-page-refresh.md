# Page refresh scheduling — 2026-10-06

Source `d1a8c740db3bf845019885a7172f84533853778a`, development runtime
`0.1.0-1d2c24f8616a`, removes the ordinary 20 ms input wait between pending
display writes. The original physical video exposed approximately 170 ms of
sequential key repaint. Code inspection found the scheduler writing one slot and
then waiting for input before proceeding to another pending slot.

The worker still writes at most one image per iteration and checks lock/control
state and ready input between writes. A zero-timeout physical poll is used only
while more images remain pending; the 20 ms wait resumes once they settle. Both
read modes share normalization, report receipt stamps and error handling. Queue
limits, meter cadence, input gating on incomplete pages, and held-input cancellation
are unchanged. There are no added threads, caches or persistent image buffers.

Validation passed:

- 138 Rust tests, with three existing diagnostics ignored; formatting and
  warnings-denied all-target/all-feature Clippy.
- Pinned dependency audit: advisories, bans, licenses and sources.
- Six current regression cases in Fedora Workstation 44: pending repaint/input
  fairness, restored idle timeout, queued receipt stamps, lock/drain behavior,
  held push-to-talk release, and failed-write session invalidation.
- Normal installer and runtime doctor on Fedora 44, followed by 30 page changes
  on a private VirtualDeck bus with isolated settings. All pages settled, layout
  stayed unchanged, and shutdown completed. Command-to-status median was 1.66 ms,
  maximum 4.24 ms; this is a VM/control-path observation without physical USB.

After the user confirmed saved edits, the desktop installer retained its previous
runtime and a configuration backup, then activated the candidate. The physical
Deck reconnected with display ready, brightness preserved, and Auto-Lock available
and enabled. Configuration hashes matched. The user confirmed faster page changes
and correct display/controls while testing swipes, an audio dial and mute.

A 61.2-second read-only journal capture contained seven physical page requests,
20 display-settled records and two successful uninstrumented plugin results. It
contained no built-in audio action timing; audio acceptance here is the user's
functional confirmation. The daemon PID and zero automatic-restart count stayed
unchanged, the layout hash matched, and no warning/error records were captured.

For the seven page requests, journal request emission to matching display-settled
emission measured median **14.158 ms**, maximum **21.690 ms**. The previous video
window's four requests measured median 159.657 ms, maximum 330.053 ms at that same
journal boundary. Pages, content and audio conditions differ, so these are
descriptive observations, not a controlled benchmark or percentage speedup.
They measure logged write completion, excluding input-to-request delay and LCD
pixel response; they do not close the physical visible-latency acceptance gate.

All four test VMs were verified shut off afterward. Raw evidence remains in the
ignored local `page-refresh-20261006` directory. This development bundle is not a
frozen V1 release: its manifest marks source dirty because pre-existing untracked
`output/` and `tmp/` directories remain, while the tracked source was committed
before building. Those unrelated files were preserved and not staged.

The earlier experimental Homebridge busy/failed results remain separate. Source
inspection confirms busy rejects an incompatible request while another plugin
job is active, and failed represents a negative bridge response. The underlying
remote failure is not diagnosed by those codes. No Homebridge commands were sent
during this investigation.
