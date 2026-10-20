# Decksmith remaining roadmap

This is the implementation roadmap, updated October 20, 2026. The v0.4 product
and architecture documents remain the broader design baseline. Items below are
planned work unless explicitly marked implemented; this is not authorization to
change system routing.

## Development toolbox — Tuesday, October 20

**XERJ: optional bounded development-search trial.** The maintainer requested an
assessment of [XERJ](https://github.com/xerj-org/xerj) for the development process
or project. The [toolbox assessment](research-toolbox.md#xerj--development-search-assessment-october-20)
records the inspected main/release pins, useful code/document/evidence retrieval
questions and the controls to validate before use. Begin CLI-only on a curated
AI-server source copy with a disposable local index, then compare ten real
questions against targeted `rg`/file reads. Measure useful results, freshness,
lookup effort and resource/indexing cost before deciding on agent integration.
Keep automatic configuration changes, memory writes, external feedback and
hosted processing off; source and reference licenses remain explicit. This is
planned development tooling, with no runtime or release dependency. It does not
delay corrected Ubuntu physical acceptance or the later editor-state trial.
The approved CLI-only AI-server trial is complete: expected-file retrieval was
8/10 against 7/10 for targeted text search; latency favored the simpler baseline.
Changed/deleted-file freshness and read-only access checks passed. Initial
indexing reached the 2 GiB cgroup ceiling; a separate short settled-node check
averaged 0.15% of one core and about 45 MiB RSS. Keep XERJ optional and on demand,
with no default agent integration or application dependency. Both trial services
are stopped; the desktop, guests and schedules are unchanged. See the toolbox
for measurements, corpus pins, limitations and the retained harness correction.

## Current work — Monday, October 19

**Physical acceptance remains first.** The October 19 updated Ubuntu VM trial
passed paired physical controls, native editor/copying, real output/app/microphone
checks, GNOME pages/lock and USB redelivery recovery. It exposed an explicit
Firefox player choice that stops resolving after browser restart; Automatic
controls still work. A bounded editor/native-adapter correction passes local
media tests and CI-toolchain Rust tests/linting on the AI server. The maintainer
approved publishing the reviewed source and building/verifying new signed
candidates; corrected-package physical retest remains required.
Do not promote Ubuntu support yet. Fedora's original build, both Decks, exact
audio settings and all 492 backed-up saved files were restored/verified, and the
VM shut down normally. Debian validation remains independent.
The October 17 corrected signed Debian candidate `1.5.2-1272d3a4d717` passed native
package/editor/recovery and actual GNOME 48.7 integration without guest metadata
changes; Ubuntu's earlier signed candidate passed actual GNOME 50.1 integration.
These results do not replace USB, real audio/media/microphone or lifecycle and
resource acceptance. See the [qualification checkpoint](checkpoints/2026-10-17-ubuntu-debian-support.md).

**After physical acceptance: bounded editor-state trial.** The maintainer approved
[TCA as a research reference](research-toolbox.md#the-composable-architecture-tca--reference-adopted-october-19)
on October 19. Trial explicit state/actions and isolated effects in one existing
assignment/Save and Apply flow. Keep each device's identity, draft, validation,
saving/error and connection state explicit. Test switching devices while discovery
or preview is pending, retaining a draft on failed save, and rejecting obsolete
responses through disconnect/reconnect. Use controlled transports/responses to
exercise complete sequences; retain current daemon ownership/context safeguards,
accepted UI, undo/redo and saved formats. Compare complexity and meaningful test
coverage before extending the approach. This adds no Swift dependency or whole-app
rewrite, and does not delay physical acceptance or authorize deployment.

**Reference refresh and follow-up plan approved October 19.** The
[21-repository review](checkpoints/2026-10-19-reference-refresh.md) includes
Deckhand v0.8.1 and 53 commits beyond the prior pin. The maintainer approved adding
the recommendations below to the project plan. They follow physical acceptance;
V2 experiments and conditional maintenance retain their separate scope. Existing
Copy/Paste and discovery/rendering work are implemented, not proposed again.

| Order / scope | Planned work | Completion evidence |
| --- | --- | --- |
| First after physical acceptance | Run the bounded TCA-inspired editor-state trial described above. | Controlled out-of-order replies, device switches, failed saves and reconnects retain the right draft/owner; compare complexity before extending the model. |
| Next usability improvement | Complete scoped failure feedback across the permanent runtime, physical controls and selected-control inspector. Extend existing feedback rather than adding a diagnostics dashboard. | Errors identify device/page/control and a practical remedy; success or reassignment clears the marker; reconnection does not conceal it; optional repeated notifications coalesce. No automatic retry of uncertain side effects. |
| Small follow-up | Add a manual update check to About; assess an opt-in check when the editor opens. Keep installed build, OS/architecture and stable/candidate channel explicit. | Correct signed release destination, bounded/offline/error states, no daemon polling or automatic installation; preserve signature verification and complete-backup recovery. |
| Alongside editor work | Extend tests protecting custom captions/artwork from late provider updates and checking artwork inputs at the import boundary. | Saved user choices survive live refreshes; invalid or unsupported inputs fail clearly without changing another control or losing a draft. Treat upstream cases as test ideas, not confirmed Decksmith defects. |
| Throughout interface work | Apply refreshed Impeccable native review questions to actual GTK windows. | Clear scroll markers and reachable Save and Apply; long labels, focus, contrast, unavailable/recovery states, keyboard/Orca and light/dark themes checked without increasing interface density. |
| Integration qualification; vendor maintenance conditional | Add initially empty Homebridge discovery/recovery, brightness-range and correct-target scenarios. Review Govee updates only after checking the actual installed plugin/model/version. | Temporary absence recovers without borrowed targets or lost assignments; no vendor scaling added blindly to Decksmith. Live bridge updates remain a separate maintenance decision. |
| Optional bounded development trial | Evaluate current RepoWise on a clean source copy before expanding the editor refactor. | Findings checked against real Python/GTK/GI/D-Bus and Rust paths; record useful results, misses, cost and license/dependency implications. No automatic hooks, transcript ingestion or hosted analysis. |
| V2 exploration | Design and trial an interactive local virtual Deck, distinct from the existing test backend. | Explicit pairing/permissions, sensible default geometry, GNOME focus behavior, display/touch removal and reconnection, lock handling and push-to-talk cancellation accepted independently. No implied browser/phone remote control. |
| V2 plugin work | Define catalogue admission checks and declarative settings before integrating third-party plugins. | Validate identity, manifests, compatible versions, permissions, dependencies, artifacts/assets and licenses; do not treat catalogue membership as trust or compatibility. |
| Optional later visual work | Select new Tabler monitor/media/status icons when needed; explore stereo/spectrum meter styles separately. | Preserve existing artwork identity and notices and keep current simple gauges as default; compare measured CPU/render cost and avoid unverified calibrated-meter claims. |

Planning approval does not mark these items implemented or qualify upstream tools.
Keep the native Rust/GTK architecture, independent default profiles and existing
release/security gates. Installation, deployment and publication follow the
applicable acceptance and approval process.

## Current work — Saturday, October 17

**Desktop editor correction accepted on Fedora.** The MK.2 Homebridge flow now
has a visible assignment step and clears obsolete action settings. A window-wide
key clipboard supports copying between Plus and MK.2 with safe page-name resolution.
All 244 editor tests and private native integration checks passed. Signed Fedora
candidate `1.5.2-46ed033e0491` was verified and installed with a verified complete
backup and V1.5.1 retained. The maintainer confirmed real MK.2 Homebridge saves/actions
and copies in both directions; the saved reverse copy changed MK.2 while preserving
Plus. Both services remain healthy without automatic restarts. The first forward
copy's saved observations were inconclusive, separately from its user-confirmed
physical success. See the
[assignment/copy checkpoint](checkpoints/2026-10-17-key-assignment-copy.md).

**Ubuntu support first; Debian validation in parallel.** The maintainer approved
this order on October 17. Qualify Ubuntu 26.04 x86_64 GNOME/Wayland against the
current V1.5.2 candidate source and keep Debian 13 validation as a separate target on the
AI server. Initial signed candidates stopped at new upstream maintenance notices;
the zbus 6 migration and bounded V1.0-preview permission migration now have verified
signed V1.5.2 Fedora, Ubuntu and Debian candidates. The installer passed 122 script
tests; the exact Ubuntu and Debian packages passed native upgrade, paired editor,
service and recovery checks. Additional parallel VM runs passed clean per-user
installation, controlled missing-dependency repair, and complete paired-store,
history and imported-artwork recovery; both original guest setups were restored
and powered off. The cancellation test fixture correction passed GitHub Quality
run 99. Pristine-OS/public installer delivery, GNOME integration and physical
Ubuntu acceptance remain open; the maintainer deferred the hardware stage.
Updated signed Ubuntu/Debian candidates including the editor corrections now
passed exact-package native upgrade, service, paired editor/restart, Homebridge
fixture save and bidirectional copy/undo/redo checks. Both guest baselines and
permissions were restored and guests shut down. One transient sleep-test snapshot
assertion failed in the first Quality attempt; the same-source retry and 300
isolated repeats passed. Retain its deterministic-fixture follow-up separately. No release or support promotion
is approved. Both retained server VMs passed backup/update/reboot and existing-preview
smoke checks. Existing previews remain experimental until their own acceptance and
publication decisions. Do not infer support from a build, VM smoke test or Fedora
physical evidence. See the [Ubuntu/Debian support qualification](checkpoints/2026-10-17-ubuntu-debian-support.md).

## Published V1.5 checkpoint — Friday, October 16

Signed candidate `1.5.0-8c0783ad4bf8` is installed on the daily desktop and has
completed its recorded physical controls/lock/USB/resource checks plus exact-package
Fedora 44 and diagnostic Fedora 45 VM checks. Source and signatures are verified;
all ten server guests passed the storage migration checks and old duplicate copies
were removed. Final review distinguishes exact-candidate repeats from earlier
lifecycle/editor/accessibility/microphone acceptance carried forward on unchanged
runtime source. The maintainer approved publication with documented limits on October 16.
**V1.5 is now published**, with all seven fresh public downloads verified before
Fedora installer/README promotion. V1 and experimental previews remain available.
See the [final V1.5 release review](v1.5-release-review.md).

**Before announcement — starter defaults update (V1.5.1).** The maintainer
requested Maker’s Mark as the installation default with Sans/Small/White labels,
transparent key and label backgrounds, Bottom labels and 85% icons. Each supported
model’s Home page gets a bottom-left Maker’s Mark link to the Decksmith GitHub
repository. Existing saved profiles stay intact. V1.5.1 is published after
signed-package, native and targeted daily-desktop checks,
with fresh public downloads verified before Fedora installer/README promotion. See the
[starter defaults checkpoint](checkpoints/2026-10-16-starter-defaults.md).

**Linked navigation — explicitly deferred after V1.5 on October 16.** Future
opt-in page pairs could let navigation keys or Plus swipes change a workspace on
both devices and foreground-app rules select paired pages. Independent pages
remain the V1.5 behavior. Pairing, disconnected-device catch-up and interaction
with per-device application rules require separate design and qualification.

The October 15 record below preserves the implementation and qualification history.

**Release scope decision — October 15:** target Friday, October 16, end of day
Eastern for a **tested multi-device V1.5**, conditional on the gates below passing.
The user chose this focused release over retaining the full feature list. Include
simultaneous Plus/15-key MK.2 support, the accepted simplified native editor and
gauges, and preservation of existing saved layouts, assignments and controls.
Unfinished gestures, Dial Stacks/Action Wheel, Now Playing, wallpaper and sticky
keys remain follow-up work; they must not delay or be advertised in this release.
Broader OS support remains independently qualified; existing previews do not
become supported platforms through this deadline.

**Plugins move to V2 by user decision on October 15.** General plugin hosting,
management, compatibility/catalog work and the plugin-provider system-monitor
track are deferred together. Preserve existing experimental Homebridge bindings
and settings; explicitly verify or document their paired-runtime behavior before
promotion. Deferral must not silently erase or substitute saved actions.

**Daily desktop builds — requested October 15:** keep the user's desktop on the
latest qualified build for real-time feedback. Preserve the previous package and
configuration backup, protect unsaved editor work, verify service/device recovery
after installation and document remaining limits. A timed development lab or a
passing build alone is not a production update. Do not install an incomplete
paired runtime over working daily controls merely to advance the revision.

Release gates: built-in app/web/system/audio/media behavior; independent saved
profiles and migration/rollback; lock, suspend, hot-plug, quit/stop blanking and
start-at-login for both devices; native editor/device parity and accessibility;
bounded shared subscriptions/queues plus idle/loaded CPU and memory; fresh
installer and artifact/security verification. Record exact build/evidence and
remaining limits. A missed gate changes the date or scope explicitly, not the
reported outcome.

1. **Simplified native editor — installed and user accepted.** On October 15,
   the user confirmed stable key/dial editing, reachable expanded controls and
   clear scroll markers, plus the normal physical Plus page and controls after
   installation. The later permanent multi-device development update now runs
   both Decks; desktop keyboard/Orca acceptance is recorded on the recovery build.
   See the [interface checkpoint](checkpoints/2026-10-15-simplified-native-editor.md).
2. **Resume multi-device lifecycle integration and qualification.** Saved meters,
   shared native microphone policy, System sounds meter isolation and held-device
   USB recovery passed their bounded checks on October 14. The opt-in paired
   lock slice passed physical Plus/MK.2 lock, locked-input rejection and locked
   reconnect checks on October 15, with normal controls restored afterward.
   Persistent startup, identity-safe reopen, the normal saved-device editor and
   provider integration are now implemented and installed as a local development
   build. Exact Plus migration, independent MK.2 defaults, restart persistence,
   native editor and rollback passed automated checks. Physical acceptance of
   this permanent build and controlled resource/lifecycle qualification remain.
   See the [permanent-service checkpoint](checkpoints/2026-10-15-permanent-multi-device-service.md)
   and the
   [latest microphone/recovery checkpoint](checkpoints/2026-10-14-native-shared-microphones.md).
   The [October 15 lock/action checkpoint](checkpoints/2026-10-15-paired-lock-and-builtins.md)
   records the accepted physical lock slice and guarded built-ins. Built-in native
   action acceptance and permanent-build suspend remain separate gates; local
   development activation is not a V1.5 release declaration.

**AI-server follow-up — October 15:** extended synthetic reconnect/lifecycle,
packaged two-device save/restart endurance, artifact and offline companion tests
passed. Two items need review before their corresponding release gates close:
bounded Previous Layout recovery beyond 64 history archives, and a separate
receipt-backed rebuild of the historical Homebridge companion with updated
advisory-affected dependencies and remapped builder paths. No current exploit
path was demonstrated; the app dependency query had no matches. Keep desktop
controls unchanged during the user's interview; physical suspend remains deferred.
See the [permanent-service checkpoint](checkpoints/2026-10-15-permanent-multi-device-service.md)
for evidence, retained data and limits. Fedora 44/45 package and native two-device
editor checks also passed, with original guest state restored and all VMs off.
VM preview emoji font coverage and a pre-existing default/no-hardware build
configuration gap remain documented follow-ups; neither caused saved-label loss
or a regression in the installed hardware-enabled build.

**Recovery/dependency fixes installed — October 15:** both server findings now
have tested development builds. The profile store retains one verified predecessor per
device; a copy of the exact 84-archive failure store passed recovery, migration,
restore/undo and restart without changing its backups. A separately rebuilt,
path-remapped/stripped Homebridge companion passed 46 current integration/protocol
tests and updated-version advisory queries. The guarded desktop update passed
with a verified whole-store backup, and the user accepted live Homebridge/native
controls on both Decks. No public release was published. Pair any downgrade with
its matching complete device-store backup because the new manifest is version 2;
a binary-only downgrade or merge restore is insufficient. Next qualify the
remaining paired lifecycle/editor/resource gates. Updated-build physical
suspend/resume also passed, with both sessions ready and without errors afterward;
bidirectional USB isolation/reconnection also passed, with the connected partner
remaining ready and each removed device recovering. Both-device quit blanking
and normal relaunch also passed with clean service exits and restored controls.
Start-at-login acceptance also passed: both Decks started automatically while the
editor stayed closed, and post-login checks found both sessions ready. Next finish
permanent editor/accessibility and controlled resources. See the
[recovery and companion checkpoint](checkpoints/2026-10-15-layout-recovery-and-companion.md).

**Permanent editor follow-up — October 15:** keyboard navigation and Orca were
user-accepted for both device views on the recovery build. Paired physical label
save/Restore previous checks passed on the installed emoji follow-up build,
including correct checkmark/music-symbol rendering, unchanged partner layouts,
retained actions/artwork and exact original-layout cleanup. The caption renderer
now bundles a monochrome fallback; full-color/compound emoji shaping remains
outside this fix. The icon-only Layout options menu below the device preview
proved hard to discover and needs a clearer visible label in later UI cleanup.
Controlled quiet/loaded resources and final release qualification remain. See the
[emoji and paired-editor checkpoint](checkpoints/2026-10-15-emoji-caption-fallback.md).

**Paired resource observation — October 15:** a five-minute paused-desktop
sample completed with both devices stable, no restarts or layout changes, and
steady 51.34 MiB daemon RSS. Daemon CPU averaged 5.737% of one core; both services
with helpers/companion averaged 8.073%. Two lingering speech-dispatcher playback
streams were documented with user acceptance, so this is not clean-idle evidence
or a matched V1 comparison. The later loaded interaction and clean-idle observations are recorded below;
resource regression review and final release gates remain open. See the
[resource checkpoint](checkpoints/2026-10-15-paired-resources.md).

**Paired loaded interaction — October 15:** the installed build completed a
ten-minute saved-layout test with both physical Decks; the user accepted ordinary/
quicker dial turns, page changes and audio/media controls with correct displays.
Both sessions and process identities stayed stable, no services restarted, and
saved layouts were unchanged. Daemon CPU averaged 5.205% of one core (7.905%
combined services); daemon RSS stayed at 51.59 MiB. Brave was active throughout
sampled points, with additional Chromium/speech streams documented. This closes
that finite loaded interaction check, not clean-idle/regression review or final
release qualification. See the [resource checkpoint](checkpoints/2026-10-15-paired-resources.md).

**Development package review — October 15:** exact installed app/companion
integrity, isolated package provenance/ELF hardening and paired staged backup
restore passed. The exact-version advisory query returned no matches across 277
distinct versions. Latest-package installer/native two-device checks passed in
Fedora 44/45 beta test clones; original disks/definitions were unchanged and all
server guests are off. Fedora45 baseline cleanup hit a start limit; fresh-boot
file/autostart verification and graceful ACPI shutdown resolved the fixture issue,
with original failures retained. No runtime code or desktop service change was
made during those package checks. The subsequently authorized clean-idle
measurement completed, with speech services restored afterward. Server space is tight
(about 19 GiB free, 227.81 GiB retained test backups); full-copy jobs need capacity
review. Final source/version/signing/publication remain separate. See the
[review and evidence limits](checkpoints/2026-10-15-release-review.md).

**Clean paired idle — October 15:** after an authorized temporary speech-service/
socket pause, 300.001 seconds and 61 points completed with no playback streams,
stable devices/services and unchanged layouts. Speech running/enablement states
were restored and accessibility settings were unchanged. Daemon CPU averaged
5.757% of one core (8.005% combined services), with flat 51.59 MiB daemon RSS.
The below-1% milestone is not met. Clean-idle collection is complete; resource
regression review against the differently configured historical V1 reference
remains explicit, alongside final source/version/signing/publication gates. See
the [resource checkpoint](checkpoints/2026-10-15-paired-resources.md).

**Discovery optimization — October 15:** the development update
`1.0.0-dev-discovery-d5e84a632d77` is installed after 341 Rust tests, strict
Clippy and isolated AI-server package/paired-backup checks. The discovery worker
uses a bounded Linux topology cache with 30-second full refresh, immediate
change/error fallback and unchanged per-open identity checks. Normal physical
controls and bidirectional USB isolation/recovery passed on both Decks. The
new clean five-minute sample averaged 6.181% combined CPU of one core versus
8.005% before (22.79% lower), with flat 42.61 MiB daemon RSS, stable services
and unchanged layouts. Speech services were restored. The combined 5% target
is not met; the main coordinator remains the largest measured daemon cost.
No coordinator polling change was made. The sequential comparison has different
service ages/history and is not historical V1 regression equivalence. See the [discovery checkpoint](checkpoints/2026-10-15-hardware-discovery-optimization.md).

**Coordinator optimization — October 15:** development package
`1.0.0-dev-coordinator-42ddae185374` is installed after 342 regression tests,
strict Clippy and isolated package/paired-backup checks. Unchanged authority
snapshots no longer rebuild the map and wake action providers. Input polling,
explicit revocation, microphone maintenance and the hardware-discovery cache
are retained. Physical controls/Homebridge and bidirectional USB recovery passed.
The clean five-minute sample averaged 2.497% combined CPU of one core
(0.897% daemon), 59.60% lower than the preceding discovery build;
daemon RSS averaged 45.38 MiB. Both services/devices/layouts stayed stable
and the speech pause was restored. The combined 5% target was met in this
finite quiet sample; this is not a workload-independent guarantee. See the
[coordinator checkpoint](checkpoints/2026-10-15-coordinator-optimization.md).

**Optimized loaded qualification — October 15:** the same coordinator package
completed ten minutes/121 points with Brave active throughout and the user
accepting both physical Decks' pages, audio/media keys, ordinary/quicker dials
and correct labels/icons/meters. Combined CPU averaged 4.507% of one core
(2.007% daemon), with flat 45.63 MiB daemon RSS, stable handles/threads,
no restarts and unchanged layouts. Speech dummy/brief corked Chromium streams
are documented; this is a finite saved-layout workload, not a four-target soak
or matched comparison with earlier builds. Current development idle/loaded
qualification is complete; final signed candidate admission remains separate.
See the [coordinator checkpoint](checkpoints/2026-10-15-coordinator-optimization.md).

**Design guidance:** Impeccable remains adopted for clarity, spacing, long labels,
unavailable states and accessibility. Adapt its ideas to native GTK/libadwaita;
its web detector and mobile audit do not qualify Decksmith's desktop interface.
Preserve the accepted layout, GNOME theme and performance budget. No tools or
hooks were installed. See the [toolbox assessment](research-toolbox.md).

**October 14 physical diagnostic checkpoint:** the new connected runtime passed
two timed Plus/MK.2 hardware windows using a shared simulated level/mute target.
Both devices supplied successful shared actions, pages/brightness stayed
independent, each partner remained available through the other's disconnect,
and both reconnected. Joined blanking and automatic V1/config/page restoration
passed. Full native providers, API/editor, lock/suspend lifecycle, migration and
resource gates remain. See the [qualification and limits](checkpoints/2026-10-14-connected-physical.md).

**October 14 first native checkpoint:** both physical Decks passed real named
output volume/mute and explicitly pinned Brave pause/resume through the connected
runtime, with shared output observation and one meter. The final recording has
28 successful action replies, zero feedback failures and no uncertain effects;
exact output/playback and normal V1/config/page restoration passed. This is an
opt-in development slice, not a desktop upgrade. Next connect established app/
System sounds/player selection rules and saved profiles; then complete the
device-addressed editor/API and native lifecycle/resource gates. See the
[native qualification and limits](checkpoints/2026-10-14-connected-native.md).

**October 14 saved-provider bridge:** the opt-in connected runtime derives
requirements from effective saved key/dial assignments, deduplicates observation,
and connects guarded app/System sounds writes and pinned family/automatic media
selection. Deliberate labels/artwork and legacy master-volume semantics are
preserved. Provider-set edits require prepared refresh before runtime application;
live editor/API activation, all-active-target metering and expanded physical/
lifecycle/resource checks remain. See the
[saved-provider scope and verification](checkpoints/2026-10-14-saved-native-providers.md).

**October 14 live editor bridge:** a separate opt-in endpoint now connects the
integrated editor to prepared, device-addressed live saves. Disk/preview/provider
work stays off the input coordinator; only the edited Deck is fenced, observer
subscriptions use epochs, and partner media pins/contexts remain unchanged.
Production activation still requires expanded physical assignment acceptance,
all-active-target meters, lifecycle/recovery and resource checks. See the
[live editor qualification and limits](checkpoints/2026-10-14-live-profile-editor.md).

**October 14 paired editor acceptance:** both physical Decks now pass native label
saves and independent Home brightness settings with custom labels retained.
MK.2 blue/Plus copper page isolation and four real shared output volume/mute
effects were recorded. A focused post-save Plus playback check recorded four
physical press/release events and two successful Brave commands. Both guarded
windows blanked both devices and restored exact native audio/playback state and
the unchanged normal V1 build/config/page. The daily service remains V1.
Next connect meter subscriptions to all active saved audio assignments, then
qualify assignment changes and recovery/lifecycle/resource gates before activation.
See the [paired editor evidence and limits](checkpoints/2026-10-14-live-profile-editor.md#paired-physical-editor-follow-up).

**October 14 saved meter bridge:** the opt-in runtime now derives peak subscriptions
from connected touch-capable devices' active saved dial assignments and page
overrides. One shared worker deduplicates targets, fences changed unions by epoch
and routes each device's levels only to its strip. MK.2 and inactive/offline pages
create no monitor streams. The 64-stream bound is retained and empty subscriptions
sleep. The guarded physical output/app/page-override check also passed: 66
successful audio effects, unavailable-target isolation, Plus disconnect releasing
peaks with MK.2 continuing, and correct reconnect/resubscription. Exact app/output/
playback and normal V1 recovery passed. The later System sounds isolation check
below also passed; lifecycle/resource qualification remains before activation; the installed V1 build is unchanged. See the
[saved meter boundary and acceptance gates](checkpoints/2026-10-14-saved-active-meters.md).

**October 14 native microphone bridge:** named input mute and push-to-talk now
share one owner in the opt-in connected runtime. Mute cancels all active holds;
the last release returns to Muted. Ordinary explicit unmute remains separate from
holds and requires a current-scope commit. Retained source/server pins prevent
retrying against replacements, and joined shutdown precedes recovery. Complete
isolated build/editor checks and a private native PipeWire microphone acceptance
passed. The paired physical microphone/input-meter check described below also passed;
the installed V1 build is unchanged. See the
[native microphone qualification and limits](checkpoints/2026-10-14-native-shared-microphones.md).

**October 14 unattended follow-up:** 60 paired simulated cycles passed independent
saves, overlapping holds/Mute override, lock cancellation, strip-only meter writes
and twelve alternating reconnects. Explicit verified history archival preserved
active profiles/contexts. Nine private native microphone fault cases and forty
repeated microphone cycles passed, with no production-source/build changes or
physical device access. Synthetic resource samples are not a production soak.

**October 14 paired microphone acceptance:** physical Plus/MK.2 holds, overlap,
last-release mute, cross-device Mute override, persistent ordinary unmute, held-page
cancellation and both input meters passed user acceptance, corroborated by pinned
native source-state observations and combined physical receipts. Both owners blanked
and exact microphone/output/playback/configuration/build/page restoration passed;
the user confirmed normal V1 recovery. Paired lifecycle/migration/resource
qualification remains before activation. See the
[physical microphone evidence and limits](checkpoints/2026-10-14-native-shared-microphones.md#paired-physical-microphone-acceptance).

**October 14 System sounds meter acceptance:** a guarded native trial passed
physical output-versus-event isolation: ordinary audio moved only the output
meter, while an event-role tone also moved System sounds. Missing app stayed
unavailable, and ended events did not borrow output peaks. All fixture processes
were reaped; exact event/output/playback/configuration/build/page restoration
passed. The trial did not exercise System sounds volume/mute actions or measure
resource budgets. See the [meter isolation evidence](checkpoints/2026-10-14-saved-active-meters.md#physical-system-sounds-isolation).

**October 14 held-device recovery:** both physical Decks passed disconnecting during
a microphone hold, no stale unmute after reconnect, and fresh-press recovery. The
surviving MK.2 hold stayed Live across Plus disconnect until its own release. Pinned
source observations, device reconnect states and combined receipts corroborate
the bounded trial; a separate requested page-state mismatch is documented and not
counted as new navigation evidence. Both owners blanked, sampled helpers exited,
exact state restoration passed, and the user confirmed normal V1 recovery. Paired
lock/suspend/startup integration, migration and resource qualification remain.
See the [held-device recovery evidence](checkpoints/2026-10-14-native-shared-microphones.md#physical-held-device-recovery-follow-up).

## Next work priorities — Monday, October 12

These are the first two tasks, ahead of new V1.5 features. Work started early
on October 11 at the user's request; verify the installer before publishing it.

1. **Simplify installation, including dependencies.** Provide one recommended
   entry point that identifies the distribution, version and architecture,
   explains support status, selects the correct stable or experimental package,
   and offers installation of missing runtime dependencies. The historical URL
   bootstrap defaulted to `v0.1.0-preview.3`; the implemented helper now selects
   the existing signed stable/experimental releases. Preserve signed provenance,
   accepted source/workflow identity, complete checksum coverage and archive
   validation before installing.
   Stop clearly for unsupported platforms or failed verification. Consolidate
   package requirements used by the installer, runtime doctor and documentation;
   distinguish required runtime packages from optional desktop integrations and
   source-build tools. Keep the application installation under the normal user,
   with elevation only for necessary system packages or USB permissions. Preserve
   layouts, artwork, startup preferences and the existing build/configuration
   recovery path. Test fresh installation, missing dependencies, upgrade,
   verification failure and recovery in the appropriate AI-server guests.
2. **Reorganize the README around a new user's needs.** Lead with a short product
   explanation, one current interface image and the video, followed by available
   features, a compact support/download table, verified installation and first
   use. Clearly separate Fedora 44 support, experimental Debian/Ubuntu/COSMIC
   downloads, test-only environments and unreleased development features. Move
   detailed qualification evidence, architecture and developer setup to linked
   guides. Keep concise dependency and troubleshooting links, the explanation of
   application-stream volume versus in-app sliders, and System sounds versus
   output volume. Preserve accessibility guidance, AI-assistance disclosure,
   license/trademark notices and the support button at the bottom. Check all
   links, image availability and release labels against the actual downloads.

**October 11 implementation checkpoint:** the simplified signed helper and
README reorganization are implemented and approved for source publication. The helper passes 108 script
tests and real signed isolated installation in all four download-platform VMs;
all test guests are shut off afterward. It offers runtime package setup and an
optional pinned temporary GitHub CLI when OS packages are too old for secure
verification. The README separates Simplified Installer from Manual Installation,
with the gallery and detailed dependencies in linked guides. Bare-desktop dependency acceptance remains a separate check; existing
release assets and production controls are unchanged. See the
[installer qualification](checkpoints/2026-10-11-simplified-installer.md).

**Fresh dependency acceptance — October 12 candidate passed:** the scheduled run
first failed during VM firmware selection. The new Fedora 44 Workstation guest
now boots correctly and retains an independent pristine disk. Real terminal
testing exposed a prompt bug in the public helper; the reviewed fix passes clean
installation, a controlled missing-dependency case, runtime readiness, desktop
integration with startup remaining off, and native Wayland editor/VirtualDeck
checks. All seven server guests are off afterward. The fix is published to
GitHub, and a fresh public-shortcut download matches the tested helper byte for
byte. Gitea synchronization completed after its server update on October 12;
both repositories were verified to contain identical source trees.
Physical USB/audio and the other distributions' fresh dependency paths retain
separate acceptance gates. See the
[fresh installer qualification](checkpoints/2026-10-12-fresh-installer-acceptance.md).

**Fresh Debian dependency acceptance — October 12 passed:** a separate Debian 13
cloud-image-plus-GNOME guest passed the unchanged public helper with experimental
opt-in. It detected and installed three missing packages through real APT/terminal
consent, verified the existing signed preview, preserved saved labels/artwork,
brightness and startup preference through same-build reinstall, and passed
backup/restore and original-integration recovery. Tampered installer and wrong
source identity were rejected before installation. Native Wayland panel/editor
and isolated VirtualDeck checks passed. The pristine baseline and two corrected
harness-failure records are retained; all eight server guests are off and VM
autostart remains disabled. This does not qualify older-engine/schema upgrades,
physical USB/audio, lifecycle, portal/file-picker or accessibility acceptance.
Ubuntu and COSMIC fresh dependency paths were the next checks. See the
[Debian qualification](checkpoints/2026-10-12-debian-fresh-installer.md).

**Fresh Ubuntu dependency acceptance — October 12 passed:** a new official,
signature-verified Ubuntu 26.04 cloud base with a repository-installed GNOME
desktop passed the unchanged public helper. It filled two missing runtime
packages, passed signed admission and negative checks, preserved saved data and
startup preference through repair/reinstall, and passed backup/integration
recovery and native Wayland editor/VirtualDeck checks. Its first fixture stopped
before removing a package that would also remove printer support; the safer
single-package fixture passed from the pristine baseline. The guest is off.
COSMIC fresh dependency acceptance was next; older-engine/schema upgrades
and physical acceptance retain separate gates. See the
[Ubuntu qualification](checkpoints/2026-10-12-ubuntu-fresh-installer.md).

**Fresh COSMIC dependency acceptance — October 12 passed:** an independent
official-ISO Pop!_OS 24.04 guest passed the unchanged public helper after OS
updates, a clean package audit and confirmed updated-kernel boot. The helper
installed five missing runtime packages with real terminal consent. Signed
admission/rejection, same-build repair/reinstall and saved-data recovery passed,
alongside native COSMIC Wayland editor/VirtualDeck checks at 1920 × 1200.
The test OS setup required an EFI mount correction and preservation of an
existing firmware-service configuration; those logs are retained. The guest is
off and its pristine disk is preserved. Fedora, Debian, Ubuntu and COSMIC now
have fresh installer evidence for their tested environments. Older-engine/schema
upgrades and physical/interactive acceptance remain separate gates. See the
[COSMIC qualification](checkpoints/2026-10-12-cosmic-fresh-installer.md).

**Fresh-guest reboot follow-up — October 12 passed:** Debian, Ubuntu and COSMIC
retained their installed build, saved layout/artwork and brightness through a
confirmed reboot with a changed boot ID. Login startup stayed disabled and
controls stayed stopped at login. Explicit normal-session start/stop exposed the
control API, waited for the absent USB device without service restarts and
released its bus name on stop. Layout/artwork hashes were unchanged; preference
values were checked semantically because startup can normalize JSON formatting.
All ten server guests are off with autostart disabled. This adds VM reboot and
service evidence, not physical lock/suspend/hotplug or older-engine/schema
upgrade acceptance. No application code or release files were changed.

**Optional supporting trial: RepoWise.** Evaluate it on a clean copy of Decksmith
for installer dependency tracing, documentation drift and later multi-device
change impact. Start with local deterministic analysis, without model calls,
transcript capture, hooks or automatic agent configuration changes. Measure
accuracy, indexing time and resource use before adding it to the regular
workflow. This evaluation is planned, not installed or qualified, and is not a
prerequisite for the two tasks above. See the [research toolbox](research-toolbox.md).

## OS testing host policy — October 10

Use the AI server as the primary host for OS compatibility and VM-based tests.
The Linux test guests (Fedora Workstation 44, Fedora Workstation 45 Beta,
Debian 13, Ubuntu 26.04 and Pop!_OS 24.04 COSMIC) have independent server copies
managed from the workstation through virt-manager. Keep workstation originals
as recovery copies; Windows reference-review testing remains on the workstation.

Use the workstation when a test requires direct physical Stream Deck access.
VirtualDeck and VM checks complement physical acceptance and do not replace it.
Refresh each guest's installed build before qualifying a new candidate, and record
the build and environment with its results. Start guests only when needed, shut
them down or pause them when not in use, and keep VM autostart disabled. Prefer
the server's on-demand test runner for unattended checks; do not interrupt a
guest already in use.

**Scheduled testing — authorized October 10:** run source builds, Rust/Python
regression tests, VirtualDeck checks and the dependency audit nightly at 2 AM
Eastern. Test the six Linux guests sequentially on Sundays at 6 AM Eastern.
Before each guest test, save and verify an offline recovery copy, apply OS package
updates, reboot and confirm a new boot ID. Leave guests off afterward and skip
guests already in use. Record update/reboot failures separately from test failures.
Nightly tests use a separate checkout of published GitHub main; VM tests record
the installed guest build and updated OS/kernel. Those are distinct qualification
contexts, not a promise that every guest automatically receives a new build.
Server timers run independently of the workstation; an hourly Codex follow-up
reports new failures, incomplete runs and recovery when the desktop app is
available. Keep logs and backup manifests; do not automatically restore disks or
force off a failed guest. The AI server host and Windows reference VM are outside
these guest-update jobs.

**Weekly maintenance recovery — October 12:** the test runners now prefer a
normal OS poweroff through the pinned, identity-checked guest connection, with
explicit graceful guest-agent/ACPI fallback and confirmed powered-off state.
They retain ownership locks, bounded waits and durable failure records; no
forced power-off is attempted. A five-minute bounded retry handles the specific
APT list-lock error exposed by COSMIC's background updater. Twenty-three
failure-path checks pass locally and on the server. The final six-guest batch
passed verified offline backup, OS updates, reboot verification, installed-build
smoke tests and shutdown. All seven server guests are off, the recurring timers
are unchanged and enabled, and the workstation build is untouched. COSMIC's
final smoke check was headless VirtualDeck; it does not add native-editor or
physical acceptance. See the [recovery qualification](checkpoints/2026-10-12-weekly-vm-recovery.md).

**openSUSE Tumbleweed GNOME — added October 10:** the sixth Linux test guest is
installed directly on the AI server, registered in its virt-manager User session,
with desktop autologin, a 1920 × 1200 display and VM autostart disabled. Initial
checks passed for checksum-verified development-build installation, native
GNOME/Wayland integrated Home/Pages/Keys/About navigation, stable window sizing,
key/dial selection, artwork picker cancellation, VirtualDeck previews, page
navigation, layout save/validation, SSH and HTTPS. The recorded environment is
Tumbleweed snapshot 20261009, GNOME 50.5, GTK 4.22.5, libadwaita 1.9.4 and
Decksmith development build `1.0.0-80b6faedd10b`.
Record the rolling snapshot, dependency versions and candidate build for each
qualification. Include native GNOME/Wayland editor, artwork picker, installation,
VirtualDeck and service checks in the OS test matrix. Physical USB/audio,
accessibility and lifecycle checks retain separate acceptance gates. Adding this
test environment does not establish a supported openSUSE release.

## Optional research toolbox — October 11

Ghidra is available as a separate AI-server research tool alongside the existing
workstation REA trial. Use official HID/SDK documentation, saved profiles and
observed behavior first; use targeted native analysis when a specific gap remains.
Keep reference binaries and recovered implementation/assets outside public source,
and verify findings through physical/reference-app acceptance. These tools are not
Decksmith runtime dependencies or automatic release gates. See the
[research toolbox](research-toolbox.md) for scope and workflow.

Awesome LLM Apps development skills, Awesome MCP Servers and Agents Towards
Production are also listed as optional references. Evaluate narrowly for change
review, tool discovery and development-job observability; no installation or
runtime integration is approved merely by adding a reference. Keep the installer
acceptance work ahead of these trials and review each candidate's licensing.

## Today's short list — October 9

1. **Touch-strip dial gauges:** implement the accepted dual-arc mock-up
   inspired by the supplied Wave Link recording. Keep volume and live signal
   distinct; retain red Muted/crossed-out icons without a red border and green
   microphone Live text. Offer the existing bar presentation as another style.
   Qualify rendering, controls and resource use before installing app changes.
2. **Pop!_OS COSMIC initial compatibility:** create a separate Pop!_OS 24.04 LTS
   VM in virt-manager's User session, with autostart disabled. Establish native
   editor/dependency, file-picker, service, audio and session capability results.
   Track COSMIC separately from the GNOME/Wayland certification goal; do not assume
   GNOME extensions or desktop-specific actions work. Shut down when not testing.

**October 9 checkpoint:** the dual-arc design is implemented in a local test build,
with the classic bar available per dial. A subsequent Single-arc meter option
shows one audio activity arc with the configured percentage on its right, while
preserving assigned turn/press actions. It passes
rendering, scope/persistence and Fedora VM native-selector checks; physical
acceptance remains pending. Existing public release assets are unchanged. Cached arc geometry and pointer masks
keep the existing meter sampling rate; preview/device pixel parity, all integer
levels, mute/missing signal states, enlarged text and style persistence pass.
Fedora 44 native editor and VirtualDeck checks pass. The local build is installed
on the desktop with the prior build and configuration backup retained; saved
configuration/artwork hashes are unchanged. The user confirmed physical gauge
readability, mute/live indications and responsive controls. A measured resource
comparison remains before a public release.

The updated Pop!_OS 24.04 COSMIC VM is registered with autostart disabled.
A separate Ubuntu 24.04-baseline signed candidate passed provenance/archive
verification, installation/recovery, native editor and VirtualDeck checks, and
normal service start/stop. The user accepted all sections, stable key/dial sizing,
keyboard focus/navigation and the artwork picker at 1920×1200. Orca is not tested.
Runtime package hints follow Ubuntu/Debian ancestry; GNOME-only DND correctly
reports unavailable. The bounded initial monitor lookup does not fully fix small
1280×800 launches: COSMIC can still select 900×540 and clip controls; enlarge the
window. Physical USB/audio, lifecycle recovery, resources and COSMIC-specific
integration remain unqualified. The experimental `v1.0.0-popos-preview.1` prerelease is published; fresh downloads
matched and verified. Test controls are stopped and the VM is shut off with
autostart disabled.
See the [signed COSMIC qualification record](checkpoints/2026-10-09-popos-cosmic-signed-preview.md).

## V1.5 feature plan — organized October 8

This is the current V1.5 overview, grouped by feature. Detailed requirements and
accepted checkpoints remain in the related sections linked below. These are
planned tasks, not features in published V1 or a calendar promise. Earlier dated
next-step lists describe their historical checkpoint; use this overview to resume.

### Interface clarity and editing — approved October 14

Reduce repeated information and competing controls as V1.5 adds devices and
features. The interactive simplified Keys & Dials mock-up was approved on
October 15. Its native GTK/libadwaita implementation passed automated qualification and is
installed locally for review. Remaining desktop acceptance is tracked in the
[interface checkpoint](checkpoints/2026-10-15-simplified-native-editor.md).

- **One device selector:** keep Choose Device in the persistent header with the
  selected Deck's name and connection state. Remove the duplicate Home selector.
  Keep navigation stable, preserve each device's drafts, and make the device being
  edited unambiguous.
- **Focused Home:** lead with connection, Background controls and brightness;
  retain Start Decksmith at login and Auto-Lock here. Put profile history and
  archiving inside a secondary inline Manage profiles section.
- **Assignment first:** organize key editing as Label → Action → Target. Retain
  Transparent label background directly beneath Label. Show only the relevant
  action fields, extending the editor's existing conditional visibility rather
  than duplicating controls.
- **One searchable action chooser:** group Applications, Audio, Media, System,
  Navigation and available plugin actions. Include Homebridge when its integration
  is available, with its experimental status clear. Avoid separate competing
  built-in/plugin assignment flows and expose no unfinished installation features.
- **Compact dial editing:** use one Assignment block for label, turn action,
  target and press action. Keep gauge style easy to find; expand less common
  behavior and detailed styling inline. Preserve shared/page-specific scope,
  press/hold semantics and the accepted gauge/mute/Live presentation.
- **Appearance by expansion:** retain the existing inline Appearance expander,
  with a concise summary such as “Page theme · Automatic icon.” Detailed fonts,
  colors and sizing remain available without adding a separate window. Preserve
  theme inheritance, custom artwork and explicit overrides.
- **One save action and status:** retain one fixed Save and Apply button and one
  authoritative saved/unsaved indicator. Remove repeated healthy-state messages.
  Put actionable validation beside the affected field; clarify any necessary
  distinction between saved edits and immediately applied device settings.
- **Stable native layout:** preserve left-side editing/right-side device preview,
  GNOME appearance, subtle section colors, visible keyboard focus and consistent
  spacing. Reserve stronger emphasis for selection, warnings and the primary
  action. Expanders, tab/key/dial changes and background updates must not shift
  window geometry, steal focus or reset typed text. Keep motion lightweight and
  purposeful; do not add glass effects or automatically rearranging navigation.

**Visible scrolling — added October 15:** whenever controls extend beyond the
visible app area, show a persistent, clearly contrasting scrollbar rather than
relying on hover-only/auto-hiding overlay bars. Add unobtrusive “More settings
below/above” cues that reflect the actual scroll position and disappear when the
content fits. Keep the Save action outside the scrolling editor, reserve scrollbar
space to prevent sideways shifts, and support wheel, trackpad and keyboard access.
Apply this consistently to overflowing settings, page/plugin panels and previews;
never leave clipped controls without a visible route to reach them. Check expanded
Appearance, long action results, small windows, enlarged text and both themes.
The native implementation reserves a scrollbar gutter, shows actual-position
above/below cues and does not add a polling timer. All-panel desktop keyboard/Orca
acceptance remains pending; automated native layout checks are separate evidence.

Mock-up review covers three tasks: assign an application key, configure an audio
volume dial, and switch between Plus and MK.2 without losing edits. Include saved,
unsaved, invalid-field and disconnected-device states. Show the MK.2's applicable
key controls without dial settings. Use representative window sizes and long/emoji
labels. Native acceptance then checks keyboard/Orca, light/dark and enlarged text,
preview parity, draft retention and idle/active CPU and memory against the baseline.
A mock-up is design review, not physical-device or performance qualification.

References reviewed October 14: [Unico Connect](https://unicoconnect.com/blogs/ui-ux-design-trends-2025)
for calm hierarchy and purposeful feedback;
[Sphinx](https://sphinxjsc.com/blog/app-design-for-2026-trends-techniques-and-tools)
for task-focused screens, clear primary actions and predictable performance;
[Fuselab](https://fuselabcreative.com/ui-ux-design-trends-2026-modern-ui-trends-ux-trends-guide/)
for stable navigation and preferences. Apply these selectively with the adopted
[Impeccable guidance](research-toolbox.md#impeccable--design-guidance-adopted-october-14),
using existing GTK/libadwaita components rather than adding a runtime/framework.


### Reference-derived improvements — approved October 15

[Deck-Switch and Deckhand](research-toolbox.md#deck-switch-and-deckhand--reference-repositories-adopted-october-15)
remain design/architecture references, not dependencies or copied implementations.
Apply only changes that support the focused release; the remaining ideas below
stay in their feature tracks.

- **Capability and failure clarity:** explain missing software, permissions or
  session support inline, preserve unavailable assignments, and refresh on actual
  provider/session changes. Keep failures scoped to device/profile/page/control,
  clear them on success or reassignment, and avoid healthy-state pop-ups. Any
  optional notifications should replace duplicates rather than stack.
- **Efficient observation:** share cached audio/system observations across devices,
  react to meaningful events and avoid identical USB writes. Measure our actual
  idle/interactive workloads; another project's CPU claim is not our benchmark.
- **Explicit copy tools — follow-up:** offer Copy to page/device with compatible
  key geometry, overflow/unsupported-control/navigation warnings, atomic save and
  undo. Keep new MK.2 defaults and each device's later layout independent; copying
  is opt-in. Retain one selected-device view rather than adopting a dense canvas.
- **Dial Stack clarity — deferred follow-up:** use a named list and clear active
  target, while retaining the approved tap-to-cycle behavior and existing
  push-to-talk/press semantics. Do not replace these with long-push cycling.
- **V2 native plugin fields:** localized schema-driven labels/help, conditional
  settings and supported key/dial capabilities inside the existing native editor.
  Retain process isolation; do not adopt in-process plugin loading.
- **V2 monitors:** one cached, modest-rate sampler per source with shared visible
  subscriptions; coalesce expensive GPU queries and stop unnecessary hidden work.
  Multi-action delays/cancellation, network decks and animated backgrounds remain
  separate follow-up ideas, outside tomorrow's release scope.

### Appearance and artwork — unfinished work deferred October 15

- **Website-artwork refresh polish:** automatic icons follow URL changes while
  preserving custom images and labels.
- **Panoramic key wallpaper:** one static image across the key grid, automatically
  sliced using physical geometry, with inline positioning and per-key overrides.
- Work within the existing native Page Appearance/editor and shared preview/device
  renderer. See [appearance requirements](#4-appearance-presets-and-local-icon-library--implemented-september-14).

### Audio, media and dial controls — gauges retained; unfinished extras deferred October 15

- **Now Playing:** touch-strip track/artist, optional artwork and playback progress
  through the existing Linux media integration.
- **Action Wheel:** rotate to select an action, press explicitly to execute.
- **Dial Stacks:** tap the relevant touch-strip section to switch adjustable
  controls, then rotate to adjust the selected target.
- **Dial gauge presentation:** accepted dual arcs for configured volume and
  independent live signal, with the classic bar retained. Implemented in the
  October 9 local test build, with physical acceptance complete; resource
  qualification remains before release.
- Define clear selected-target feedback and consistent tap/press/hold behavior;
  preserve audio mute, push-to-talk and page overrides. See
  [media and dial requirements](#now-playing-dial-action-wheels-and-dial-stacks--v15-targets-added-october-8).

### Pages and shared actions — unfinished extras deferred October 15

- **Key press gestures:** finish independent Single press, Double press and Hold
  assignments in the integrated key editor and daemon. Reuse the existing core
  resolver, retain normal single-press behavior and push-to-talk, and make gesture
  conflicts and timing clear. See [key gesture requirements](#key-single-press-double-press-and-hold--v15-target-added-october-9).
- **Sticky keys:** one shared action/appearance/label binding across pages, with
  visible inheritance, edit propagation and explicit per-page conflicts/overrides.
- Preserve existing automatic/manual page switching and saved layouts. See
  [pages and reusable actions](#profiles-pages-and-reusable-actions--clarified-september-13).

### Devices and simultaneous control

- **Multi-device support — V1.5 requirement:** run multiple connected Stream
  Decks at the same time, with a named Choose Device selector, connection status,
  independent layouts/pages and device preferences. Disconnecting or recovering
  one device must not interrupt another. See
  [multi-device requirements](#simultaneous-multiple-device-support--clarified-september-13).
- **Initial mixed-device acceptance:** Stream Deck + together with the user's
  connected 15-key MK.2. Complete MK.2 geometry, key rendering/input, brightness and
  recovery prerequisites for this pair; qualify each model before advertising
  support. Simulate two identical devices as well as mixed models, and track the
  additional physical identical-model check separately.

### Plugins and integrations — moved to V2 on October 15

- **Native host foundations:** versioned contracts and saved settings, isolated
  processes, bounded dispatch, permissions, cancellation and recovery.
- **Plugin management:** installation, enable/disable/removal, inline settings,
  reference plugin and developer documentation.
- **Built-in system monitors:** ship local system-information displays with
  Decksmith, including CPU, memory, storage, network and uptime; expose available
  hardware temperatures/GPU metrics with clear fallbacks. No third-party plugin
  download is required. See [system monitor requirements](#built-in-system-monitors--v2-target).
- **Dependency diagnostics:** actionable missing/incompatible-provider information
  while retaining unavailable assignments and settings.
- **Compatibility:** native host first, then the verified OpenAction subset and
  catalog; separately qualify compatible Linux-native/Node.js Stream Deck plugins.
- **Optional feasibility only:** Spotify-specific and AI controls after the native
  host; neither is core V1.5 release work. See [plugin foundations](#plugin-architecture-foundations--remaining-production-work)
  and [plugin delivery](#plugin-support--v2-target).

### GNOME/Wayland compatibility and installation

- Build on the experimental Debian 13 and Ubuntu 26.04 previews; complete native
  dependencies, GNOME extension/session integration and distribution-aware install,
  upgrade and recovery paths.
- Maintain the Fedora 44, Fedora 45, Ubuntu and Debian qualification matrix;
  certify each advertised version/architecture independently. VM results do not
  replace physical USB/audio/recovery acceptance. See
  [distribution compatibility](#broad-gnomewayland-distribution-compatibility--v15-goal-v1-vm-diagnostics).
- **Pop!_OS 24.04 COSMIC investigation:** add an isolated, manually started VM
  for initial compatibility checks. Treat COSMIC's session, portals, lock state
  and desktop actions as a separate integration track; the GNOME extension does
  not establish COSMIC compatibility. Initial tests are not a supported release.

### Shared quality gates and delivery dependencies

Every feature preserves custom labels/artwork, saved assignments, undo/redo and
import/export; stays inside the integrated GNOME editor where practical; and
passes accessibility, security, lock/cancellation, recovery, preview parity and
idle/loaded resource checks. Cache static work and use bounded asynchronous/event
updates rather than adding unnecessary polling.

**October 12 architecture review:** [ADR-0017](adr/0017-device-scoped-actions-and-settings.md)
maps the current single-Plus implementation and proposes the device/action,
settings-migration and cancellation boundaries needed for V1.5. The review and
first descriptor/VirtualDeck and model-document/migration slices are complete.
A bounded private inventory codec and supplied-adapter session supervisor now
pass simulated independent-device checks; production assignment, shared backends,
device-addressed IPC, editor and physical integration remain planned. See the
[model-foundation checkpoint](checkpoints/2026-10-12-device-model-foundation.md).
The [layout/migration checkpoint](checkpoints/2026-10-12-model-layout-migration.md)
records V1 preservation, V2 model validation, independent MK.2 defaults, action
identity and offline recovery tests. No migration bundle is activated by normal
startup; per-device preferences and deployed rollback still need integration.
The [session-foundation checkpoint](checkpoints/2026-10-12-device-session-foundation.md)
records stable-ID/ambiguity rules, fair polling, independent reconnect/page state
and stale-input/feedback fences. It is not wired to the production daemon.
The [shared-action checkpoint](checkpoints/2026-10-13-shared-action-foundation.md)
adds pure scheduling/target reservations, device-specific microphone leases and
instance-safe feedback. Native helper/guardian execution, canonical target
compilation and live device-addressed integration remain required before activation.
The user confirmed that MK.2
starts with its own default pages, independent of the existing Plus layout.

The [shared-worker bridge](checkpoints/2026-10-13-shared-worker-bridge.md)
adds a bounded off-HID executor pool, immutable compiled payloads, per-write and
consumer scope checks, and a dormant named-volume/output-mute native adapter.
Controlled backends qualify ordering, concurrency, cancellation and failure
isolation. Production dispatch, microphone guardians, aliases/app targets,
shared observation and physical acceptance remain unfinished.

The [shared-microphone guardian](checkpoints/2026-10-13-shared-microphone-guardian.md)
connects shared hold leases to bounded per-microphone guardians with independent
watchdogs, acknowledged mute, failure isolation and consumed recovery presses.
Its controlled/pipe/helper checks do not access real audio. Production routing,
canonical targets, persistent associations and physical PTT acceptance remain open.

The [saved-profile checkpoint](checkpoints/2026-10-13-saved-device-profiles.md)
connects explicit private identity/profile generations and per-device cached
control tables to supplied-adapter sessions. Independent saves, recovery,
all fifteen MK.2 addresses, page/brightness isolation and Plus behavior parity
are fixture-qualified. Live API/editor/rendering, bounded retention, hot enrollment,
backend routing and physical acceptance remain required before activation.

Proceed in small slices: (1) model descriptors and Plus/MK.2 VirtualDeck fixtures;
(2) model-aware layouts, stable action identities and migration/rollback;
(3) device manager, device-addressed dispatch/API and shared audio/PTT services;
(4) integrated device selection and MK.2 physical acceptance; (5) simulated pairs,
then physical Plus/MK.2 simultaneous recovery and resource qualification. The
first slice leaves the running desktop unchanged. General plugin-host
protocol/permissions and verified OpenAction mapping remain separate gates;
the experimental bridge does not close them. Within Appearance, refresh polish precedes wallpaper. Within
Media/Dials, specify interaction/rendering behavior before implementation, starting
with Now Playing, then Action Wheel and Dial Stacks. Catalog and optional service
integrations follow a qualified native host. Compatibility checks accompany each
packaged feature rather than waiting until the end.

**Separate future work:** full five-language localization remains V2. Below-1%
idle CPU and numeric visible-response qualification retain their post-V1 goals
without becoming new V1.5 promises. Animated wallpaper, independent full-width
touch-strip backgrounds, the full profile/workspace/state theme hierarchy and
other hardware certification remain separately scoped. MK.2 integration and
Plus/MK.2 simultaneous operation now belong to the V1.5 multi-device gate;
XL, Neo and other model certifications continue to follow the hardware matrix.

Review references:

- [Stream Deck app overview](https://www.elgato.com/us/en/s/stream-deck-app)
- [Stream Deck 7.6 release notes](https://help.elgato.com/hc/en-us/articles/51156609278737-Elgato-Stream-Deck-7-6-Release-Notes)
- [Stream Deck 7.5 release notes](https://help.elgato.com/hc/en-us/articles/48328697898897-Elgato-Stream-Deck-7-5-0-Release-Notes)

## Focused V1 boundary and release gates — September 23

**October 8 Debian compatibility checkpoint:** a native Debian 13 build resolves
the Fedora archive's newer-glibc requirement. Installation/recovery, native
GNOME 48 headless Wayland editor rendering, VirtualDeck and regression checks
passed. See [Debian build instructions and remaining gates](debian-native-build.md).
The [signed experimental Debian preview](debian-preview-installation.md) is available. Full-session navigation and file-picker
checks passed; screen-reader and physical device/audio/recovery qualification
remain open under the V1.5 compatibility goal; V1's
published Fedora support boundary is unchanged.

**October 7 release decision:** the maintainer accepted the frozen signed V1
candidate's documented testing coverage and known limits and authorized
publication of `v1.0.0`. Publication, public-download verification and README
promotion are complete; see the [release review](v1-release-review.md). Earlier dated
checkpoint lists describe their state at the time, not newly uncompleted tests.

The [reconciled V1 scope and acceptance plan](v1-release-scope.md) is the current
release checklist. V1 supports Fedora 44 GNOME/Wayland x86_64 with one Stream Deck +;
Fedora 45 final-release support needs separate final-release validation. All four
retained VMs — Fedora 44, Fedora 45 Beta, Debian 13 and Ubuntu 26.04 — must be
tested for every V1 release candidate. Fedora checks are supported-platform gates;
Debian and Ubuntu are compatibility diagnostics and must not be advertised as
supported installers because a diagnostic run succeeds. Physical USB/audio and
suspend acceptance still belongs on the reference desktop.

V1 qualification covered sustained core/audio/device reliability, human-paced
accessibility review, clean installation and migration/rollback across builds,
security and release-artifact verification, and the documented candidate matrix.
Its accepted coverage and limits are recorded in the release review. Advanced profiles/workspaces, workflows, SQLite migration, panoramic
canvases, general HTTP/MQTT and RPM packaging are post-V1 product goals. Broad GNOME/Wayland distribution compatibility remains a separately qualified
post-V1 goal; plugin delivery and full five-language localization target V2. The earlier broader v0.4 V1 list must not be used to
claim these features are already implemented.

### Post-V1 performance milestone — approved October 5

Retain the original normally-below-1% idle daemon CPU goal as post-V1 engineering
work, with no new version number or deadline. The accepted quiet reference result
is 2.11% daemon CPU and 3.75% full-service CPU of one core, with 17.90 MiB daemon
RSS. This is an explicit release-scope deferral, not a claim that the original
target passed. V1 still requires idle/loaded measurements and regression review,
bounded memory/handles, responsive input and display feedback, and all recovery,
security and release checks. See the
[resource acceptance decision](v1-release-scope.md#v1-resource-acceptance-and-post-v1-cpu-milestone).

Profile remaining periodic work, remove unnecessary queries/encoding and verify
idle and interactive improvements together. Keep full-service costs visible and
preserve control responsiveness, safety checks and recovery. Broad GNOME/Wayland
compatibility remains independently tracked; plugin delivery and localization
now target V2.

**V1.5 supporting performance work — added October 8:** retain asynchronous,
cached installed-app discovery and event-driven audio inventory updates. Measure
catalog refresh costs and coalesce redundant scans, following Elgato 7.6's
application-discovery performance improvement as a review prompt. Verify install/
uninstall changes remain discoverable, saved unavailable targets remain intact,
and new media displays do not materially regress idle CPU or memory.

### Post-V1 visible-response measurement — approved October 7

Numerical qualification of visible key updates below 150 ms and local dial
feedback below 100 ms is deferred beyond V1 by user approval. These targets
remain unmeasured on the frozen signed candidate, not achieved results. Preserve
functional responsiveness and the internal dispatch target. Plan synchronized
input/display capture or a well-anchored conservative movement bound; ordinary
unsynchronized footage and internal dispatch logs cannot certify pixels. Record
sample counts, exclusions, workload and exact artifact in the future measurement.
See the [release review](v1-release-review.md) and [protocol](input-latency.md).

## Agreed next steps — September 22

1. **Homebridge reliability and performance first.** Measure idle and active CPU
   and memory, including dial adjustment bursts. Use simulated accessories for
   connection loss, child/host failure and background-service restart tests. Verify
   bounded queues, recovery/backoff, no replayed commands and no unexpected device
   changes. Follow with a short, agreed live Main_LED’s check. Record duration,
   environment, peak/steady resource use, recovery outcomes and remaining limits;
   a short successful run is not sustained acceptance.
2. **V1 usability and accessibility polish.** Complete the focused review under
   Editor polish below: field validation, status feedback, keyboard access,
   contrast, stable geometry and consistent spacing. Preserve native GNOME theme
   behavior, existing layouts and the accepted inline editing flow.
3. **Follow-on editor improvements.** Evaluate searchable navigation/actions,
   artwork drag-and-drop and optional compact spacing after the V1 polish pass.
   These are planned candidates, not requirements to expand before reliability
   acceptance. General plugin delivery and five-language localization now target V2.

### Reliability checkpoint — September 22

The bounded Homebridge pass found and corrected read-outage recovery and catalog
refresh starvation during continuous dial activity. The repeat completed 1,444
simulated adjustments without errors; crash/outage/restart/no-replay regression
checks passed. Companion idle CPU averaged 0.56% of one core with flat sampled
RSS. The main service and helpers averaged 22.24% in a separate live workload
measurement. The subsequent main-service pass reuses the health helper and parses
unchanged Homebridge snapshots once: new one-minute samples measured 18.35% before
and 6.58% after, without slowing polling. Persistent-helper memory and longer
active-meter acceptance still need observation; see the
[performance checkpoint](performance-checkpoint-2026-09-22.md).
Longer multi-device/authentication/session-lifecycle acceptance remains open.
See [measurements, fixes and limits](homebridge-reliability-2026-09-22.md).

### Active-audio follow-up — September 22

A ten-minute reference-desktop run with one active playback stream completed with
zero service restarts and 41/41 healthy connection/display checks. Main service
CPU averaged 9.87% of one core; companion CPU averaged 0.56%. Memory and persistent
helper resources remained within a small range. Seventeen isolated recovery,
helper and queue tests passed afterward; no code fix was indicated. Continue the
V1 usability/accessibility review. Longer runs, multiple active meters and physical
session/USB recovery remain open; see the [results and limits](active-audio-reliability-2026-09-22.md).

### Accessibility and compact-layout follow-up — September 22

The native pass found and fixed clipped field beginnings caused by side-by-side
Homebridge buttons forcing a collapsed settings panel too wide. Compact native
high-contrast and 150% enlarged-text dark-mode checks now pass, including keyboard
navigation, draft retention and stable key/dial geometry; all 142 editor tests
passed. See the [checkpoint and remaining limits](accessibility-checkpoint-2026-09-22.md).
Screen-reader acceptance and actual compositor fractional scaling remain open.

## Current plugin work — native review bridge

**Current status:** the managed experimental Homebridge runtime, saved key/dial
assignments, account setup, access review and connection removal are installed
and user-reviewed. The paragraphs immediately below record earlier trial stages;
the September 21 managed-runtime section supersedes their placeholder and
opt-in-only limitations. General marketplace/plugin compatibility remains planned.


September 21: the isolated OpenHomeB live trial and recovery checks passed,
including user-confirmed on/off, brightness and feedback. An opt-in native panel
now occupies the Plugins tab in a separate source-built review window. Normal
installed launches still show the roadmap placeholder. This does not enable
physical key/dial assignments, generic installation or marketplace browsing.
See [ADR 0016](adr/0016-plugin-native-review-bridge.md) for the boundary, checks,
limitations and remaining integration gates. Native interaction acceptance is next.

The user accepted the native panel. The next experimental slice adds bounded,
versioned saved plugin bindings and a separate execution worker for the approved
Main_LED’s key on/off and dial-brightness subset. The opt-in local test host must
be running; this is not a general plugin installer or production autostart runtime.
HB TEST is the authorized separate trial page. Existing pages and shared dials are
preserved. Missing providers/schemas are retained without execution. In-flight
cancellation closes the bridge connection; already applied external effects cannot
be undone automatically. See the lab handoff for evidence and remaining acceptance.

## Plugin architecture foundations — remaining production work

The September 17 architecture priority remains: complete the necessary contract,
saved-layout and lifecycle foundations before dependent feature work. Plugin
delivery now targets V2; the limited managed experiment does not close the general
host gates below. Use the current V1.5 feature overview to resume; the September 22
sequence records an earlier reliability checkpoint.

1. **Review existing boundaries and write an architecture decision.** Map action
   definitions, saved layouts, dispatch, Auto-Lock/cancellation, rendering and
   helper permissions. Record what can be reused, actual gaps, and the smallest
   required changes. Define a versioned native plugin contract before building
   a host. Evaluate OpenAction before finalizing that contract: map its manifest
   identity, key/dial/touch events, settings, property inspectors and display updates
   to Decksmith. Record supported protocol versions and gaps in the ADR; retain
   Decksmith's native controls and settings model. Do not refactor working code
   solely for hypothetical future needs.
2. **Define stable action identity and settings compatibility.** Separate provider
   and action IDs, schema versions, settings and translated display labels. Plan
   migration of existing built-in actions without losing custom labels, artwork,
   history or imported layouts; add only necessary compatibility seams now.
3. **Specify shared dispatch and lifecycle rules.** Route future key/dial/touch
   actions through the existing lock gates, cancellation and stale-input checks.
   Define held-input cancellation, disconnect/resume behavior and action results;
   verify built-in behavior remains unchanged when making any small changes.
4. **Design missing-plugin persistence.** Retain unknown plugin assignments and
   settings across load/save/export/import, show them as unavailable and keep
   them non-executable. Validate structure and bounds without silently discarding
   unknown data; prepare migration and round-trip tests before implementation.
5. **Set resource and permission boundaries.** Define bounded messages/queues,
   action timeouts, restart/backoff policy and coalesced/rate-limited display
   updates. Specify host-mediated requests and permissions; no direct hardware
   handles or privileged-helper access. Distinguish crash isolation from sandboxing.

Acceptance for this preparation: a reviewable contract/ADR and migration plan,
focused tests for any necessary compatibility changes, no regression in built-in
controls or preview parity, and no material idle/active resource regression.
The managed OpenHomeB experiment below implements a limited execution path.
General plugin runtime/installation, Node.js bridging and WebKit settings remain
V2 work; the experiment does not establish general compatibility.

## Native workspace redesign — implemented September 17

The approved Home / Pages / Keys & Dials / About interface, muted section colors,
original About logo, tooltips, explicit Test action, keyboard save and close-draft
prompt are implemented. See [behavior and performance evidence](workspace-ui.md).
Independent multi-device control remains deferred by user decision. V1 polish
still includes sustained resource profiling, high-DPI/accessibility acceptance
and the wider hardware/desktop validation below.

## Plugins navigation placeholder — implemented September 17

Plugins appears between Keys & Dials and About, with a puzzle-piece icon and a
quiet roadmap message. This describes the original placeholder; the installed
experimental Homebridge manager now replaces it where that integration is present.
General marketplace downloads and plugin installation controls remain planned.

### OpenHomeB managed runtime trial — September 21

The opted-in Main_LED’s trial now packages a pinned companion runtime, follows
Decksmith Background controls and its existing login preference, and recovers
from child/host crashes without replaying commands. Stale artifacts and duplicate
hosts are checked; the editor reloads rotated connection details. Live stop/start,
child crash and host crash checks preserved light state and layout. An uncertain
command still requires explicit recovery in the running host. The accepted
1% dial increment and On-before-brightness behavior remain in place.
See [runtime scope and recovery](openhomeb-runtime.md).

September 21: capability-based discovery and inline key/dial assignments added.
Lights/sockets default to power toggle; fans offer native-step speed where supported.
Cameras and sensors provide status only. Existing HB TEST layout remains intact.
Main_LED’s live toggle/level checks passed; other devices need user acceptance.

September 21: Plugins connection manager added with persistent Enable/Disable,
server/health display and explicit Reconnect. Live disable/restart/enable checks
preserve assignments and physical state. Account setup and removal followed below.

September 21: inline server/account testing and explicit save added, with GNOME
Keyring passwords and session-only two-factor codes. The active connection is
retained until a candidate is tested and saved; assignments remain intact.

September 21: Plugins now shows saved key/dial assignments, including shared dial
inheritance and page overrides, plus the integration's actual access and sandbox
limitations. Inline Remove Connection requires confirmation, preserves assignments,
and optionally deletes the currently referenced Keyring password. Removed state
survives restarts and cannot reconnect until Test/Save creates a new connection.
Older backup credentials are not purged. Failure and cancellation checks pass.

Next: enforceable permission boundaries and sustained resource acceptance. This is not general plugin compatibility or an OS sandbox.

### Plugin support — V2 target

User decision, October 15: plugin delivery moves to **V2**, superseding the
September 17 V1.5 target and original V1 baseline. Complete core multi-device
reliability first; this is a release milestone, not a promised calendar date.

Deliver the native plugin foundation first:

- Versioned native contract for actions, live state and key/dial/touch-strip controls.
- Separate plugin processes with bounded communication, crash/hang recovery and
  clear diagnostics; built-in controls must continue working after plugin failure.
- Manifest/runtime validation, local installation, enable/disable and removal.
- **V2 plugin dependency diagnostics — added October 8, retargeted October 15:** imported layouts must identify
  the required provider/action, required versus installed version, missing runtime
  or unsupported platform, and an actionable next step. Preserve unavailable
  assignments/settings through editing, save and export; never execute an
  incompatible provider or silently substitute another action. Test missing,
  disabled, incompatible and recovered dependencies. Elgato's 7.5 import diagnostics
  inform this UX; catalog availability is not proof of compatibility.
- Declarative settings shown within the main GTK editor, plus clear capability
  and permission information. Do not describe process isolation as a full sandbox.
- At least one reference plugin and developer documentation, with acceptance for
  installation/removal, missing dependencies, crashes, restarts and resource use.

#### Built-in system monitors — V2 target

User request, October 8: include system-information capabilities with Decksmith
and organize them in the Plugins section as built-in providers. These ship with
the app, work locally without an account or marketplace download, and use the
native action/state/rendering contracts. They are separate from the external
OpenAction System Information plugin used for compatibility testing.

Initial monitor choices:

- **CPU:** overall utilization, with optional per-core display.
- **Memory:** used/total RAM and percentage; distinguish available memory/cache
  from actual pressure, with swap information where available.
- **Storage:** used/free space for a selected filesystem or mount.
- **Network:** receive/transmit rates for a selected interface, with units and
  clear disconnected/unavailable states.
- **System information:** uptime, distribution, kernel and basic hardware summary.
- **Hardware-dependent sensors:** CPU/GPU temperatures, fan speeds and GPU
  utilization/memory when a supported read-only source is available. Discover
  capabilities rather than promising every metric on every machine; missing
  sensors, guest limitations or permissions must show unavailable, not zero.

Allow assignment to keys and touch-strip regions. Provide readable values, units,
icons and optional compact bars/graphs with matching preview and hardware
rendering. Keep metric, source/interface/mount, label and appearance settings
inline in the main editor. Monitor displays are read-only; they do not alter
system settings. Retain user-customized labels and saved unavailable sources.

Use one shared sampler for duplicate subscriptions, bounded sample/history
buffers, cached static information and configurable modest refresh intervals.
Collect off the UI/input path, render only changed visible values, and reduce or
stop work for hidden/unassigned displays. Avoid privileged helpers and mandatory
vendor tools for basic metrics; document optional sensor dependencies. Preserve
built-in controls if a monitor source or optional collector fails.

Validate units and rate calculations, zero/idle values, counter resets, suspend,
interface/mount changes, missing sensors, permissions, stale data, import/export,
long/emoji labels and preview parity. Use all four Linux test VMs for supported
basic metrics, then physical key/touch-strip readability and representative host
sensors. Measure idle/loaded CPU and memory with several monitors active before
release. These capabilities are planned for V2, not present in published V1.

**OpenAction is the first V2 plugin compatibility target.** This means implementing
Decksmith's host-side adapter, not embedding the marketplace website or assuming
that the plugin-author SDK supplies a host. Follow this delivery order:

1. Complete the native host's isolation, lifecycle and bounded-update foundations.
2. Implement and document a supported OpenAction protocol subset. Validate it
   first with Counter and System Information, then a selected Linux-compatible
   plugin exercising dial/touch behavior. These are test candidates, not current
   compatibility claims. Verify plugin versions, settings UI, missing runtimes,
   device feedback, failure recovery and resource use.
3. After the host passes those checks, add OpenAction catalog browsing within the
   Plugins tab as a follow-on V2 target. Decksmith manages installation and
   settings; local package installation remains available independently of the
   catalog. Show **Tested**, **Untested** or **Unsupported** per plugin/version,
   with reasons. Catalog membership alone does not establish Linux compatibility.

Before integrating catalog data or installing a candidate, review metadata/asset
reuse terms, each plugin's license, runtime/architecture requirements and requested
capabilities. Treat catalog metadata and downloaded packages as untrusted input;
validate manifests and archives, identify the exact release/source, and require an
explicit installation action. Keep download/discovery separate from execution.
Do not grant device-support plugins direct hardware ownership through the action
adapter. Preserve local operation if the external catalog is unavailable.

References for the next review (recheck upstream versions when implementation starts):
- [OpenAction Marketplace](https://marketplace.tacto.live/)
- [OpenAction protocol documentation](https://openaction.amankhanna.me/)
- [Official plugin registry](https://github.com/OpenActionAPI/plugins)
- [Machine-readable catalog](https://openactionapi.github.io/plugins/catalogue.json)

V2 also targets an explicitly documented, tested plugin compatibility subset for open,
unprotected Stream Deck plugins: Linux-native executables first, then compatible
Node.js plugins. Compatibility requires its own bridge and isolated WebKitGTK
settings support where needed; ship only verified combinations and explain
unsupported requirements. Native support is the first delivery gate, not evidence
that arbitrary existing plugins work.

A Decksmith-owned marketplace/catalog, Windows plugins through Wine, and broader sandbox policies
are later work, not V1.5 release requirements. DRM bypass remains out of scope.
Full five-language localization remains scheduled separately for V2.

**Optional V2 integration feasibility — added October 8, retargeted October 15:**

- **Spotify:** assess library, playlist, search and queue controls through the
  official Web API, with explicit authorization, secure token storage, account
  requirements and API limits. Elgato's new Web API actions require Spotify
  Premium. Keep existing general MPRIS controls available independently; do not
  make Spotify accounts a core dependency or assume Elgato plugin compatibility.
- **AI assistant controls/status:** assess saved prompts and working/waiting/done
  feedback through a documented, authorized Linux integration. Elgato's current
  ChatGPT/Codex plugin requires its Windows/macOS desktop integration; Linux
  compatibility has not been established. A feasibility result must precede a
  delivery commitment; do not automate approvals or execute commands from status
  data. Follow the same native-host permission and resource boundaries.

These are follow-on candidates after the native host, not core V1.5 release work.
References: [ChatGPT/Codex plugin requirements](https://marketplace.elgato.com/product/chatgpt-codex-fb6a7052-2b1a-4500-ba2a-13996edf0b47)
and [SDK 3 changes](https://docs.elgato.com/streamdeck/sdk/releases/upgrading/v3/).
Recheck service requirements and supported protocols when implementation starts.

## Localization — foundation now; five-language release in V2

The gettext loader, catalog extraction/validation, bundle compilation and initial
navigation/confirmation examples are implemented. Current releases remain English
only; no complete translations or Unicode label changes are included yet.

V2 targets English, German, French, Spanish and Italian: full interface and GNOME
extension coverage, Unicode labels, localized physical-device messages, safe
handling of automatic/custom labels, system-language selection with an override,
long-text/accessibility testing, fluent-speaker review and translated onboarding.
See [localization scope and acceptance](localization.md). This rollout is explicitly
deferred from V1; new UI work should adopt the foundation as it is written.

## Integrated editor and dial improvements

The September 12 Windows recordings, physical-device photo, and follow-up decisions
set the following feature sequence. The integrated editor is implemented September
13 with GTK interaction checks; the user approved the refined device proportions
and matching physical touch-strip preview. On September 13 the user selected small
audio-target icons and red/crossed-out mute styling as the next focused increment,
using the shared renderer. That increment is implemented and checked in the live
editor. Live audio meters were subsequently implemented September 13; see
[audio meter behavior and validation](audio-meters.md).
Shared dial defaults/page overrides are implemented September 13;
retain the reliability and installation work below as ongoing requirements.

### 1. One integrated device editor — implemented September 13

Show keys, touch strip, and dials on one stable device canvas. Clicking any control
opens its properties in the same editor; clicking a touch-strip section selects
its associated dial settings. Preserve draft changes, undo/redo, collapsed appearance
controls, keyboard access, and responsive interaction during background refresh.
Add a searchable, grouped action catalog that filters for the selected control.
Keep the existing Percent per tick setting easy to find.

Retain the current Save and Apply behavior for this milestone. Reconcile the v0.4
auto-save/live-apply design separately rather than changing it as part of navigation.

### 2. Shared dial defaults with optional page overrides — implemented September 13

Implemented with optional per-page dial slots, a shared effective-dial resolver,
native Customize/Use shared settings controls, and draft/history/package coverage.
VirtualDeck checks cover dispatch, targets, rendering parity, and held-microphone
cancellation before switching targets. Live draft verification preserved saved
assignments. User confirmation of customized physical page switching remains useful.

Existing dial assignments become shared defaults. Let each page override individual
dials, visibly distinguish shared and customized settings, and offer Use shared
settings to remove an override. Shared edits affect only inheriting dials. Include
overrides in undo/redo and layout export/import; verify page changes update rotation,
press actions, touch behavior, and displayed targets together, including held
microphone controls.

### 3. Touch-strip audio identity, mute styling, and live meters

Implemented September 13: original vector microphone, speaker, headphone, and
application-category icons; neutral audio fallback; crossed-out icons and red
Muted text without a red border. Input Live readouts are green. The editor and physical strip use the same renderer.
Measured signal meters are now implemented for local PulseAudio-compatible
outputs, inputs, and application playback streams. Installed application artwork can now be embedded in dial assignments; the generic
application icon remains the fallback.

Use a distinct panel above each dial with a small target image beside the short,
editable label: microphone, speakers, headphones, or the application's icon as
appropriate. Provide a clear generic fallback when specific artwork is unavailable.
Do not infer a particular device type when discovery cannot identify it reliably.

Place the label and icon at the top, a readable volume percentage in the middle,
and the rounded blue live signal bar along the bottom. The numerical percentage represents configured volume; colored fill measures
actual activity. Do not animate the volume setting
as a substitute for measuring audio. Brightness retains its percentage display
without an audio meter.

When muted, show red Muted text and a crossed-out target icon without a red border: microphone for
inputs and speaker for outputs/application playback. Keep target identity readable
alongside the mute symbol. Mute must be recognizable without relying on color alone.
Distinguish muted, silent, and unavailable targets, and reflect external mute changes
as well as actions from Decksmith. Mirror these visuals in the device preview while
keeping live status separate from unsaved configuration.

Initial output/microphone/app monitoring and rendering checks are complete. Continue
hardware coverage for multiple streams, reconnects, and unusual devices. Metering is a separate capability
from the optional master mixer. Validate icon and label readability on the physical
touch strip, including long names, unavailable artwork, and all four active panels.

### Now Playing, dial action wheels and Dial Stacks — V1.5 targets, added October 8

**October 15 scope update:** unfinished delivery is deferred from the focused
multi-device V1.5 release. Requirements below remain the follow-up specification.

**Now Playing:** add an optional touch-strip presentation with track title, artist,
artwork and playback progress where the selected MPRIS player exposes them. Use
the existing explicit/automatic player selection rules; missing metadata or a
closed player must show a clear fallback rather than unrelated media. Keep volume,
signal level and playback progress distinct. Define panel allocation and tap/press
behavior in the integrated editor before replacing any saved audio presentation.

Subscribe to metadata/playback changes, cache bounded artwork, and update progress
only while its panel is visible and playing. Do not add high-frequency player
polling. Validate absent/long/emoji metadata, seek/pause/player changes, missing
artwork, page changes, preview parity and idle/loaded resource use. Follow with
physical readability and playback acceptance. This adapts the information-display
idea from Neo Infobar to the supported Plus touch strip; it does not require Neo.

**Dial action wheel:** rotate to choose a compatible existing action, then press
to execute it. Design touch-strip selection and explicit Back/exit behavior without
conflicting with swipe navigation, existing dial adjustment or held-microphone
controls. Keep settings inline and share rendering between preview and hardware.
Apply existing confirmations, Auto-Lock, cancellation and stale-input rules;
selection alone must not execute an action. Test rapid rotation, page/lock changes,
unavailable actions, import/export and physical selection/feedback.

**Dial Stacks:** group compatible adjustable controls on one dial, including audio
targets and device brightness. Tap that dial's touch-strip section to cycle through
the ordered controls, then rotate to adjust the selected control. Clearly display
the selected target, icon, value, mute/unavailable state and stack-position marker;
share rendering between the editor preview and physical device. This switching
and adjustment model passed the October 8 physical Windows reference review.
Decksmith implementation and acceptance remain planned.

Keep stack creation, ordering and editing inline. Support shared dial defaults
with optional page overrides, preserving existing assignments and custom labels.
Define press, mute and hold-to-talk behavior for each compatible control before
implementation; switching must not adjust another target or leave a microphone
hold active. Keep taps distinct from page swipes and leave existing single-control
dials unchanged unless the user explicitly converts them to a stack.

Use existing audio subscriptions and bounded caches; meter only the visible
selected control rather than every hidden stack member. Apply Auto-Lock and
stale-input cancellation on page/target changes, disconnects and recovery. Test
rapid taps and rotation, missing/reappearing targets, external value/mute changes,
press/hold release, inheritance, import/export and preview/hardware parity.
Require physical acceptance and idle/loaded resource checks before release.

References: [Infobar information displays](https://www.elgato.com/us/en/explorer/products/stream-deck/stream-deck-76-infobar-actions-for-stream-deck-neo/),
[Action Wheel behavior](https://www.elgato.com/us/en/explorer/products/stream-deck/what-is-an-action-wheel-for-stream-deck/)
and [MPRIS player interface](https://specifications.freedesktop.org/mpris/latest/Player_Interface.html).

### 4. Appearance presets and local icon library — implemented September 14

Implemented layout-wide typography/color/label defaults, key and dial overrides,
Reset to theme, compact per-key icon scaling (10–100% in five-point steps), and Maker’s Mark, Dark, Light and High Contrast presets. The offline
library includes 134 pinned official Tabler icons, searchable local imports and a
link to download the full official set. Renderer-backed key/touch previews, draft
history and theme/layout import/export preserve actions. See
[appearance and icon behavior](appearance-and-icons.md) and the
[September 14 checkpoint](checkpoints/2026-09-14.md) for validation and limits.
The broader v0.4 theme hierarchy remains planned; static panoramic key wallpaper
is now a V1.5 target detailed below. Independent full-width touch-strip backgrounds
remain a separate appearance milestone. A future
visual iteration should offer richer, less generic theme presets; the current
controls and presets are accepted for now.

**Website-artwork refresh polish — V1.5, added October 8:** retain the existing
Enter/focus-leave lookup and explicit Refresh website icon control, and make
automatic artwork follow URL changes consistently across relevant editing paths.
Distinguish automatic website artwork from user-selected artwork so a URL change
does not overwrite a custom image or label. Ignore stale fetch completions, retain
usable artwork on failures and preserve undo/redo and Save and Apply semantics.
Keep requests asynchronous, bounded and coalesced, using the existing URL/redirect
and image-decoding protections. Test rapid URL changes, custom artwork, failure,
history and imported layouts without fetching on every keystroke or opening
websites. This refines existing behavior rather than introducing website icons.

**Panoramic key wallpaper — V1.5, approved October 8:** let users apply one static
background image across a page's entire key grid without manually slicing it.
Use device geometry to sample each key's region, accounting for physical gaps so
the image remains visually coherent. Begin with the supported Stream Deck +;
reuse model-specific geometry when other devices receive separate qualification.

Keep selection, fit/crop, position, scale and label-legibility adjustments inline
in Page Appearance. Share the render model between preview and hardware, with
icons and labels layered over the wallpaper and explicit per-key background
overrides. Preserve saved artwork, custom labels and action assignments; applying
or removing wallpaper must not silently replace them. Define how existing full-key
artwork covers the shared background and make the result clear in the preview.

Decode and resize images asynchronously with existing import protections and
bounded memory. Cache the static canvas and key slices; rerender only after relevant
image, geometry or appearance changes. Preserve undo/redo, Save and Apply,
export/import and backward-compatible saved layouts. Validate gaps and image
alignment, crop/scale settings, label contrast, overrides, missing assets, page
changes and reconnects. Require physical preview-parity and resource checks.
Animated wallpaper and the full profile/workspace/state theme hierarchy remain
separate future work. This schedules FR-APPEAR-002/003 from the v0.4 baseline;
wallpaper is not yet an implemented V1 feature.

Later candidates from the recording review include launch-or-focus application
behavior, browser selection for website actions, multi-action sequences, and idle
dimming/sleep. Application-associated pages and Auto-Lock are detailed below.
These remain separate milestones; their presence in the recordings does not establish Fedora compatibility.

## Everyday control feedback — core implemented September 14

Implemented saved-layout target notices in the panel, amber availability warnings
on affected keys/dials, temporary red failed-action messages, explicit timeout/partial
completion wording, and automatic availability recovery checks outside the input/editor
threads. Feedback identifies the target, reason and next step while preserving draft
isolation, geometry and appearance. See [everyday control feedback](control-feedback.md)
for implemented behavior, safe user checks and validation limits.

Ongoing coverage: exercise discovery and matching with installed Zoom, Discord, Chrome, Firefox,
and other media apps as they are actually used. Cover app restart, audio-service
restart, missing devices, default changes, and simultaneous players. Keep live
status separate from configuration previews and avoid implying successful actions.

## Installation and recovery hardening

Turn the current development setup into a repeatable install/update experience.
Verify dependencies, desktop launch, service lifecycle, reconnect, configuration
backup/restore, and removal.

September 15 implementation: explicit start-at-login preference and optional
GNOME panel menu with status, Open Decksmith, and Start/Stop controls. Fedora 45
Beta reboot validation confirmed automatic background controls, active extension,
and no editor process. Native preference toggling preserved the running service.
Host reboot/icon acceptance is user-confirmed; Quit Decksmith is implemented. Broader desktop compatibility remains release work. See
[desktop integration](desktop-integration.md).

Exit criterion: documented setup and rollback can be verified on another Fedora
environment without relying on task-local build paths or settings.

September 14 implementation: relocatable development runtime bundles, per-user
versioned installation, dependency/integrity checks, preserved configuration and
icon backups, rollback and integration removal. See [installation and recovery](installation.md).
Isolated staging, checkout-hidden runtime validation, the approved live Fedora
migration and one physical Stream Deck + unplug/reconnect acceptance test are
complete. The device recovered automatically without a daemon restart and retained
saved state. The approved prior-release rollback-and-return test also passed,
preserving configuration/audio and restoring the current UI. These releases share
the same daemon, so older schema/engine rollback is not covered. A fresh Fedora44
Workstation VM now passes standalone installation, menu/service/no-device checks,
VirtualDeck rendering, backup/restore, update/rollback/removal/reinstall and reboot
persistence; see [clean-Fedora acceptance](clean-fedora-acceptance.md). The bounded
runtime installation/recovery milestone is complete. Guest physical USB permissions,
repeated reconnect and suspend/resume coverage remain separate.
Login startup stays off; native RPM/signed release packaging remains later work.

September 15: a separate fresh Fedora 45 Beta Workstation VM passed the existing
runtime installation, native editor/VirtualDeck, guest-only audio/meter, recovery,
menu-control and reboot-persistence checks. See [Fedora 45 Beta acceptance](fedora45-beta-acceptance.md).
A Fedora package-daemon shutdown delay was recorded separately. This beta sample
does not extend physical-device certification or replace final-release testing.

### Broad GNOME/Wayland distribution compatibility — V1.5 goal; V1 VM diagnostics

- Debian delivery checkpoint (October 8): separately named Debian 13 candidate
  built, signed, independently verified and installed/tested in the retained VM;
  experimental preview publication accepted. See the
  [qualification record](checkpoints/2026-10-08-debian-signed-preview.md). Continue to keep Fedora V1 assets
  and support boundaries unchanged. Full GNOME editor navigation and key/dial
  sizing accepted in the logged-in Debian VM; hardware/recovery qualification
  and broad supported GNOME Wayland compatibility remain V1.5 work.

- Ubuntu preview checkpoint (October 8): separate native Ubuntu 26.04 package
  built, signed, independently verified and qualified in the retained VM for
  installation/recovery, native editor, keyboard focus, artwork picker and
  limited Orca acceptance. Experimental preview published and fresh public
  downloads verified. See the
  [preview checkpoint](ubuntu-preview-checkpoint.md). Preserve Fedora V1 support
  and the existing Debian preview; broader certification remains V1.5 work.


V1.5 targets broad compatibility across Linux distributions running GNOME on
Wayland, alongside its plugin milestone. Start with installable Debian 13 and
Ubuntu 26.04 builds, using the V1 diagnostic VM results to identify ABI and
package gaps. Maintain a tested distribution, architecture, dependency and GNOME
version matrix. For each supported combination, verify the native editor, GNOME
extension, background service, audio and session features, USB permissions,
installation, upgrade and recovery. Document prerequisites and unsupported
combinations. "Broad" is a goal, not a claim that every GNOME version or Linux
distribution is already certified. The V1 gate requires Debian/Ubuntu VM test
results without claiming support; see the [matrix](v1-release-scope.md#current-vm-test-matrix).

V1.5 acceptance for each advertised distribution: provide a compatible binary
or native build path and dependency mapping; verify installation/upgrade/rollback,
the editor and GNOME extension on its shipped GNOME version, PipeWire/audio and
system actions, session lock/login behavior, USB permissions/reconnect, and
physical Stream Deck + use on representative hardware. A VM without USB/audio
passthrough supplies only part of that evidence. Keep a public pass/limitation
matrix, and do not silently reuse a Fedora binary that requires a newer glibc
than the target distribution provides.

### Curl-based shell installer — implemented for Fedora; cross-distribution expansion later

The published one-command Fedora installer detects Fedora/architecture and checks
dependencies and release checksums. It fails on other distributions. For V1,
finish missing-dependency and interrupted-install acceptance on a clean Fedora
test system and add release authenticity beyond checksums. Distribution-aware
package selection and native Debian/Ubuntu installation are V1.5 work.

Broader distribution support targets V1.5; the four-VM V1 test obligation is
not deferred.

### Auto-Lock and session lifecycle — initial implementation September 15

Implemented opt-in lock gating, fixed Locked artwork, stale-state protection,
queued-action invalidation, held-input cancellation, and same-page restoration.
GNOME/logind reporting is the first implementation. The user confirmed physical
lock/unlock, first-key-only Locked artwork and same-page restoration. Additional
compositor adapters and configurable appearance remain open.
See [Auto-Lock](auto-lock.md).

Extend FR-DAEMON-006/007 alongside startup/reconnect reliability: when enabled,
Auto-Lock suppresses Stream Deck actions on system lock, with configurable locked
appearance. Handle held controls/repeats, discard queued triggers, and test locked
startup/reconnect plus confirmed unlock without replaying actions. Dimming alone
must not count as locking the controls.

Target sessions that expose lock state through systemd-logind, plus dedicated
Hyprland, GNOME, KDE, and Cinnamon adapters. Detect missing/unreliable lock-state
support and show its status. Validate each session before advertising compatibility;
an unknown state must not unlock a device known to be locked. This expands existing
session-lock requirements; it is not a new implementation commitment for today.

## Optional independent master mixer

Prototype a virtual master volume stage ahead of a named output such as the
Schiit Magni. Preserve separate master and device settings. Then address explicit
stream routing, device changes, restart persistence, uninstall/disable recovery,
and behavior when Decksmith is closed.

Start with one output. Multiple outputs and Bluetooth/network-device behavior
come after the single-output path is reliable. Apps routed around the mixer will
bypass its master volume unless routing policy handles them. Keep this opt-in;
no persistent mixer routing has been installed.

## Broader product milestones

Reconcile completed prototype work against v0.4 before selecting the next major
feature: profiles/workspaces and context rules, storage migration, richer audio
mixing/soundboard, integrations/plugins, and packaging/release work.
These are future candidates, not a commitment to implement all of them next.

### Profiles, pages, and reusable actions — clarified September 13

**Automatic Page Switching (FR-PROFILE-010/011) — initial implementation September 16:**
GNOME/Wayland application-to-page assignments use desktop identities.
Select the assigned page immediately on foreground app changes, with no pause
control or sticky manual override. Unassigned apps select the chosen default page; manual
navigation lasts until the next foreground application change. Keep switching
event-driven, preserve unsaved drafts, prevent editor focus loops and respect
Auto-Lock. See [automatic application pages](automatic-pages.md) for behavior and
validation scope.

The requested support targets are GNOME via Decksmith's planned GNOME Shell
extension, Hyprland, Sway, Mangowm, KDE with kdotool installed, and all X11 desktops.
Implement adapters and dependency/status reporting; verify each target rather than
copying another product's compatibility claims. Default to application identity;
window-title matching remains a separate privacy/design decision.

**Sticky actions (FR-PROFILE-008) — V1.5 target, confirmed October 8:** expose the
existing shared binding scopes as a simple option to keep a key action in the same
position across a profile's pages.
Use one shared action/appearance/label binding, not manual copies. Show inherited
state, scope and conflicts in the integrated editor; handle edit propagation,
removal, undo/redo and import/export. Specify page-specific override/conflict behavior
before implementing the editor option, using the existing scope-resolution model.

Validate focus changes and fallback/manual navigation, plus sticky-action persistence
through automatic/manual page switches and exports. These are planned extensions
of existing requirements, grouped with profiles/context and shared controls.

### Key single-press, double-press and hold — V1.5 target, added October 9

**October 15 scope update:** unfinished delivery is deferred from the focused
multi-device V1.5 release. Requirements below remain the follow-up specification.

Provide separate **Single press**, **Double press** and **Hold** action assignments
for each key within the integrated editor. Single press and push-to-talk work
today; the core resolver already contains short/double/long-press and repeat
recognition with tests. General gesture assignments still need saved-layout,
editor and daemon integration; this is planned V1.5 work, not a published feature.

Preserve existing layouts and their action-on-release behavior. Only defer a
single press when a double-press assignment is configured; a recognized double
press suppresses that sequence's single action. An assigned Hold action fires
once after its threshold and suppresses single/double actions for that hold.
Start from the architecture's 300 ms double-press window and 500 ms hold threshold,
and explain the timing inline. Do not enable automatic repeat by default;
general hold-repeat configuration is a separate follow-on decision.

Keep push-to-talk as a distinct held action: activate on press and release safely
when the last overlapping hold ends. Prevent incompatible gesture combinations
from unintentionally opening a microphone or firing multiple actions. Cancel
pending gestures and held state on lock, page/binding changes, disconnect,
suspend and service stop; recovery must not replay stale inputs. Retain existing
action confirmations, unavailable-target handling and plugin permissions.

Store assignments without losing custom labels/artwork or existing action data.
Support undo/redo, import/export and sticky-key inheritance. Reuse monotonic
deadline handling with bounded work rather than adding busy polling. Coordinate
the semantics with Dial Stacks and Action Wheels without changing existing dial
press or touch behavior implicitly.

Acceptance covers timing boundaries, quick/slow double presses, duplicate input
reports, long-hold suppression, release/cancellation and push-to-talk overlap;
saved-layout migration/round trips; inline editor accessibility; and physical
Stream Deck + and MK.2 key tests as hardware becomes available. Qualify idle and
loaded resource use before release.

### Simultaneous multiple-device support — clarified September 13

**October 13 integration checkpoint:** the explicit
[workspace lab](profile-editor-lab.md) now uses one native editor with independent
layout/Home settings drafts, stable model geometry, owned name/brightness saves
and late-preview fencing. [Verified history archives](profile-history.md) retain
older generations before explicit pruning, with journaled recovery and unchanged
active device contexts. The [physical discovery/transport foundation](hardware-discovery.md) now adds a
bounded private inventory, verified model opener/shared cooperative lease and live
enrollment without disturbing other sessions. All fifteen MK.2 key reports and
model-sized images are fixture-qualified. Production service/lock/roster/backend
wiring, async disk and archive quota/policy, full paired lifecycle and resource
qualification remain required; this does not enable public multi-device support.

**October 13 physical checkpoint:** the guarded
[MK.2/Plus trial](checkpoints/2026-10-13-physical-mk2-pair.md) confirmed all fifteen
MK.2 keys and orientation, independent paired pages/brightness and Plus strip/dial
feedback. MK.2 removal/recovery was isolated while Plus input continued. Both tests
restored the original Plus page/build and unchanged configuration. The resumed
checks also passed Plus removal/recovery with MK.2 continuing, Plus-only swipes,
four dial pushes/turns and isolated touch input. Physical reversed connection
order, paired lifecycle, shared action backends and resource qualification remain
open; multi-device control is not installed or released.

**October 13 saved-workspace checkpoint:** the opt-in
[saved-device workspace](device-workspace.md) now connects real read-only discovery,
explicit private profile enrollment/reopening and one native device chooser/editor.
Plus import retains the original V1 document; MK.2 has its own defaults. Offline
saves and selection preserve other drafts, and status separates detection from
physical readiness. This endpoint can coexist with V1 and does not claim HID or
execute actions. Shared backends, live lock/feedback/display scheduling and physical
production qualification are still required before activation. The subsequent
[restart/pressure checks](checkpoints/2026-10-13-device-workspace-hardening.md) fixed
endpoint replacement and bus reply starvation; twenty restarts and bounded
concurrent requests now pass. The eight-device five-minute stress also passed.

**October 13 connected-execution checkpoint:** the supplied-adapter
[execution runtime](device-execution.md) now routes cached model-specific input to
shared action/microphone workers and per-device render/I/O owners. Context fences
cancel stale effects/feedback, same-target writes retain serialization, microphone
holds expire when their I/O owner stalls, and metering can update only the Plus
strip. Native provider resolution, live API/startup/roster integration and physical
paired qualification remain activation gates; installed V1 stays unchanged.

**V1.5 release requirement — confirmed October 9:** deliver simultaneous device
control, initially qualified with Stream Deck + and the user's MK.2.
This supersedes the earlier separately scoped multi-device milestone. Complete
the MK.2 adapter/geometry and physical acceptance needed for that pair; broader
model certification remains independently tracked in the hardware matrix.

Implement the existing FR-DEVICE-007 end to end: stable identity for each unit
(including identical models and ambiguous/missing serial handling), a named device
selector with connection status, per-device layout/profile assignments and
preferences, and independent simultaneous input/rendering/navigation. Restore the
correct assignments across hot-plug, reversed connection order and suspend/resume;
isolate failures and stale session events to the affected device. Shared desktop
audio/app targets remain shared resources. Preserve the existing single-device
layout during migration; bind configuration to stable device identity rather
than USB enumeration order. Keep Choose Device and per-device editing inside the
main interface, showing each model's actual controls. Never silently discard keys
or bind nonexistent dials when importing a layout onto a different model.

Validate at least two VirtualDecks, including identical and mixed models, and
perform the Plus/MK.2 two-device physical check before V1.5 release. Exercise
independent page changes, simultaneous inputs/rendering, reversed reconnect order,
single-device failure, lock, suspend/resume, quit/stop blanking and saved assignment
recovery. Test ambiguous identities explicitly; never silently attach another
device's layout. Measure CPU/memory with both devices active, share common audio
subscriptions where possible and bound per-device caches/queues. Physical
identical-model certification remains a separately recorded hardware check.
This is planned V1.5 work, not a claim that published V1 controls multiple devices.
Details are in
FR-DEVICE-007 and architecture sections 5, 6.1 and 10.2.

### Complete current physical model range

FR-DEVICE-012 and the [hardware support matrix](hardware-support-matrix.md) now
name the current catalog, variants, control differences and phased acceptance gates.
Generalized geometry and per-device identity/state are prerequisites for broad model
support; existing Plus-first certification remains the first release boundary.
Network Dock, partner/embedded integrations and mobile/software products have explicit
scope questions rather than assumed support. This planning update does not implement
additional devices or claim existing Linux compatibility.

## Completed reference

See the [September 12 checkpoint](checkpoints/2026-09-12.md) for confirmed behavior,
validation evidence, and current limits, and the [user guide](USER_GUIDE.md) for
how the implemented controls work.

### System key actions — September 17

Implemented inline System control assignments for Lock, DND, immediate Night Light, power profiles, Suspend, explicit-confirmation Reboot/Shutdown, and Bluetooth radio toggling. Automatic labels/artwork remain customizable; observed state indicators share the hardware/preview renderer. See [System actions](system-actions.md).

Fedora 44 VM acceptance covers desktop lock, notification toggles, immediate Night Light, supported profiles, and confirmed restart. Suspend reaches sleep and resumes, but the VM virtual display required a guest reboot afterward. Host Bluetooth, Lock, DND, Night Light, and power confirmation dialogs are user-confirmed. The VM correctly reports Bluetooth unavailable without passthrough. A host suspend exposed a blank Stream Deck after resume; resume-time device reopening/full redraw is implemented with regression coverage, and the user confirmed successful physical suspend/resume recovery on September 17. The subsequent host reboot and updated icons were also user-confirmed. Further desktop adapters and Bluetooth pairing/device management remain future work.

## Editor polish — September 17 accepted checkpoint

Dial settings use Assignment, Behavior and collapsed Appearance groups. Scope labels identify shared defaults versus page overrides. Layout appearance is explicitly shared across pages. The editor shows page/control context and unsaved-preview state, with local guidance for common incomplete key fields. About shows the installed release and offers a copyable diagnostic summary excluding personal layout contents. App audio targets support embedded 32-pixel installed artwork with generic fallback and retained mute slash. Existing assignments gain artwork through Refresh application icon or selecting the target again, then Save and Apply.

The user confirmed the key/dial sizing correction and successful application-icon refresh. Touch-strip titles are left-aligned with consistent icon spacing; muted targets retain a red slash and red Muted text without a border, input Live readouts are green, and No Audio uses the configured readout typography without a yellow outline. Sustained resource profiling, high-DPI/accessibility coverage and repeated lifecycle testing are still V1 work.

### V1 usability and accessibility pass — added September 22

Use the question.design component library and theme previews as design references.
Implement relevant patterns with existing GTK/libadwaita controls; do not introduce
Next.js, React, Tailwind or a web runtime into the desktop app for these changes.
GNOME light/dark appearance and accessibility settings remain authoritative.

- **Form feedback:** place actionable validation beside the affected field and
  explain why Save and Apply is unavailable. Preserve custom labels and drafts;
  asynchronous discovery and status polling must not steal focus, move the cursor,
  overwrite typed values or reset a text selection.
- **Consistent status:** use coherent wording and presentation for working,
  successful, muted, silent, unavailable and failed states. Pair color with text
  or symbols, reserve disruptive confirmations for consequential actions, and
  keep recovery guidance inline where possible.
- **Keyboard access and tooltips:** verify logical tab order, visible focus,
  accessible control names, keyboard activation, and useful concise tooltips.
  Essential instructions must remain available without hovering.
- **Contrast and typography:** review muted section colors, labels, status text,
  disabled controls and device previews under light, dark and high-contrast
  settings. Check enlarged text, high-DPI scaling and longer labels without
  weakening the accepted red Muted/crossed-out icon and green Live conventions.
- **Geometry and spacing:** standardize group spacing, field alignment and action
  placement. Preserve device aspect ratio and the stable footer while switching
  tabs, keys/dials, expanding sections or receiving background updates. Keep as
  much editing as practical inside the main interface.

Acceptance: document representative keyboard-only workflows, light/dark and
high-contrast checks, high-DPI/enlarged-text checks, and invalid-field recovery.
Exercise save/cancel, key/dial switching and discovery/status refresh while typing.
Verify no draft loss or layout shift; compare idle/active CPU and memory with the
recorded baseline. These are planned checks, not claims of completed accessibility
certification. Keep changes small and review the resulting native UI with the user.

September 22 VM checkpoint: the native regression passed on Fedora 44 / GNOME
50.4 at actual 125% light, 150% dark and 150% high-contrast display scaling.
A compact run with Orca active also passed, with named controls in AT-SPI and
representative announcements in Orca's speech log. A human-paced screen-reader
listen-through remains open. See [accessibility results and limits](accessibility-checkpoint-2026-09-22.md).
Use the testing VM wherever practical; restore temporary preferences and shut it
down after testing, keeping autostart disabled.

### Follow-on editor candidates — after the V1 polish pass

- **Narrow windows with enlarged text:** improve horizontal adaptation of the
  shared editor/preview at 1024-pixel width. The signed candidate's VM fixture
  passes ordinary text at 1024 and enlarged text at 1280; the combined compact/
  enlarged case requests 1070 pixels and clips. Retain the wider-window guidance
  until a fix is packaged and qualified; do not call the failed combination a pass.

- **Searchable navigation and actions:** evaluate a keyboard-accessible search for
  pages, actions, settings and assigned devices. Coordinate it with the integrated
  editor's planned grouped action catalog rather than adding competing searches.
  Keep destructive actions behind their existing confirmations.
- **Artwork drag-and-drop:** extend local artwork selection with a preview,
  clear supported-format/size guidance and actionable validation. Reuse current
  image decoding limits and preserve custom labels, artwork and undo/redo.
- **Optional display density:** assess compact versus comfortable spacing only
  after stable geometry is verified. Preserve readable text, usable click targets,
  device proportions and GNOME scaling; keep the accepted layout as the default.

Reference resources reviewed September 22:
- [question.design component library](https://www.question.design/design-system/docs):
  forms, alerts, tooltips, command menu, file upload and dashboard-density examples.
- [question.design Theme Generator](https://www.question.design/design-system/theme-generator):
  palette and component-state exploration; adapt to GNOME rather than importing
  fixed brand themes.

The published library targets shadcn/ui, Next.js and Tailwind. Individual component
pages and the linked source repository were not all retrievable during review.
Treat it as a pattern reference; verify availability and reuse terms before copying
code/assets. Decorative continuous animation and unrelated AI/chat surfaces are
not part of this Decksmith usability milestone.

## V1 performance baseline — September 17

The first bounded resource pass completed 100 isolated page changes with responsive previews and near-flat final warmed editor memory. Background service CPU averaged 44.7% of one core, with a separate attribution sample identifying short-lived helpers as the main contributor. Prioritize batched/persistent audio-state queries, then repeat measurements; sustained four-source testing and accessibility/lifecycle checks remain open. See [performance baseline and limitations](performance-checkpoint-2026-09-17.md).

### Reviewer distribution checkpoint — September 22

The published preview.2 installer passed fresh-user installation, preview.1 upgrade,
rollback, retained-data uninstall and private-bus VirtualDeck checks in Fedora 44.
See [reviewer readiness](reviewer-readiness-2026-09-22.md). Missing-package installation
and a human-paced screen-reader listen-through remain explicit limits. The
compact-editor fix was subsequently packaged in preview.3. The later validation
speech change and rounded launcher icon are in source/installed development state;
they require a new tested download before being advertised as release features.

### Validation speech follow-up — September 22

Implemented debounced validation announcements with no focus movement or repeated
status-poll speech. Orca in Fedora 44 confirmed the empty-label reason was spoken;
147 editor tests passed. Keep full human screen-reader review on the V1 checklist.
The change is in development and is not part of the existing preview.3 download.
