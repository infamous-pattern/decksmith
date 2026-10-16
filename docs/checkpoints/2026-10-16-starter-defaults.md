# Maker’s Mark starter defaults — October 16, 2026

Status: implementation and local qualification passed for proposed V1.5.1;
signed-candidate qualification is pending. Source synchronization and the signed
candidate build were approved separately; stable publication remains pending.
The existing V1.5.0 release, installer pin and desktop installation remain intact.

The maintainer requested these changes before announcing V1.5 and selected
Home-page-only placement for the new website key.

## New installations and new device profiles

Use the Maker’s Mark preset with Sans, Small, White text, Transparent · page color
background, Bottom key labels, Transparent label background and 85% icon size.
The Home page’s bottom-left key uses the approved existing Maker’s Mark artwork
and Open website → https://github.com/infamous-pattern/decksmith.
This is key 5 on Stream Deck + and key 11 on the standard 15-key MK.2.
Other starter pages retain their assignments. On the Plus Home page, Mute moves
from key 5 to unused key 6; its action remains available.

`config/plus-default.json` is the Plus starter for installation, virtual startup
and enrollment without an imported layout. `config/mk2-default.json` is the MK.2
starter. Historical `audio.json` and `navigation.json` examples remain unchanged.
The installer can still manage older bundles that have only `audio.json`.

## Appearance and preservation

Transparent key backgrounds reveal the page background behind artwork and text;
physical key displays remain opaque RGB surfaces. Transparent label backgrounds
show the underlying artwork. Transparency is valid only for backgrounds, not text
colors. The native editor, theme import/export and Rust renderer agree on this
meaning. Ordinary Theme inheritance remains distinct from explicit transparency.

Choosing Maker’s Mark or Reset shared defaults supplies the new explicit preset
settings. Per-control customizations retain priority. Merely opening, upgrading or
rendering a saved legacy theme does not inject the new defaults. Saved device
profiles, labels, artwork and behavior are not rewritten by this change.
Downgrading after saving the new transparent-background value requires the
complete earlier configuration backup and matching earlier build, as documented
for V1.5 recovery; an older parser will reject the new value.

## Qualification and remaining delivery

Local Rust, native editor model, installer and strict Clippy checks cover both
starter models, explicit transparency, legacy appearance, profile preservation,
import/export, real pixel rendering at 120×120 and 72×72, correct model-aware
website routing, staged installation and fresh paired VirtualDeck startup.
Native GUI checks passed for both models on AI_server with isolated XDG directories
and a fake in-memory endpoint, without physical-device actions or production services.
All four native screenshots and the physical-size pixel contact sheet were inspected.
Final automated totals: 346 Rust tests passed (6 ignored), 238 editor tests passed,
and 115 installer tests passed; strict Clippy and formatting passed.
The first native harness attempt omitted the scoped-device/fake-save metadata;
its fixture errors are retained separately and corrected checks passed. No product
change was needed for those harness errors.
Private results and images are retained under `outputs/v15-defaults-update-20261016`
in the parent workspace; they are not release assets.

The initial local bundle was an unsigned development build from uncommitted source.
Before public delivery: review the defaults, commit/push the approved source,
build and verify a new signed V1.5.1 candidate, perform the targeted desktop check,
and approve publication and installer promotion. Do not replace signed V1.5.0
assets or move its tag. Leave the announcement pending until the update is ready.
