# Simplified native editor — October 15, 2026

The user approved the interactive Keys & Dials mock-up, including visible overflow
markers, and requested native implementation. This change reuses GTK/libadwaita,
the existing Editor and draft/history models. It adds no framework or recurring
scroll observer timer, and does not promote the experimental multi-device runtime.

## Interface changes

- One header device selector; Home retains service, login, Auto-Lock and brightness
  controls. Saved-profile history/archival is inside inline Manage profiles.
- Key assignment starts with Label, Transparent label background, an inline
  searchable categorized Action chooser and the selected action's fields.
  Available Homebridge assignment appears in that same flow. Explicitly choosing
  the already-selected built-in action also replaces a plugin assignment.
- Dial Assignment keeps label, rotation, target and press together. Adjustment
  step is inside More behavior; Appearance summarizes inheritance and gauge style.
  Shared defaults, per-page overrides and existing input semantics are preserved.
- Save and Apply stays in the fixed header with one saved/unsaved indicator.
  Repeated healthy messages are hidden; validation and operation errors remain.
  Test action is above the device preview rather than inside the assignment form.
- Native scrollbar gutters are reserved in the main form, Home, Pages, About,
  plugin and preview scrollers. Actual-position above/below buttons disappear when
  content fits; homogeneous cue slots keep the allocation stable. Adjustment
  listeners disconnect when a panel unmaps. Nested action search has a persistent
  non-overlay native scrollbar.
- Compact windows use narrower spacing and a proportionally smaller model-specific
  preview. Selecting a key or dial does not trigger resizing.

## Qualification and limits

Automated native checks run on the AI server with a private display/session bus,
private user directories, VirtualDecks and no real audio/HID or VM access. The
normal V1 desktop service remains the daily single-device runtime. Source snapshots,
intermediate failures, logs and rendered captures are retained privately.

The final source passed 216 Studio Python tests and native checks for normal,
1024 × 768 compact, dark and enlarged-text layouts; VirtualDeck profile switching;
fresh saved-device enrollment; connected profile edits; and Homebridge key/dial
assignment and discovery. An additional wrapped plugin-page check verified hidden
panels stop polling and disconnect scroll listeners, then reconnect on return.

Layout checks exercise an expanded searchable chooser and Appearance, click the
actual-position scroll cues, reach the final control, verify a usable editor
viewport and keep Save/preview/editor geometry stable through key/dial changes.
Separate per-mode rendered captures were visually inspected. This review caught
and corrected an initially collapsed short-window viewport; the regression check
now requires more than 300 pixels of usable assignment height. Intermediate
failures remain in private evidence, including a stale picker-fixture expectation
updated to match the existing accepted Unicode label policy. That policy did not
change.

A staged install validated the packaged application against copies of the saved
user layouts. All 55 packaged runtime Python modules match the qualified source.
The local UI package retains byte-identical installed V1 daemon/CLI binaries;
the normal packaging strip step was bypassed for those retained bytes. The live
V1 read-only preview API returned correctly sized key and strip images without
changing saved configuration. No Rust runtime source or live action was changed.

Desktop visual review, compositor keyboard navigation and Orca are separate
acceptance checks; the server's headless fixture cannot certify them. This is a
local interface candidate, not a public release or a multi-device promotion.


## Desktop installation

The tested UI candidate is installed locally and opened for review. The previous
V1 build and an installer configuration backup are retained. Post-install checks
confirmed byte-identical daemon code, unchanged saved configuration, a connected
and ready Plus and the same active page. The first endpoint response preceded USB
settling; the later ready-state observation established recovery. Login settings
were preserved. No editor was open before installation.

On October 15, the user confirmed stable key/dial switching, inline Action and
Appearance editing, clear scroll markers and reachable expanded controls. The
user also confirmed the physical Plus shows its normal page and its controls work
normally after installation. These checks establish acceptance of the simplified
interface and ordinary physical-control recovery; they do not certify a new
compositor keyboard traversal or Orca check.

Desktop keyboard/Orca checks remain pending. The installed daemon remains
single-device V1; the MK.2 stays outside this desktop activation. Paired lifecycle,
migration and resource qualification remain the next runtime work.
