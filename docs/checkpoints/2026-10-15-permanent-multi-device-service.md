# Permanent saved-device desktop service — October 15, 2026

This is a local V1.5 development qualification checkpoint. It is not publication
of V1.5 or an expansion of advertised OS/device support.

## Implementation

The normal background unit uses a persistent multi-device coordinator rather than
a timed physical lab. Unique supported serial/model identities enroll with stable
private IDs. The first uniquely identified Plus imports the exact existing layout
with V2 metadata added; original legacy bytes and root configuration remain
available for rollback. MK.2 receives its own 15-key Home, Media and System pages.
Pages, brightness, selected editor device and profile revisions stay independent.

The normal application launcher opens one simplified editor with a device chooser,
15-key/no-dial MK.2 preview, independent drafts, device-addressed saves and safe
saved-action tests. Closing the editor leaves background controls running.
Start-at-login and stop/restart/quit continue to use the same installed unit.

Native audio, media, named microphone, app/web/system and experimental Homebridge
assignments use shared bounded workers with separate device input/display owners.
Unavailable targets remain unavailable. Media selection is resolved per action;
service startup and editing do not depend on an active browser player. Homebridge
uses a matching versioned companion with a late authorization fence; uncertain
writes are not replayed automatically.

Lock and sleep invalidate stale input, pending edits and display work. Sleep
preparation requires actual microphone mute/exit acknowledgements and quiet effect
lanes before acknowledging logind. Identity-safe reconnection waits for old I/O to
retire; an unavailable partner does not globally block a healthy device. Shutdown
stops new profile requests, fences/mutes/blanks devices, then joins profile work.

## Qualification and activation

The immutable source snapshot `af7ae26c68bf28425f8c7958b0dd36f68c003d6798b919223b34bcc4be2e96e4`
passed 331 Rust tests (five intentional ignores), 235 Studio Python tests and 114
script tests, strict Clippy/formatting, native old/new profile editors, private lock
proxy, optimized-build self-checks and VirtualDeck. The permanent native fixture
verified independent drafts, saves and brightness, selected-device compatibility
previews, previous-layout reads, Test Action, editor-close survival and pages/
brightness after restart. A valid pending save returned `profile_request_cancelled`
on shutdown with its active store pointer unchanged and a clean bounded exit.

The packaged optimized binary separately passed migration/restart using a private
copy of the actual daily configuration. The Plus document matched the original
values with only V2 model/version metadata added; original bytes were retained.
No physical device or native audio was used in these isolated checks.

Installed locally as `1.0.0-dev-paired-c3dc9ec9e167`, package SHA-256
`1e204401ae9eefca4004af0c4b2327a30673d563b26a03016ef762a44d3ff2c0`.
The matching Homebridge companion is `8dc50ffed3a869d8`. Both installed services
and both connected models passed repeated machine readiness checks; device errors
were null, brightness was retained (Plus 55%, MK.2 50%), startup remained enabled,
and exact Plus migration/retained legacy bytes were verified.

Fedora auto-locked during qualification. The first two guarded installations
restored V1 because their verifier incorrectly required input readiness while
locked. The verifier was corrected, without changing runtime source, to require
confirmed current available/enabled/locked state, completed displays and disabled
input while locked; unlocked state still requires ready input. The third guarded
installation passed. This verifies rollback of both app and companion in practice;
it does not substitute for the user's visible lock/unlock acceptance.

Evidence under `outputs/v15-permanent-20261015`: `qualified/final`, `package-result.json`,
`packaged-migration/result.json`, `desktop-install-03/result.json`, and
`post-install-state.json`. Configuration backup before final activation:
`~/.local/state/decksmith/backups/20261015-113256-386101.tar.gz`; the installed
release and integration backups also retain the previous UI/V1 daemon and old
companion. These local paths are evidence locations, not public download links.

A 45-second observational sample with Fedora locked found no missing service
processes: daemon 4.375% of one CPU core, combined sampled app/companion/helper
processes 5.396%, summed RSS 130.69 MiB. This short locked, uncontrolled sample
is not the release idle/loaded resource qualification or a comparable performance
claim against V1.

**Initial physical acceptance passed:** the user unlocked Fedora and confirmed
the normal Plus page, independent MK.2 Home/navigation, Plus swipe and audio dial
worked on both devices. A contemporaneous read confirmed both connected models
with ready input/displays and no device errors after unlocking. Evidence:
`physical-initial-acceptance.json`.

**Existing Homebridge acceptance passed:** the user confirmed the saved toggle
keys and ordinary-speed brightness dial, followed by a normal Plus audio control
and MK.2 navigation, with both Decks remaining available. Both sessions still
reported connected, ready input/displays and no device errors; the companion was
fresh and ready. Evidence: `physical-homebridge-acceptance.json`.

**Permanent lock acceptance passed:** the user confirmed both Locked displays,
blank remaining keys and Plus touch strip, then independent working page recovery
on unlock. A contemporaneous read confirmed both sessions fully ready and without
device errors while unlocked. Evidence: `physical-permanent-lock-acceptance.json`.

**Remaining acceptance:** expanded MK.2 assignments and
permanent-service lifecycle, accessibility and controlled resource tests. The
initial smoke check does not qualify these separate release gates.

## Development limits

- Offline devices can be viewed and drafts retained; reconnect before saving.
- Initial enrollment runs at service startup. Known saved devices reconnect
  automatically; a newly added device needs a background-service restart to enroll.
- A save committed at the same moment as a lifecycle fence can require an explicit
  background-service restart. The editor says whether the layout was saved; it
  does not silently retry or discard the draft.
- Ambiguous or missing physical identity is never guessed; multiple brand-new
  Plus devices do not compete for the legacy layout import.
- A V1 rollback reads the unchanged legacy layout. Subsequent per-device V1.5 edits
  remain stored for recovery; they are not silently merged into the V1 layout.
- Microphone assignments require an explicitly named input; the old generic
  `microphone` alias remains unavailable.
- General plugins remain deferred to V2. Existing Homebridge is experimental,
  with its own physical acceptance and uncertainty-recovery limitations.
- Release qualification still needs paired permanent lifecycle/resource/security
  and installer evidence; this checkpoint alone does not declare V1.5 ready.

**Suspend deferred by the user:** on October 15 the user chose “Test suspend
later.” No permanent-build physical suspend result is claimed. Keep it as an
open release gate; leave the installed service running normally.

## AI-server follow-up during the interview

The exact packaged development build and immutable qualified source were tested
without touching desktop services, real audio/HID, physical power operations or
scheduled jobs. Evidence is under `outputs/v15-permanent-20261015/server-followup`.

- Five-minute synthetic manager stress: eight alternating virtual Plus/MK.2
  devices, 137,942 reconnect cycles with reversed discovery, holds and stale
  feedback rejection. Separately, 280 repeated paired runtime fault tests,
  60 private fake-logind tests and 40 saved-meter/page/reconnect tests passed.
  These are synthetic tests, not eight physical devices or an actual suspend.
- Packaged API endurance: 160 owner-addressed layout saves, 80 brightness saves,
  80 selections and three service restarts passed. Both profiles stayed separate,
  stale pre-restart tokens were rejected without disk writes, active history stayed
  bounded (10 generations at the end), and 78 history archives were present.
  The original fixture archive counter used a wrong prefix; the corrected
  post-run inspection and original result are both retained.
- Seven artifact checks and 25 offline companion protocol tests passed. All
  479 package-manifest files verified; 477 packaged source files matched the
  immutable source, and stripped daemon/CLI bytes matched the qualified outputs.
  Relocation included a path with spaces. Doctor used a hardware-enumeration-only
  stub, so this does not certify guest/physical HID access. Primary OSV queries
  found no matches for 135 app/websockets versions.

Two follow-ups were identified/confirmed:

1. After more than 64 sibling history archives, when the previous differing
   layout is no longer in active generations, Previous Layout returns
   `profile_archive_review_required`. A copied endurance store reproduced this
   with 84 archives. Current layouts/sessions remained healthy, shutdown was
   clean, and older layout files were present with hashes matching their
   generation manifests. Review bounded history indexing/recovery; do not delete
   archives or silently present this as successful previous-layout restoration.
2. The historical receipt-bound companion has advisory-affected lock versions:
   compiled `rustls 0.23.45` (fixed `0.23.46`) and optional `quinn-proto 0.11.16`
   (matches fixed by `0.11.18`). Quinn was not shown compiled/reachable; the
   Rustls exploit-specific conditions were not demonstrated in the reviewed
   ordinary reqwest client. This remains a dependency finding, not a clean audit
   or proof of an exploitable running setup. The old unstripped companion also
   retains a builder home path. Prepare a separate copied-source, updated,
   path-remapped/stripped companion with a new receipt and offline/live acceptance
   before replacing or distributing it. No accepted binary was modified in place.

Fedora 44 and Fedora 45 package/native checks passed: dependency verification,
installation, no-device waiting editor, independent virtual two-device editing and
saves, restart persistence, rollback and reinstall. Both guest baselines were
restored and every AI-server VM is shut off. Fedora 45 hit systemd’s rapid-start
limit while restoring its original service after repeated test cycles; a separate
baseline-only boot verified all 5,158 original regular files byte-for-byte, the
original release/startup preference, and its original service active with success
and zero restarts. Original failed cleanup evidence is retained. No security or
schedule setting was relaxed.

Emoji labels survived the native editor and API unchanged, but some emoji rendered
as question marks in VM device-preview images. Review font coverage separately;
this is not a crash or label-data loss. VM checks do not qualify physical USB,
audio or suspend. Final VM evidence is `server-followup/final/summary.json`; the
combined report is `server-followup/server-testing-summary.json`. Physical suspend
remains deferred by the user. No implementation changes or publication occurred.

The additional feature matrix passed 308 no-default-feature workspace tests
(zero failures, four intentional ignores). The first runner's two Unix-socket
fixture failures passed when the private temporary path was shortened; original
logs remain. Plain default/no-hardware binary `cargo check` has 14 conditional-
compilation errors; the identical diagnostics were reproduced on pre-change
HEAD `517d76` from verified archive `e55a42c393ed415c5588389f366650fb100cf8e1e8b4e1584c93b36becaa8b7e`.
This is a pre-existing developer-build configuration gap, not a regression in the
installed hardware-enabled candidate. Its module boundaries still need cleanup;
do not summarize the full feature matrix as entirely passing.
