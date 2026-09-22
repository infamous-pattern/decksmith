# Decksmith remaining roadmap

This is the implementation roadmap as of September 22, 2026. The v0.4 product
and architecture documents remain the broader design baseline. Items below are
planned work unless explicitly marked implemented; this is not authorization to
change system routing.

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
   acceptance. General plugin delivery stays V1.5; five-language localization V2.

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

User priority for the next project session, September 17: work through these tasks
before taking on other feature work. Plugin delivery remains V1.5; these are
focused pre-V1 preparations to avoid breaking saved layouts or rebuilding working
execution paths later. Use the September 22 sequence above for the next session; these remain the architectural acceptance gates.

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
V1.5 work; the experiment does not establish general compatibility.

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

### Plugin support — V1.5 target

User decision, September 17: plugin delivery targets **V1.5**, after V1 stability,
performance and core-device acceptance. This supersedes the original baseline's
V1 plugin target; it is a release milestone, not a promised calendar date.

Deliver the native plugin foundation first:

- Versioned native contract for actions, live state and key/dial/touch-strip controls.
- Separate plugin processes with bounded communication, crash/hang recovery and
  clear diagnostics; built-in controls must continue working after plugin failure.
- Manifest/runtime validation, local installation, enable/disable and removal.
- Declarative settings shown within the main GTK editor, plus clear capability
  and permission information. Do not describe process isolation as a full sandbox.
- At least one reference plugin and developer documentation, with acceptance for
  installation/removal, missing dependencies, crashes, restarts and resource use.

**OpenAction is the first V1.5 compatibility target.** This means implementing
Decksmith's host-side adapter, not embedding the marketplace website or assuming
that the plugin-author SDK supplies a host. Follow this delivery order:

1. Complete the native host's isolation, lifecycle and bounded-update foundations.
2. Implement and document a supported OpenAction protocol subset. Validate it
   first with Counter and System Information, then a selected Linux-compatible
   plugin exercising dial/touch behavior. These are test candidates, not current
   compatibility claims. Verify plugin versions, settings UI, missing runtimes,
   device feedback, failure recovery and resource use.
3. After the host passes those checks, add OpenAction catalog browsing within the
   Plugins tab as a follow-on V1.5 target. Decksmith manages installation and
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

V1.5 also targets an explicitly documented, tested compatibility subset for open,
unprotected Stream Deck plugins: Linux-native executables first, then compatible
Node.js plugins. Compatibility requires its own bridge and isolated WebKitGTK
settings support where needed; ship only verified combinations and explain
unsupported requirements. Native support is the first delivery gate, not evidence
that arbitrary existing plugins work.

A Decksmith-owned marketplace/catalog, Windows plugins through Wine, and broader sandbox policies
are later work, not V1.5 release requirements. DRM bypass remains out of scope.
Full five-language localization remains scheduled separately for V2.

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

### 4. Appearance presets and local icon library — implemented September 14

Implemented layout-wide typography/color/label defaults, key and dial overrides,
Reset to theme, compact per-key icon scaling (10–100% in five-point steps), and Maker’s Mark, Dark, Light and High Contrast presets. The offline
library includes 134 pinned official Tabler icons, searchable local imports and a
link to download the full official set. Renderer-backed key/touch previews, draft
history and theme/layout import/export preserve actions. See
[appearance and icon behavior](appearance-and-icons.md) and the
[September 14 checkpoint](checkpoints/2026-09-14.md) for validation and limits.
The broader v0.4 theme hierarchy and panoramic canvases remain planned. A future
visual iteration should offer richer, less generic theme presets; the current
controls and presets are accepted for now.

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

### Broader GNOME/Wayland distribution compatibility — future

Goal: support Linux distributions running GNOME on Wayland beyond Fedora. This
is a compatibility target, not a claim of universal support today. Maintain a
validated distribution, architecture, dependency and GNOME-version matrix;
verify native UI, extension compatibility, audio/session services, USB access,
installation and recovery on each advertised combination. Document prerequisites,
known limitations and unsupported combinations before claiming support.

### Curl-based shell installer — future

Provide a documented curl-command installation path backed by an inspectable
shell installer. Detect distribution, architecture, session and dependencies;
select a compatible release and verify its integrity against trusted release
metadata before installation. Fail clearly on unsupported platforms, missing
dependencies, download failures or verification failures, with actionable recovery
steps and no partial activation. Reuse the existing configuration-preserving
installation/update, backup, rollback and removal workflow; validate fresh installs,
repeat runs, upgrades and interrupted-install recovery on supported platforms.

Both milestones are deferred; neither selects or replaces the next feature task.

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

**Sticky actions (FR-PROFILE-008):** expose the existing shared binding scopes as a
simple option to keep a key action in the same position across a profile's pages.
Use one shared action/appearance/label binding, not manual copies. Show inherited
state, scope and conflicts in the integrated editor; handle edit propagation,
removal, undo/redo and import/export. Specify page-specific override/conflict behavior
before implementing the editor option, using the existing scope-resolution model.

Validate focus changes and fallback/manual navigation, plus sticky-action persistence
through automatic/manual page switches and exports. These are planned extensions
of existing requirements, grouped with profiles/context and shared controls.

### Simultaneous multiple-device support — clarified September 13

Implement the existing FR-DEVICE-007 end to end: stable identity for each unit
(including identical models and ambiguous/missing serial handling), a named device
selector with connection status, per-device layout/profile assignments and
preferences, and independent simultaneous input/rendering/navigation. Restore the
correct assignments across hot-plug, reversed connection order and suspend/resume;
isolate failures and stale session events to the affected device. Shared desktop
audio/app targets remain shared resources.

Validate at least two VirtualDecks and perform a two-device physical check before
claiming simultaneous-device certification. This clarifies existing design intent;
it is planned work, not a claim that the current single-device prototype supports
it or an expansion of the V1 hardware-model certification promise. Details are in
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

### Follow-on editor candidates — after the V1 polish pass

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
