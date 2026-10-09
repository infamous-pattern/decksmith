# Single-arc audio meter — October 9, 2026

## Accepted behavior

The user requested a meter-only dial presentation while preserving the assigned
turn and press actions. **Single-arc meter** is an additional per-dial style;
Dual-arc gauge remains the default and Classic bar remains available. Shared
settings and page overrides follow the existing effective-dial resolver.

After reviewing the first installation, the user requested restoring the dial
percentage to the right of the single arc. The revised audio presentation has
one wider left-hand activity arc, with configured volume on its right and no
second volume arc or pointer. Target icon/label, red crossed-out mute icon and
Muted text, green microphone Live text and input/output default marker remain.
Silent measured signal, missing measurement and No Audio remain distinct.
The live arc cannot overwrite its right-hand percentage/status column. Device
brightness keeps a setting gauge because it has no audio measurement. Plugin displays are unchanged.

This is a presentation choice, not a new volume/mute action or calibrated
loudness meter. The signal uses the existing normalized peak scale and colors.
Geometry is cached once; the current meter sampling budget and worker count are
unchanged. No measured CPU improvement or new resource qualification is claimed.

## Validation and installation

Strict all-feature formatting/lint and 146 Rust tests passed; three existing
hardware/long diagnostics were ignored. The 171 editor tests passed, along with
dependency advisory, license, ban and source checks. New checks cover every
integer meter level, silence versus unavailable measurement, mute suppression,
status clearance, signal independence from configured volume, updating volume
readouts matching Dual-arc, device/preview byte parity, unchanged action bindings, shared/page override history and package
round-trip. Existing dual-arc/classic behavior remains covered.

An isolated Fedora 44 VM install passed runtime integrity/dependency checks,
the actual GTK Single-arc selector/reselection, custom-label preservation and
private-bus VirtualDeck/native panel rendering. Test configurations used private
paths; no physical USB or host audio was passed into the VM. The completed VM
was shut down with autostart disabled.

After the user saved desktop edits and approved installation, the local test
bundle `1.0.0-55993ea45866` was installed, retaining the previous build and a
configuration backup. Saved configuration/artwork hashes were unchanged, and
the physical Deck connected with its display ready. Existing assignments were
preserved; the user chooses Single-arc explicitly in Appearance and Save and
Apply. The user reported the missing percentage during physical review.

The corrected local bundle `1.0.0-80b6faedd10b` restores the configured percentage
on the right of the single signal arc. All 146 Rust and 171 editor tests passed
again with strict all-feature lint/format checks. Synthetic live/muted captures
match the shared physical renderer; every integer level leaves the value column
clear. Installation retained the preceding test build and a fresh configuration
backup. Saved configuration/artwork hashes remained unchanged, and the physical
Deck reconnected with its display ready. Physical acceptance of this revision is
pending; the earlier native selector/persistence VM checks remain applicable.

Existing signed Fedora V1, Debian, Ubuntu and COSMIC release assets are unchanged.
The COSMIC preview includes Dual-arc and Classic bar; Single-arc is subsequent
development work. Use the retained configuration backup with the previous build
if rolling back a layout that now selects Single-arc, since older versions do
not recognize this new style.
