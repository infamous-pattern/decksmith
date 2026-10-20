# Decksmith pause and resume checkpoint — October 20, 2026

The maintainer paused active Decksmith work for about a week and requested this
checkpoint. Resume when the maintainer returns; no automatic restart date is set.
Existing server-native scheduled tests, failure monitoring and four-hour Reddit
feedback monitoring continue. Do not launch additional development tests, hand
off USB devices, install a candidate or publish a release merely because a
monitor wakes up.

## Resume here

1. Re-read this checkpoint, the [qualification history](2026-10-17-ubuntu-debian-support.md)
   and the [roadmap](../ROADMAP.md). Inspect current source, retained candidates,
   desktop/VM state and any monitor incidents before changing anything.
2. Revalidate the corrected signed Ubuntu candidate and repeat the physical
   Firefox-restart case in the desktop's Ubuntu VM. This is the first blocking
   acceptance item, not another broad feature implementation task.
3. Finish Ubuntu's remaining lifecycle, accessibility, delivery and resource
   gates below. Run safe unattended Debian package/GNOME repeats on the AI
   server in parallel once active testing resumes, preserving its baseline.
4. Arrange Debian's own physical acceptance separately. Ubuntu/Fedora hardware
   passes and Debian virtual-device checks do not establish Debian hardware support.
5. Review all evidence and remaining limits before any support/release promotion.
   After physical acceptance, return to the bounded editor-state trial and
   approved usability work; keep V2 ideas separate.

## Source, release and desktop boundary

- Public release remains **V1.5.1**, with Fedora 44 GNOME/Wayland as the supported
  setup. Ubuntu/Debian public previews have not been promoted to supported V1.5.2
  releases. Candidate signing does not publish a release or change installer pins.
- Last synchronized source before saving this checkpoint: canonical/Gitea
  `8eb5fdd648efe905a4d4c48c525fc6d47616cc20`, sanitized GitHub
  `b4f6c2efc91f3c3a3c9e82595115e550579c3e5d`, identical tree
  `8e3447c8bd4a68d8bcfea692b91a5cf2b60c9cbb`. Both remote heads were rechecked
  while saving this checkpoint. [Quality run 39427666159](https://github.com/infamous-pattern/decksmith/actions/runs/39427666159)
  completed successfully. Later commits since the Firefox correction are
  documentation-only; they do not create new signed runtime candidates.
- The desktop's current-build symlink was re-read as **1.5.2-46ed033e0491**.
  This is the Fedora Homebridge-assignment/cross-Deck-copy candidate already
  physically accepted; it does **not** contain the later Firefox correction.
- Last completed handoff restoration, October 19: both Fedora Decks and companion
  recovered, original audio defaults/volume/mute restored with Wave:3 muted,
  all 492 backed-up saved files unchanged, temporary audio USB access removed,
  original local VM definition restored and Ubuntu shut down normally. The
  maintainer confirmed normal Fedora pages and controls on both Decks.
- This checkpoint did not restart services or start a VM. Live user-bus/libvirt
  status was unavailable in the current restricted execution context; the
  preceding restoration states are dated evidence, not a fresh live-health claim.
  Recheck them through the appropriate user session before the next handoff.

## Corrected signed candidates ready for qualification

All three candidates were built from public source
`b16b5d04cc565c445fccff596a5ae5cde50e5188` (canonical
`c5d8cc67bc833b547973acaf2001e2fc49d90d1a`, tree
`daf0f1e4bd12d8bdd29f0a1c883fb43cecb182f1`). Their stored admission records
report verified signatures/provenance, checksums and archive validation. Retained
archive SHA-256 values were recomputed and matched those records on October 20.
They have not been installed on the daily desktop by the correction/build stage.

| Target | Corrected bundle | Successful build/signing run |
| --- | --- | --- |
| Ubuntu 26.04 | `1.5.2-c21a53271456` | [39321110741, attempt 2](https://github.com/infamous-pattern/decksmith/actions/runs/39321110741) |
| Debian 13 | `1.5.2-372bb4b4aed5` | [39321114430](https://github.com/infamous-pattern/decksmith/actions/runs/39321114430) |
| Fedora 44 | `1.5.2-9a8658523f12` | [39321118003](https://github.com/infamous-pattern/decksmith/actions/runs/39321118003) |

Private workspace evidence: `outputs/firefox-candidates-20261019/`, especially
`build-outcomes.json`, `tested-source-correspondence.json`, and each
`candidates/<target>/candidate-admission.json`, `accepted-hashes.json`,
`downloads/` and `signing/` directory. Preserve the retained files and recheck
their admission before use; an expiring hosted artifact is not the only copy.

The first Ubuntu build attempt hit an existing microphone-thread fixture's
two-second wait; the Firefox tests passed. The same fixture passed 100 isolated
parallel repetitions, and one unchanged full workflow retry passed. Its failure
log remains retained; do not erase it, call it a Firefox defect, or claim its
timing cause was established. No policy or timeout was weakened.

## Ubuntu — completed work and next physical retest

October 19 physical testing used **1.5.2-d4a17d906de7**, an earlier candidate,
in the local Ubuntu 26.04 VM with actual Plus, MK.2, Schiit Magni and Wave:3.
APT/Snap updates and reboot were verified first (GNOME 50.1; no pending updates
at that time). Check for new updates again when resuming; that dated result is
not a claim that the VM remains current next week.

Passed on that earlier candidate: independent pages and brightness; real output
and Firefox app volume/mute/meters; ordinary Firefox playback controls from both
Decks; microphone volume/meter and overlapping push-to-talk, last-release mute,
cross-device mute cancellation and page-change cancellation; physical emoji
label saves and key copying in both directions; explicit saved-brightness
readback (MK.2 40%, Plus 60%); Quit blanking; foreground-app/default pages; lock
blanking, ignored locked inputs and restored independent pages/brightness.

Both-direction USB/PTT recovery passed **with explicit VM USB redelivery** after
physical reconnection changed the host USB address. The partner kept working and
Decksmith was not restarted. Keep that transport assistance visible in the
acceptance record; it is not proof of unattended bare-metal hot-plug recovery.

The confirmed blocker was a saved explicit Firefox MPRIS target tied to an old
Snap `instance_<D-Bus numbers>` name. After closing/reopening Firefox, Plus's
explicit third-dial playback action stopped working while Automatic keys still
worked. The committed editor/native-adapter correction recognizes that family.
Source tests and a private helper trial passed; the corrected signed physical
test is still outstanding. Earlier JSON fields saying "prepared/not committed"
are historical capture state; the later build/admission receipt above supersedes
them only for source publication and signing, not for physical acceptance.

For the retest: install the exact corrected Ubuntu bundle in the backed-up VM;
select/save the explicit Firefox target through the native editor, verify the
physical Plus dial and MK.2 Automatic control, close Firefox fully, reopen the
local test media and compare both controls again. Exercise both an existing
saved instance choice and a freshly saved choice. Record old/new identities,
saved assignment, package identity and the maintainer's actual result. Preserve
unavailable/missing targets rather than borrowing a different player.

## Remaining qualification gates

| Gate | Ubuntu 26.04 | Debian 13 |
| --- | --- | --- |
| Corrected exact-package native qualification | Install/upgrade the retained corrected signed bundle; repeat changed media behavior and affected paired editor/runtime paths. Earlier package passes are not exact-bundle repeats. | Repeat package install/upgrade/rollback, paired editor/history/copy and media cases using `1.5.2-372bb4b4aed5` on the AI server. |
| Desktop integration | Earlier GNOME 50.1 app/lock checks passed; confirm affected paths with corrected bundle. | GNOME 48.7 compatibility fix passed on earlier signed `1.5.2-1272d3a4d717`: actual foreground/default pages, indicator start/stop/open/quit and source-loss recovery. Repeat relevant paths with the corrected bundle; do not restore obsolete Shell metadata workarounds. |
| Physical USB/audio/media/microphone | Firefox-restart retest first, then targeted regressions for both Decks, real audio, meters and microphone safety. Preserve VM-redelivery qualification limit. | Still needs independent two-Deck hardware/audio/media/microphone/editor acceptance; select a suitable physical test environment with the maintainer. |
| Lifecycle | Complete sign-out/in automatic controls with editor closed, persistence through restart, suspend/resume and applicable locked-reconnect/held-input recovery checks. Record VM/bare-metal coverage limits explicitly. | Independently qualify USB, lock, suspend/resume, login, quit/blanking and restart with physical hardware. |
| Resources and usability | Earlier five-minute VM samples: idle 2.89%, loaded 9.37% of one core, with no automatic restarts; loaded sample was five minutes, not the standard ten-minute acceptance. Repeat corrected-package clean idle and ten-minute interactive load, document streams/conditions; complete native keyboard/Orca, scroll/focus, long-label and window-size checks. | Native accessibility/editor and physical idle/loaded measurements remain independent gates; do not transfer Fedora/Ubuntu numbers. |
| Delivery and recovery | Audit carried-forward fresh/missing-dependency and preview-upgrade evidence against the corrected bundle; close pristine-OS/public-delivery, USB-permission and complete-backup recovery gaps. | Same per-OS exact-artifact delivery/dependency/permission/recovery review; retain original configuration, modes, integration and VM baseline. |
| Promotion | Not approved. Assemble final Ubuntu evidence/limits and obtain the release/support decision separately. | Not approved. Keep Debian preview status until its own evidence/limits and support decision are complete. |

Private primary evidence: `outputs/ubuntu-physical-20261019/physical-acceptance.json`,
`firefox-target-after.json`, `resource-idle.json`, `resource-loaded.json`,
`host-restoration-verification.json`, `host-saved-data-comparison.json` and
`guest-final-power-state.json`. Debian's prior successful native package/GNOME
receipt is `outputs/debian-gnome48-signed-20261017/completion.json`; it explicitly
records no physical HID or real-audio acceptance. Preserve failed/superseded
fixtures alongside successful receipts rather than inferring state from a folder
name alone.

## Safe handoff and later work

Before any resumed physical trial, confirm saved/closed editor state, inspect the
current host and guest, retain fresh verified complete configuration/integration
and VM recovery copies, and arrange the temporary interruption of Fedora controls
and real audio with the maintainer. The previous local Ubuntu helper set and
dated backup references are in `outputs/ubuntu-physical-20261019/`; inspect them
before reuse. Temporary USB addresses/permissions and old test process state are
not reusable assumptions. Finish by restoring original Fedora build/layouts,
audio defaults/volume/mute, permissions and VM definition, checking both Decks
with the maintainer, and gracefully stopping the guest.

After acceptance: bounded TCA-inspired editor-state trial, then scoped device/
target/draft/applied/disconnected feedback and the existing usability queue.
Boatswain/metric-aware design recommendations are recorded in the
[toolbox](../research-toolbox.md#boatswain-and-metric-aware-design--references-adopted-october-20).
Linked navigation, multi-actions, OBS, soundboard and interactive virtual Decks
retain their later/V2 boundaries. The XERJ CLI trial is complete; it remains
optional/on demand, with its trial services stopped and no runtime integration.

This is a saved resume record, not authorization to start tests during the pause,
deploy a new desktop build, publish V1.5.2 or promote another supported OS.
