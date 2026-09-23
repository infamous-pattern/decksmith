# Product Requirements Document

**Project:** Decksmith   
**Document status:** Architecture-frozen baseline for Phase 0  
**Version:** 0.4  
**Reference platform:** Fedora Workstation 44 / GNOME / Wayland  
**Reference hardware:** Elgato Stream Deck +  
**Date:** September 2026

---

## 1. Executive summary

Decksmith — **Open Control, Forged for Linux.** — is a Linux-native control application for Elgato Stream Deck hardware. The initial release is optimized for Fedora Workstation and the Stream Deck +, with the goal of delivering a product that is visually polished, reliable enough for daily use, and deeply integrated with the Linux desktop.

The product is intentionally different from existing alternatives in three ways:

1. **Linux is the primary design target.** GNOME, Wayland, PipeWire, systemd, D-Bus, desktop entries, and other Linux APIs are first-class design inputs rather than compatibility afterthoughts.
2. **The Stream Deck + is the reference device.** Dials and the LCD touch strip are core interaction surfaces, not optional extensions to a key-only design.
3. **The application is split into a durable background service and a native configurator.** The hardware continues to work when the configuration UI is closed.

The initial product will not require a cloud account, telemetry, proprietary server, or online marketplace. Core capabilities must remain useful without third-party plugins.

---

## 2. Product vision

### 2.1 Vision statement

Create the best Linux desktop experience for Stream Deck hardware: fast, native, local-first, polished, secure, automation-friendly, deeply customizable, context-aware, and open source.

### 2.2 Product principles

1. **Linux native over cross-platform convenience.** Prefer native platform APIs when they provide materially better reliability or UX.
2. **Local first.** Device configuration and execution work without internet access.
3. **No telemetry by default.** The application performs no analytics or tracking unless a future opt-in mechanism is explicitly introduced.
4. **Useful without plugins.** Core Linux workflows must be available as built-in capabilities.
5. **Instant feedback.** Configuration changes are reflected on the hardware immediately.
6. **Crash isolation.** A plugin failure cannot take down the device daemon.
7. **Explicit permissions.** Dangerous capabilities such as input injection and command execution are clearly represented and constrained.
8. **Polish is a release criterion.** Accessibility, onboarding, visual consistency, diagnostics, and recovery are part of the definition of done.
9. **Capability driven.** Model commands, adjustments, stateful controls, navigation, and live data as reusable capabilities rather than device-specific button actions.
10. **Context aware.** Profiles, workspaces, pages, and temporary context layers may change what controls do without duplicating capability implementations.
11. **Customization first.** Backgrounds, themes, icons, typography, control treatments, and state styling are first-class product features.
12. **Behavior and appearance are separate.** Functional mappings and visual styling must be independently reusable, exportable, replaceable, and shareable.
13. **Open protocols where practical.** Internal APIs should be documented enough for community tools and integrations.
14. **Avoid unnecessary Elgato coupling.** Compatibility is valuable, but the project must not depend on private or DRM-protected Elgato services to remain useful.
15. **Deterministic interaction semantics.** Short press, double press, long press, hold/repeat, raw press/release, dial gestures, and touch gestures must have documented conflict-resolution and timing behavior.
16. **Test without losing hardware fidelity.** A VirtualDeck must enable deterministic development and CI, while physical hardware-in-loop tests remain release gates for behavior that simulation cannot prove.
17. **Secure when unattended.** Session lock and idle state must be able to disable or visually transform the device without relying on the configurator being open.
18. **Always available, never intrusive.** The runtime starts with the graphical login and remains useful without Decksmith Studio open; quick access is available from the desktop panel/tray while the heavy configurator stays closed.

---

## 2.3 Product naming and terminology

The standardized product identity is:

- **Decksmith** — product/project
- **Decksmith Studio** — native configuration application
- **decksmithd** — background service
- **decksmithctl** — CLI
- **The Forge** — creation/editing workspace inside Studio
- **Blueprints** — reusable profile/templates
- **Decksmith Foundry** — future community catalog
- **Tagline:** **Open Control, Forged for Linux.**

Public documentation may describe compatibility with Elgato Stream Deck hardware, but “Stream Deck” is not part of the Decksmith product name.

## 3. Target users

### 3.1 Primary user

A Linux desktop user with one or more Stream Deck devices who wants reliable native control of applications, audio, automation, development tools, and workflows.

### 3.2 Initial reference persona

- Fedora Workstation user
- GNOME on Wayland
- Stream Deck + connected by USB
- Uses shell commands, browsers, media applications, communication tools, development tools, and homelab services
- Values local control and open-source software
- Expects professional desktop UX rather than a hobby-style configuration utility

### 3.3 Secondary users after 1.0

- KDE Plasma users
- Stream Deck MK.2, XL, Mini, Neo, and newer compatible Elgato models
- Debian/Ubuntu, Arch, openSUSE, and other desktop Linux distributions
- Users who need existing open/unprotected Stream Deck plugins
- Developers who want a CLI/D-Bus API for automation

---

## 4. Supported environment

### 4.1 Reference environment for V1

- Fedora Workstation 44 is the supported V1 reference. Fedora 45 final-release
  support requires a separate final-release pass; the existing Fedora 45 Beta
  VM is a forward-compatibility test, not that certification.
- GNOME
- Wayland
- x86_64
- PipeWire/WirePlumber
- systemd user session
- Stream Deck +

The V1 candidate must also be tested in the retained Fedora 45 Beta, Debian 13
and Ubuntu 26.04 GNOME VMs. Debian and Ubuntu results are diagnostic, not support
claims; the required cases and pass rules are in the [V1 release scope and VM
matrix](v1-release-scope.md). Broad supported GNOME/Wayland distribution
compatibility is a V1.5 goal, starting with Debian and Ubuntu. Each advertised
distribution/GNOME combination requires its own compatibility validation.

### 4.2 V1 hardware scope

**Required:**

- Stream Deck +

**Planned beyond the initial V1 certification; not yet claimed as supported:**

- Stream Deck MK.2
- Stream Deck XL
- Stream Deck Mini
- the complete current physical Elgato Stream Deck range, including variant-specific controls, as tracked in the [hardware support matrix](hardware-support-matrix.md); library coverage shall not silently narrow the product target

The core device abstraction must not hard-code a single 2x4 layout even though the Stream Deck + is the first certified device.

---

## 5. Competitive/reference baseline

The product should cover the most important expectations established by current Stream Deck software while deliberately implementing Linux-native equivalents where appropriate.

Baseline expectations include:

- customizable keys
- dials and encoder capabilities
- touch-strip feedback
- profiles
- application-aware profile switching
- pages
- folders
- multi actions
- key logic (short press, double press, long press/hold, hold/repeat, and raw press/release where appropriate)
- dial stacks
- action wheels or equivalent dial menus
- custom titles and images
- built-in system capabilities
- third-party plugin support
- import/export and sharing of configuration
- brightness and device preferences

Elgato currently supports plugin resources and a newer SDK generation that continues to evolve. Compatibility must therefore be versioned, tested, and treated as a subsystem rather than allowing SDK details to leak into the core capability model.

Additional product ideas adopted from the Logitech MX Creative Console and Loupedeck software model include:

- application-aware contexts and temporary dial context layers
- task-oriented workspaces inside profiles
- global/profile/workspace/page control scopes
- dedicated Control Views for adjustments and rich stateful controls
- dynamic providers that enumerate live entities such as audio streams, Home Assistant entities, OBS scenes, or Proxmox guests
- reusable workflow/profile templates
- SVG-first visual assets and theme inheritance
- multi-state controls rather than Boolean toggles only
- panoramic backgrounds spanning the full key grid plus independent touch-strip backgrounds

The cursor-centered Quick Palette concept is architecture-ready but intentionally deferred until after 1.0 so it does not compete with core device quality.

Additional Linux-native engineering ideas reinforced by StreamController include a first-class VirtualDeck for development/CI, machine-readable CLI and input emulation, session-lock/idle handling, event-driven plugin integration, and explicit hardware-in-loop testing. These are implemented within Decksmith's Rust daemon/service architecture rather than by copying StreamController's GPL implementation.

---

## 6. V1 scope

### 6.1 V1 must include

The September 23 [V1 release scope and acceptance plan](v1-release-scope.md)
supersedes the original all-features V1 list while preserving the broader product
requirements below. The supported V1 must provide:

- reliable single Stream Deck + key, dial and touch-strip operation, brightness,
  disconnect/reconnect and suspend/resume on the Fedora reference desktop;
- the integrated GTK editor with pages, application-based page switching, shared
  dial defaults/page overrides, saved layout and appearance editing, import/export,
  visible state/error feedback and stable Save and Apply behavior;
- the currently implemented built-in application/website, audio, media and system
  actions with safe unavailable-state reporting where a target is absent;
- a background service, optional start at login, GNOME indicator, Auto-Lock and
  safe stop/quit/lock behavior;
- versioned saved-data compatibility, backup, recovery and rollback without loss
  of existing user layouts, custom text, artwork or assignments;
- reproducible VirtualDeck, Fedora VM and physical-device acceptance plus the
  four-VM diagnostic matrix, accessibility review and security/release gates in
  the linked plan.

The original v0.4 list included advanced profile/workflow, storage, capability,
appearance and packaging features not yet implemented. Their deferrals are mapped
in the linked plan; they must not be described as V1 capabilities. General plugin
delivery moved to **V1.5** by the user on September 17, 2026. Broad GNOME/Wayland
distribution compatibility is also a **V1.5 goal** as of September 23. The plugin
requirements for that milestone are:

- plugin architecture
- dynamic capability-provider API for native integrations/plugins
- declarative native plugin settings rendered by GTK
- compatibility with a documented subset of unprotected Stream Deck plugins
- WebKitGTK-hosted property inspectors for compatible plugins

### 6.2 Later candidates, not V1 release gates

The following ideas remain in the product plan. They should not expand or delay
the focused V1 unless separately prioritized after the release gates pass:

- Home Assistant built-in integration
- SSH capability
- NetworkManager capabilities
- Bluetooth/BlueZ capabilities
- icon library management
- visual theme presets
- profile templates
- workflow templates
- first-run profile examples
- optional plugin sandboxing with bubblewrap — deferred beyond V1; not a V1.5 release requirement

### 6.3 Explicit V1 non-goals

- replicating Wave Link itself
- cloud account/sync
- remote device fleet management
- mobile companion application
- Windows or macOS builds
- a proprietary plugin marketplace
- guaranteed operation of Windows-only plugins under Wine
- guaranteed compatibility with DRM-protected Elgato Marketplace plugins
- reverse engineering Elgato DRM or authentication
- Flatpak as the reference packaging format
- every Stream Deck hardware model certified at launch
- cursor-centered Quick Palette in 1.0 (architecture-ready for a later release)
- continuously animated panoramic backgrounds in 1.0

---

## 7. Functional requirements

### FR-DEVICE — Device and HID management

**FR-DEVICE-001** The daemon shall discover attached supported Stream Deck devices without requiring root privileges after installation of project udev rules.

**FR-DEVICE-002** The system shall identify device model, serial number, and firmware where exposed by the hardware API.

**FR-DEVICE-003** The daemon shall reconnect automatically after USB unplug/replug.

**FR-DEVICE-004** The daemon shall recover from system suspend/resume without requiring the configurator to be relaunched.

**FR-DEVICE-005** One device worker shall own each physical Stream Deck connection so blocking HID behavior cannot stall the core event loop.

**FR-DEVICE-006** Hardware communication errors shall be surfaced through structured diagnostics and the UI without crashing the daemon.

**FR-DEVICE-007** The system shall support simultaneous operation of multiple supported Stream Deck devices attached to the same Linux system, including multiple units of the same model, even if V1 certification focuses on one Stream Deck +. This includes:

- Persistent identification of each physical unit, using model/vendor/product and serial information where reliable; transient USB paths or enumeration order shall not determine saved assignments. Missing or duplicate serials shall be handled explicitly, with user-assisted association when identity is ambiguous rather than silently assigning another unit's configuration.
- A device selector in Studio showing a friendly name, model, distinguishing identity and connection status. Users shall be able to name each unit and choose/configure its assigned layout/profile and device preferences, including while it is disconnected.
- Independent active profile/workspace/page, input routing, rendering, brightness and device lifecycle for each unit. Selecting a device in the editor shall not switch or pause other devices, and editing/applying a device assignment shall target that unit. If a reusable profile is shared, the UI shall make the scope of edits clear.
- Hot-plug/reconnect and suspend/resume recovery that restores each identified unit's assignment and preferences regardless of attachment order, without swapping configurations or disrupting other connected units. Disconnected assignments remain available for later reconnection, and stale events/commands from an old connection shall not affect a new session.
- Verification with at least two VirtualDeck instances, including identical-model identity and reversed reconnect order, plus a physical two-device check before claiming simultaneous-device certification. One device's failure shall not stop healthy devices.

Independent device operation does not imply separate copies of shared desktop resources: two controls intentionally targeting the same application or audio output still control that same resource.

**FR-DEVICE-008** The device abstraction shall provide a VirtualDeck implementation capable of representing the Stream Deck + without USB hardware.

**FR-DEVICE-009** VirtualDeck shall use the same control geometry, renderer output contracts, input normalization, trigger resolution, and capability-binding paths as physical devices wherever practical.

**FR-DEVICE-010** Automated tests shall be able to create multiple VirtualDeck instances, connect/disconnect them deterministically, inject raw input, and inspect rendered output/state.

**FR-DEVICE-011** Physical hardware-in-loop tests shall remain required for release-critical HID, touch, dial, reconnect, suspend/resume, and rendering behavior that a VirtualDeck cannot validate.

**FR-DEVICE-012** The product shall target all currently available physical Elgato Stream Deck models and variants in the dated [hardware support matrix](hardware-support-matrix.md). Coverage shall include each model's actual key, display, dial, touch, pedal and indicator capabilities, capability-aware configuration/import behavior, and model-specific validation. Track implementation, simulation, hardware testing and release certification separately; refresh the catalog before releases. Retain the Plus-first V1 certification scope while phasing complete-range delivery. Mobile/virtual products, partner integrations and transport accessories require explicit scope decisions as recorded in the matrix.

### FR-PLUS — Stream Deck + interaction

**FR-PLUS-001** All eight LCD keys shall support key-down and key-up events.

**FR-PLUS-002** All eight keys shall support independently rendered icons, titles, and state-specific appearance.

**FR-PLUS-003** All four dials shall report clockwise/counterclockwise rotation and push/release events.

**FR-PLUS-004** The touch strip shall support full-width rendering and partial-region updates when supported by the device API.

**FR-PLUS-005** Touch gestures exposed by the hardware protocol, including tap, press/hold, and flick, shall be mapped into normalized application events.

**FR-PLUS-006** Dial feedback widgets shall support at minimum: icon, title, value, progress/bar representation, state color/background token, and optional custom raster content.

**FR-PLUS-007** Dial stacks shall allow multiple actions to be cycled on one encoder.

**FR-PLUS-008** The product shall provide a dial menu/action-wheel equivalent that can select and invoke multiple actions from one dial slot.

### FR-TRIGGER — Input gestures and trigger resolution

**FR-TRIGGER-001** Physical input shall be normalized into raw device events before semantic gesture recognition. Raw events shall include key down/up, dial push down/up, dial rotation ticks/direction, and touch events exposed by the hardware protocol.

**FR-TRIGGER-002** Key bindings shall support short press, double press, long press, hold/repeat, and direct raw press/release bindings for advanced use cases such as push-to-talk or press-and-release workflows.

**FR-TRIGGER-003** Dial push bindings shall support short push, double push, long push, and optional hold/repeat in addition to independent clockwise/counterclockwise rotation.

**FR-TRIGGER-004** Touch bindings shall support the semantic gestures exposed by the Stream Deck + protocol, including tap, long press/hold, and flick left/right. Continuous drag/repeat behavior shall not be promised unless the device protocol exposes sufficient events.

**FR-TRIGGER-005** Gesture recognition shall use monotonic time and documented timing defaults. Global defaults shall be user-adjustable, and per-binding overrides may be exposed as advanced settings.

**FR-TRIGGER-006** When a double-press binding exists, the first short press shall be deferred until the double-press window expires. If the second press completes in time, the double-press binding shall execute and the deferred short-press binding shall be suppressed.

**FR-TRIGGER-007** A recognized long press shall suppress the short-press/double-press gesture for that press sequence. Hold/repeat shall begin only after the configured hold threshold/repeat delay and shall stop immediately on release.

**FR-TRIGGER-008** Raw press/release bindings shall be treated as independent edge-triggered behavior. The configurator shall warn when raw-edge bindings are combined with gesture bindings in a way likely to surprise the user.

**FR-TRIGGER-009** Default interaction timing shall be polished for ordinary desktop use and shall include configurable long-press threshold, double-press window, hold-repeat delay, and hold-repeat rate.

**FR-TRIGGER-010** The event monitor, diagnostics, VirtualDeck, and test harness shall be able to display or generate both raw physical events and resolved semantic triggers.

### FR-DAEMON — Background service

**FR-DAEMON-001** A `systemd --user` service shall own hardware access, profiles, capability execution, and plugin supervision.

**FR-DAEMON-002** Closing the configurator shall not interrupt device operation.

**FR-DAEMON-003** The daemon shall expose a versioned D-Bus API.

**FR-DAEMON-004** The daemon shall publish state-change signals for device, profile, workspace, page, capability, context, appearance, and plugin events.

**FR-DAEMON-005** `decksmithd` shall start automatically with the user’s graphical login by default after Decksmith is installed/activated. The user may disable “Start Decksmith at login” in Preferences without disabling manual startup.

**FR-DAEMON-006** The daemon shall provide **Auto-Lock**, subscribing to desktop/session lock state without depending on the configurator. The support target is any session where systemd-logind reports lock state, with dedicated adapters planned for Hyprland, GNOME, KDE, and Cinnamon. Detect adapter availability and explain unavailable lock detection; this is planned coverage requiring per-session validation, not a claim of current support.

**FR-DAEMON-007** Users shall be able to enable Auto-Lock and choose a locked appearance, including dimming/turning off displays where supported. When Auto-Lock is enabled and the session locks, suppress device-triggered action execution regardless of the chosen appearance. Cancel held/repeating actions safely, discard pending triggers rather than replaying them on unlock, and restore normal controls only after a confirmed unlock. Reconnects and daemon restarts while locked must preserve suppression; an unknown state must not be interpreted as an unlock. Display dimming alone does not satisfy Auto-Lock.

**FR-DAEMON-008** Idle/screensaver presentation shall use the normal renderer/theme pipeline so active, idle, and locked visual states remain consistent and testable.

**FR-DAEMON-009** Decksmith Studio shall not auto-open at login. The background runtime, device control, plugins, profile switching, and quick-access indicator shall operate without the Studio process running.

**FR-DAEMON-010** The daemon shall remain low-overhead when no compatible hardware is attached and shall continue monitoring for hot-plugged devices.

**FR-DAEMON-011** Login startup shall be implemented with native user-session mechanisms, preferring a `systemd --user` unit associated with the graphical session on the Fedora/GNOME reference platform. Packaging may provide a standards-based fallback for desktops that do not reliably activate the graphical user target.

### FR-INDICATOR — Background presence and quick access

**FR-INDICATOR-001** While Decksmith is running, the user shall have a lightweight desktop indicator that provides one-click access to Decksmith Studio.

**FR-INDICATOR-002** On GNOME, the Decksmith GNOME Shell companion shall provide the panel indicator/menu directly so the Fedora reference experience does not require a third-party AppIndicator/tray extension.

**FR-INDICATOR-003** On desktops that implement StatusNotifierItem/AppIndicator-style system trays, Decksmith shall expose a standards-compatible status item and menu without requiring Decksmith Studio to remain open.

**FR-INDICATOR-004** The quick-access menu shall provide at minimum: Open Decksmith Studio, runtime/device status, current profile/workspace, Pause/Resume Controls, and access to Preferences/Diagnostics.

**FR-INDICATOR-005** The indicator shall visually communicate meaningful states such as no device, connected/healthy, paused, locked, and attention/error without excessive animation.

**FR-INDICATOR-006** Users shall be able to hide the panel/tray indicator from Preferences while leaving `decksmithd` running.

**FR-INDICATOR-007** Selecting Open Decksmith Studio shall focus an existing Studio window if one exists; otherwise it shall launch a single Studio instance.

### FR-STORE — Persistence

**FR-STORE-001** Configuration shall be stored in SQLite.

**FR-STORE-002** Foreign keys shall be enabled.

**FR-STORE-003** WAL mode shall be enabled unless a platform issue requires fallback.

**FR-STORE-004** Database schema changes shall use ordered, versioned migrations.

**FR-STORE-005** The daemon shall maintain recoverable backups before destructive migrations.

**FR-STORE-006** Secrets such as API tokens shall not be stored as plaintext in SQLite; the system shall use the desktop Secret Service/libsecret when available.

**FR-STORE-007** Profile export shall exclude secrets by default.

### FR-PROFILE — Profiles, workspaces, pages, folders, and context

**FR-PROFILE-001** Users shall be able to create, rename, duplicate, delete, and reorder profiles.

**FR-PROFILE-002** A profile shall contain one or more task-oriented Workspaces.

**FR-PROFILE-003** Every profile shall designate a Home Workspace that provides a predictable navigation target.

**FR-PROFILE-004** A Workspace shall support multiple pages.

**FR-PROFILE-005** Pages shall support forward/back navigation and direct selection.

**FR-PROFILE-006** Keys shall support folders containing nested capability layouts.

**FR-PROFILE-007** The data model shall prevent recursive folder cycles.

**FR-PROFILE-008** A binding shall declare a scope of Global, Profile, Workspace, or Page, allowing persistent controls to survive lower-level navigation. Expose this reuse as **Sticky actions**: users can mark a key action to appear in the same position across all pages of its profile, with broader scope available through the existing model. Store one shared binding rather than copies, so action, appearance, and label edits propagate together. Clearly distinguish inherited sticky controls from page-specific controls, expose scope and conflicts before replacing a binding, and support removing stickiness without silently duplicating actions. Include shared bindings in undo/redo and layout import/export.

**FR-PROFILE-009** Profiles shall be switchable manually from the UI, CLI, and D-Bus.

**FR-PROFILE-010** Smart Profiles and **Automatic Page Switching** shall select an explicit profile/workspace/page based on the active window's application identity, without high-frequency polling. Page switching must not require a separate profile per application. Example rules include Spotify → music-album page, VSCode → projects page, and Firefox → websites page; these examples do not imply new music/project/browser integrations.

Planned desktop support targets are GNOME through a Decksmith GNOME Shell extension, Hyprland, Sway, Mangowm, KDE when kdotool is installed, and all X11 desktops through an appropriate active-window adapter. Verify each integration and its prerequisites before claiming support; detect missing dependencies and retain manual navigation when unavailable. These are requested support targets, not implemented compatibility guarantees.

**FR-PROFILE-011** For the GNOME application-page milestone selected September 16, each application maps to one page. Foreground application changes select the assigned page immediately, with no pause control or sticky manual override. Unassigned applications select the chosen default page (revised September 16); manual navigation lasts until the next foreground application change. Users shall understand the active application and assignment. Coalesce rapid focus changes, avoid feedback loops caused by editing Decksmith, and suspend automatic switching while Auto-Lock suppresses controls. Application identity is the default matching input; window-title/content matching requires a separate privacy/design decision.

**FR-PROFILE-012** A fallback/default profile shall always be defined.

**FR-PROFILE-013** Temporary Context Layers shall be able to override compatible controls, especially dials and touch-strip feedback, without mutating the underlying page configuration.

**FR-PROFILE-014** Context Layers shall have deterministic exit behavior such as explicit Back/Home, action completion, profile/workspace change, or optional timeout.

### FR-CAP — Capability engine

**FR-CAP-001** Functional behavior shall be modeled as reusable capabilities independent of device coordinates.

**FR-CAP-002** The capability taxonomy shall support at minimum Commands, Adjustments, Stateful Controls, Navigation, and Dynamic Providers.

**FR-CAP-003** A capability shall declare which input/output surfaces it supports, including raw key press/release, short press, double press, long press, hold/repeat, dial rotate, short/double/long dial push, touch gesture, workflow use, state subscription, and Control View rendering.

**FR-CAP-004** The system shall provide built-in capabilities for launching applications, opening URLs/files/folders, and running commands/scripts.

**FR-CAP-005** Keyboard shortcuts shall function on GNOME/Wayland through a supported input-injection helper rather than depending on X11-only tools.

**FR-CAP-006** PipeWire/WirePlumber capabilities shall include output volume, input volume, mute, default source/sink, and per-application stream control where exposed by the platform.

**FR-CAP-007** MPRIS capabilities shall include play/pause, next, previous, stop, seeking where supported, and current-track metadata/state.

**FR-CAP-008** systemd capabilities shall support start, stop, restart, and status for permitted user services and explicitly configured system services where authorized.

**FR-CAP-009** HTTP capabilities shall support method, URL, headers, body, timeout, and optional response-to-state mapping.

**FR-CAP-010** MQTT capabilities shall support publish and subscribe/state-feedback patterns.

**FR-CAP-011** Workflow/Multi Action execution shall support ordered steps, configurable delay, cancellation, stop-on-error, and continue-on-error. Conditional branching may be added after the initial stable workflow engine unless it can be implemented without compromising reliability.

**FR-CAP-012** Trigger bindings shall allow different capability bindings for short press, double press, long press, hold/repeat, and raw press/release, with equivalent dial-push gestures where applicable.

**FR-CAP-013** Stateful Controls shall support N states rather than being limited to Boolean toggles.

**FR-CAP-014** Adjustments shall expose current value, range/step metadata when available, incremental control, and live subscription when provided by the backend.

**FR-CAP-015** Dynamic Providers shall be able to enumerate, update, and remove live capabilities such as PipeWire streams, Home Assistant entities, OBS scenes, containers, VMs, or other provider-defined resources.

**FR-CAP-016** Control Views shall provide a generic rich interaction surface for compatible Adjustments and Stateful Controls, including value feedback, reset/default behavior, and optional secondary adjustments.

**FR-CAP-017** Capability execution shall emit structured start/completion/failure/state-change events for diagnostics and visual feedback.

### FR-UI — Configurator experience

**FR-UI-001** The configurator shall use GTK4 and libadwaita.

**FR-UI-002** The main canvas shall visually represent the selected physical device, including the Stream Deck + keys, touch strip, and dials.

**FR-UI-003** Compatible capabilities shall be assignable by drag-and-drop.

**FR-UI-004** The capability catalog shall be searchable and grouped by capability/integration.

**FR-UI-005** Selecting an assigned control shall open its properties without navigating away from the device canvas.

**FR-UI-006** Changes shall auto-save transactionally; no global Apply button shall be required.

**FR-UI-007** Valid changes shall appear on the device immediately.

**FR-UI-008** Undo/redo shall cover profile layout and capability-property changes within a user session.

**FR-UI-009** Copy/paste and duplicate shall work for controls and capability configurations.

**FR-UI-010** The application shall support light/dark appearance through GNOME/libadwaita conventions.

**FR-UI-011** The application shall support keyboard navigation, accessible names, focus states, scalable text, and screen-reader-friendly control descriptions.

**FR-UI-012** Empty states, error states, first-run onboarding, and disconnected-device states shall be intentionally designed.

**FR-UI-013** The UI shall expose a one-click diagnostics/export workflow for support.

**FR-UI-014** Empty controls shall offer an in-place Add workflow in addition to drag-and-drop.

**FR-UI-015** Search shall be type-aware: the catalog shall prioritize capabilities appropriate for the selected key, dial, touch region, workflow slot, or Control View.

**FR-UI-016** Capability entries shall provide human-readable descriptions, provider identity, compatible surfaces, requirements, and searchable aliases.

**FR-UI-017** The configurator shall provide first-run profile/template choices so new users are not forced to begin with an empty device.

**FR-UI-018** The configurator shall expose Profile -> Workspace -> Page navigation while keeping higher-scope/global controls understandable.

### FR-APPEAR — Customization, themes, and rendering

**FR-APPEAR-001** Functional behavior and visual appearance shall be stored as separate concepts so changing a theme cannot alter control behavior.

**FR-APPEAR-002** The Stream Deck + key grid shall support one panoramic background canvas that is automatically sampled/sliced for the eight physical key displays.

**FR-APPEAR-003** Device geometry shall account for key size, row/column position, and physical gaps when sampling a panoramic canvas so artwork remains visually coherent across the device.

**FR-APPEAR-004** The touch strip shall support a separate full-width background canvas independent of the key-grid background.

**FR-APPEAR-005** The touch strip shall support full-width, segmented, and hybrid visual modes, including transparent/semi-transparent widgets over a shared background.

**FR-APPEAR-006** Individual controls and individual capability states shall be able to override inherited background/appearance settings.

**FR-APPEAR-007** Appearance inheritance shall support at least Theme -> Profile -> Workspace -> Page -> Control -> State precedence.

**FR-APPEAR-008** SVG shall be the preferred source format for icons and scalable theme assets; PNG, JPEG, and WebP shall also be supported where practical.

**FR-APPEAR-009** Users shall be able to configure fit mode, position, scale, opacity, and common legibility treatments for background artwork.

**FR-APPEAR-010** Theme definitions shall be able to include deck background, touch background, icon treatment, typography, label placement, overlays, state styles, progress styles, and widget styles.

**FR-APPEAR-011** The GUI preview and physical device shall use the same render model so appearance edits provide WYSIWYG feedback.

**FR-APPEAR-012** Changes to appearance shall update the physical device interactively without requiring a global Apply operation.

**FR-APPEAR-013** User assets shall be content-addressed/deduplicated on disk; large image blobs shall not be stored directly in SQLite.

**FR-APPEAR-014** Static backgrounds are required for 1.0. Full-frame decorative animation is deferred, while functional state/progress animation may be supported when bandwidth and performance allow.

### FR-PLUGIN — Plugin system

**FR-PLUGIN-001** Plugins shall execute outside the core daemon process.

**FR-PLUGIN-002** Plugin crashes shall not interrupt device operation or native actions.

**FR-PLUGIN-003** The plugin host shall support a native Decksmith plugin contract and a compatibility layer for selected Stream Deck SDK behaviors.

**FR-PLUGIN-004** Compatibility WebSocket servers shall bind only to loopback addresses.

**FR-PLUGIN-005** Property inspectors for compatible web-based plugins shall run in an embedded WebKitGTK context separated from the main application UI.

**FR-PLUGIN-006** The host shall validate plugin manifests before installation or execution.

**FR-PLUGIN-007** The product shall clearly report why a plugin is incompatible (unsupported SDK requirement, Windows-only executable, DRM-protected package, missing runtime, invalid manifest, etc.).

**FR-PLUGIN-008** The project shall not attempt to bypass Elgato Marketplace DRM.

**FR-PLUGIN-009** Wine support, if provided, shall be optional and clearly marked best-effort.

**FR-PLUGIN-010** Native plugins shall be able to expose dynamic capability providers and live state subscriptions.

**FR-PLUGIN-011** Native plugin settings shall prefer a declarative schema rendered by the GTK configurator for visual consistency and accessibility.

**FR-PLUGIN-012** Application-scoped and global/system-scoped providers shall be distinguishable in metadata so the Context Engine and UI can present them appropriately.

### FR-IMPORT — Import, export, and backup

**FR-IMPORT-001** Profiles shall be exportable to a documented project format containing metadata, layout, capability configuration, and required assets.

**FR-IMPORT-002** Exported profiles shall not contain secrets by default.

**FR-IMPORT-003** Imports shall be validated before modifying the active configuration.

**FR-IMPORT-004** The system shall preserve a backup when importing over existing profiles.

**FR-IMPORT-005** Themes/appearance packages shall be exportable and importable independently of functional profile mappings.

**FR-IMPORT-006** Workflow templates shall be exportable/importable independently when they do not contain secrets.

**FR-IMPORT-007** Imports shall report missing plugins/providers/assets before committing configuration changes.

**FR-IMPORT-008** A later compatibility importer for `.streamDeckProfile` files may be implemented, but it is not required unless feasibility is validated without relying on private formats or DRM.

### FR-DIAG — Diagnostics and supportability

**FR-DIAG-001** Logs shall use structured Rust `tracing` events.

**FR-DIAG-002** Normal daemon logs shall integrate with the user journal.

**FR-DIAG-003** The configurator shall display device status, daemon version, database schema version, plugin status, and last relevant errors.

**FR-DIAG-004** A diagnostics bundle shall redact known secrets and include only data necessary for debugging.

**FR-DIAG-005** The CLI shall provide a hardware event monitor suitable for development and troubleshooting.

---

## 8. Non-functional requirements

### NFR-PERF — Performance targets

These are engineering targets to validate during hardware-in-loop testing, not marketing guarantees.

- Idle daemon CPU: normally below 1% on the reference desktop.
- Idle daemon memory: target below 100 MB RSS.
- Configurator idle memory: target below 250 MB RSS.
- Internal input-event dispatch: p95 target below 25 ms from HID event receipt to capability dispatch.
- Typical key image update visible on device: target below 150 ms after state change.
- Dial feedback update: target below 100 ms for local controls where the device transport permits it.
- GUI interactions such as profile/page switching should feel immediate, with no blocking I/O on the GTK main thread.

### NFR-REL — Reliability

- No configuration loss on abrupt GUI termination.
- Safe transactional writes for layout changes.
- Automatic USB reconnect.
- Recovery after suspend/resume.
- Plugin crash containment.
- Database migration backup/rollback strategy.
- No requirement to restart the daemon for ordinary configuration changes.

### NFR-SEC — Security

- No root daemon.
- USB access through udev user permissions.
- No network listener on non-loopback interfaces by default.
- Secrets stored through Secret Service/libsecret rather than SQLite.
- Plugin packages validated before execution.
- Plugin processes supervised and assigned explicit capabilities where practical.
- Input injection isolated behind a private helper boundary.
- Diagnostic exports redact secrets and auth headers.
- No attempt to bypass Elgato DRM or Marketplace protections.

### NFR-PRIV — Privacy

- No telemetry by default.
- No cloud account required.
- No background network calls unless required by a configured user action/plugin or explicit update check.
- Clear network-access indication for plugins/actions that use external services.

### NFR-ACCESS — Accessibility

- Keyboard-accessible configurator.
- Accessible labels for device controls and property editors.
- High-contrast compatibility through platform theming.
- No reliance on color alone for status/error indication.
- Scalable text and sensible minimum window size.

### NFR-MAINT — Maintainability

- Rust workspace separated by domain responsibility.
- No raw SQL outside the storage crate except migration files.
- No HID calls outside the device crate/worker.
- Versioned D-Bus contract.
- Capability and appearance models remain device-independent.
- No direct appearance/image rendering logic inside plugin business code when a project-native RenderModel can express the desired result.
- Architecture Decision Records for irreversible choices.
- `cargo fmt`, `clippy`, unit tests, integration tests, and dependency/license checks in CI.

---

## 9. UX quality bar

The application should feel comparable to a professionally shipped GNOME desktop product.

Required characteristics:

- consistent spacing and typography
- strong hierarchy
- concise labels and contextual help
- no exposed internal IDs or implementation jargon in ordinary workflows
- drag-and-drop behavior with clear drop targets
- visible selection state
- immediate WYSIWYG hardware preview
- panoramic/background editor with understandable inheritance and overrides
- consistent theme/style system rather than one-off image hacks
- in-place Add workflow on empty controls
- context-aware/type-aware capability search
- undo/redo
- safe destructive-action confirmation
- graceful offline/disconnected states
- helpful error recovery rather than stack traces
- polished onboarding
- accessible keyboard operation
- sane defaults
- no unnecessary modal dialogs

A feature is not considered complete merely because it functions technically; it must also satisfy the design, diagnostics, and error-state requirements relevant to that feature.

---

## 10. Product phases and exit criteria

### Phase 0 — Foundation

**Deliverables**

- repository skeleton
- coding standards
- license decision
- ADR process
- Rust workspace
- logging and error conventions
- CI scripts that can run on Gitea Actions and later GitHub Actions
- project-owned device interface plus first VirtualDeck implementation
- deterministic Trigger Resolver skeleton and timing-policy tests

**Exit criteria**

- workspace builds cleanly on Fedora reference machine
- formatting, linting, tests, and dependency policy run from one documented command
- architectural boundaries are represented in crate layout

### Phase 1 — Hardware developer preview

**Deliverables**

- Stream Deck + discovery
- serial/firmware read
- brightness
- eight key rendering
- raw key down/up events plus resolved short/double/long/hold-repeat triggers
- raw dial rotate/push events plus resolved short/double/long push triggers
- touch-strip drawing
- touch events and normalized semantic triggers
- reconnect
- CLI event monitor with raw/resolved views and JSON output
- CLI/VirtualDeck input emulation

**Exit criteria**

- reference Stream Deck + can be unplugged/replugged repeatedly without daemon restart
- every physical input can be observed as raw input and, where applicable, a resolved semantic trigger
- single/double/long/hold-repeat conflict-resolution tests pass deterministically
- test images render correctly on every key and touch-strip region

### Phase 2 — Daemon and persistence

**Deliverables**

- `decksmithd` systemd user service with graphical-login autostart
- Preferences-backed Start Decksmith at Login behavior
- D-Bus API
- SQLite schema and migrations
- profile/workspace/page/control persistence
- capability/context/appearance persistence
- backup before migration
- session lock/idle state subscription and policy persistence

**Exit criteria**

- closing Decksmith Studio leaves device operation intact
- after graphical login, `decksmithd` becomes available automatically without opening Studio
- configuration survives reboot, daemon restart, and clean upgrade migration

### Phase 3 — Capability and Context Engine

**Deliverables**

- application/file/URL launch
- command/script capabilities
- keyboard shortcut helper
- PipeWire/WirePlumber
- MPRIS
- systemd
- HTTP
- MQTT
- workflows/multi actions
- deterministic trigger logic: short press, double press, long press, hold/repeat, and raw press/release
- capability taxonomy and catalog
- multi-state controls
- live adjustment/state subscription model
- Context Engine and temporary Context Layers

**Exit criteria**

- a useful daily profile can be built without installing any plugin
- native capabilities provide visible success/failure/state feedback where relevant
- the same capability implementation can be bound to compatible keys, dials, workflows, and Control Views without duplication

### Phase 4 — Stream Deck + interaction and workspace layer

**Deliverables**

- encoder feedback widgets
- per-dial strip regions
- dial stacks
- action wheel/menu
- Workspaces and Home Workspace navigation
- global/profile/workspace/page binding scopes
- Control Views
- touch-strip flick navigation
- partial strip refresh optimization

**Exit criteria**

- all four dials can independently control and display live Linux state
- no visible stale placeholder/render artifacts under rapid updates

### Phase 5 — Professional configurator

**Deliverables**

- Decksmith Studio libadwaita shell
- The Forge creation/editing workspace
- Blueprints import/export/template terminology
- device canvas
- capability catalog
- property inspector pane
- drag-and-drop
- icon/title editor
- SVG-first asset library
- panoramic key-grid background editor
- independent touch-strip background editor
- theme/appearance inheritance and overrides
- active/idle/locked appearance editing
- live WYSIWYG preview using the production renderer
- undo/redo
- copy/paste
- onboarding and profile templates
- diagnostics screen

**Exit criteria**

- a new user can configure a complete profile without CLI use
- a user can apply one panoramic key background, a separate touch background, and per-control overrides without manual asset slicing
- changing a theme does not modify functional bindings
- no known major keyboard-accessibility blockers
- common workflows require no manual file editing

### Phase 6 — Smart Profiles and GNOME integration

**Deliverables**

- minimal Decksmith GNOME Shell companion
- active-app D-Bus signal
- Decksmith GNOME panel indicator/menu with Open Studio and runtime status
- StatusNotifierItem fallback/integration for supporting non-GNOME desktops
- Smart Profile rule UI
- fallback behavior

**Exit criteria**

- profiles switch reliably on GNOME Wayland without frequent polling
- panel indicator opens/focuses Decksmith Studio and reflects daemon/device state
- shell companion failure does not stop manual device use

### Phase 7 — Plugin compatibility beta

**Deliverables**

- plugin manifest validation
- plugin process supervisor
- loopback WebSocket compatibility host
- property inspectors via WebKitGTK
- supported SDK event subset
- native dynamic provider API
- declarative native plugin settings
- compatibility diagnostics

**Exit criteria**

- documented test plugins work consistently
- plugin crash does not affect daemon/device
- unsupported/DRM plugins fail with a clear explanation

### Phase 8 — Hardening and release engineering

For the focused V1, use the [release gates and four-VM matrix](v1-release-scope.md)
as the acceptance checklist. RPM, COPR and a generic diagnostics bundle remain
later goals rather than V1 blockers.

**V1 deliverables**

- tested per-user Fedora installation, update, rollback and data migration;
- physical Stream Deck + regression and the four-VM test matrix;
- security policy, release authenticity, release notes and contributor guidance.

**Later packaging/support deliverables:** native RPM and COPR builds, plus a
general-purpose redacted diagnostics bundle.

**Exit criteria**

- clean installation and uninstall on a fresh Fedora test system
- suspend/resume/reconnect regression suite passes
- no release-blocking security or data-loss defects

### Phase 9 — Public 1.0

This phase is complete only when the V1 plan's Fedora reference, physical
Stream Deck +, VM, accessibility, security and release-candidate gates pass.
Publishing source or a preview archive by itself does not satisfy the phase.

**Deliverables**

- public canonical GitHub repository
- release notes
- installation documentation
- support policy
- known limitations
- contribution guide

**Exit criteria**

- project quality bar is met on Fedora reference environment
- core use does not depend on Elgato Marketplace, cloud services, or Wine
- appearance customization is stable, portable, and independent of behavior mappings

---

## 11. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| GNOME/Wayland does not expose global active-window state to ordinary apps | Smart Profiles fail | Minimal GNOME Shell extension emits only active-app metadata over D-Bus; no polling architecture |
| Elgato Marketplace DRM prevents third-party hosts from executing protected plugins | Plugin parity gap | Do not promise DRM compatibility; prioritize open/unprotected plugins and high-quality native actions |
| Windows-only plugins depend on Wine/Mono/MinGW quirks | High support burden | Best-effort optional subsystem after native/Linux plugins; never required for core product |
| Plugin process can crash/hang | Device reliability | Separate processes, supervision, timeout/kill/restart, no plugin code in daemon process |
| Plugin gets access to command/input capabilities | Security | Capability-oriented host APIs, sandboxing where practical, private helper boundaries, clear user permission UI |
| Touch-strip rendering regressions | Visible quality failure | Dedicated renderer tests, golden-image tests, mock layouts, hardware-in-loop visual verification |
| Gesture ambiguity/timing | Wrong action fires or perceived latency | Deterministic Trigger Resolver, monotonic time, explicit conflict rules, configurable defaults, automated timing tests |
| Simulator/hardware divergence | CI passes while physical Deck fails | Shared device contracts plus mandatory hardware-in-loop release gates |
| HID library limitations or upstream API changes | Hardware reliability | Thin adapter crate around upstream library; never expose third-party types throughout domain model |
| SQLite corruption or bad migration | Configuration loss | WAL, transactional migrations, pre-migration backup, integrity check, recovery UI |
| Project copies GPL OpenDeck implementation too closely | Licensing constraint | Use OpenDeck as behavioral/reference research; write original implementation against public docs unless project license intentionally adopts GPL |
| Trademark/confusion with Elgato | Legal/branding issue | Avoid “Stream Deck” in product brand; use compatibility language and clear non-affiliation statement |
| Flatpak sandbox conflicts with HID/plugins/uinput | Packaging complexity | Native RPM is reference packaging; Flatpak deferred until architecture is stable |
| Rich customization causes render/update bandwidth problems | Device responsiveness degrades | Device-aware render cache, dirty regions, coalescing, static V1 backgrounds, functional animation only |
| Appearance inheritance becomes confusing | UX complexity | Explicit scope breadcrumb, preview, reset-to-inherited actions, and strict Theme -> Profile -> Workspace -> Page -> Control -> State precedence |
| Dynamic providers create unstable IDs/entities | Broken bindings | Provider-defined stable identity contract, stale-resource state, reconciliation UI, and explicit missing-provider diagnostics |

---

## 12. Success criteria

The focused V1 succeeds when a Fedora 44 GNOME/Wayland user can install the
application, connect one Stream Deck +, configure pages and supported controls
visually, control desktop/audio/media/system actions, use the dials and touch strip
meaningfully, customize the currently supported appearance, close Decksmith
Studio, continue using the device reliably, and reopen Studio quickly from the
desktop indicator without cloud services or Elgato software. The physical
reference-host and four-VM acceptance in the [V1 release plan](v1-release-scope.md)
must be complete. Profiles/workspaces and panoramic backgrounds remain broader
product goals rather than claims of this V1.

A stronger success signal is when the application is preferred even by Linux users who do not need Elgato plugin compatibility because the native Linux integrations are better than simply emulating Windows/macOS behavior.

---

## 13. Source and compatibility notes

Current reference material reviewed for this baseline:

- Stream Deck + product capabilities: https://www.elgato.com/us/en/p/stream-deck-plus
- Elgato software/downloads: https://www.elgato.com/us/en/s/downloads
- Stream Deck + HID protocol: https://docs.elgato.com/streamdeck/hid/stream-deck-plus/
- Stream Deck SDK and plugin protocol: https://docs.elgato.com/streamdeck/sdk/
- Plugin distribution/DRM: https://docs.elgato.com/streamdeck/sdk/introduction/distribution/
- OpenDeck project: https://github.com/nekename/OpenDeck
- `elgato-streamdeck` Rust crate: https://docs.rs/elgato-streamdeck/latest/elgato_streamdeck/
- Logitech MX Creative Console: https://www.logitech.com/en-us/shop/p/buy-mx-creative-console
- Logitech Actions Ring / Smart Actions concepts: https://www.logitech.com/en-us/software/options-plus
- Loupedeck software/downloads and interaction model: https://loupedeck.com/us/downloads/
- StreamController Linux Stream Deck project: https://github.com/StreamController/StreamController
