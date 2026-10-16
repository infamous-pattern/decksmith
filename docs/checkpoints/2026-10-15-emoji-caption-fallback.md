# Emoji caption fallback — October 15, 2026

During permanent paired-editor acceptance, the MK.2 Home Key 2 label saved correctly as `MK2✅`, but the physical renderer substituted a question mark for the checkmark. The Plus layout was unchanged. This is a rendering defect, not a save or device-routing failure; save/restore acceptance remains pending.

The shared key and touch-strip caption renderer now falls back to bundled Noto Emoji outlines when the selected font lacks a symbol. Ordinary letters retain their selected font. Glyph metrics and variable-font locations follow the font used for each glyph. The fallback uses the configured label color. It does not implement full-color emoji or joined-sequence shaping; unsupported sequences retain bounded safe rendering.

The unmodified OFL 1.1 font and license are pinned and hashed in `assets/fonts/font-sources.json`. No system font installation, network access, dependency update or profile migration is required. Label text and action assignments are unchanged.

Automated verification covers common standalone emoji in all fourteen font choices, label color, variation selectors, nonblank output, and replacement-glyph regression. Before/after contact sheets for ordinary captions and decorative runes are pixel-identical. Physical acceptance of the corrected symbol and the pending editor restore test must be recorded separately.

Private build, image and qualification evidence: `outputs/v15-emoji-20261015` in the enclosing workspace. Desktop activation is pending a qualified package and saved/closed editor. Previous installed build, complete device store and persistent device page/brightness state are retained for rollback.

## Packaged and installed qualification

The optimized hardware build and strict Clippy passed; the all-feature Rust suite passed 338 tests with five intentional ignores. The package verified 485 manifest files. A private-bus two-device VirtualDeck check rendered the exact saved `MK2✅` caption through the packaged Preview API at 72 × 72; the checkmark was visually inspected and differed from the same text without the symbol. Direct renderer regressions also distinguish emoji output from the question-mark replacement. The comparison fixture initially used an ASCII question mark, which the existing layout validator rejects; it was corrected to valid plain text without changing the validator.

Desktop build `1.0.0-dev-emoji-4551ce366af5` installed successfully with the unchanged qualified Homebridge companion. Both device sessions were machine-verified ready; all saved layouts, selected device, page and brightness were preserved. The update retained the prior build, a verified full device-store snapshot, the persistent page/brightness state, and a complete configuration backup. No rollback was needed. Physical confirmation of the checkmark and the still-pending editor Restore previous test remain open. Earlier lifecycle and accessibility results apply to the preceding recovery build; they have not been repeated for this text-rendering-only follow-up.

## Physical checkmark acceptance

The user confirmed the physical checkmark is recognizable on MK.2 Home Key 2 and the Plus still works normally. A live read found both sessions connected, display/input ready, and error-free; the Plus layout matched its private pre-test baseline exactly. Saved MK.2 text remained `MK2✅`.

The preceding MK.2 revision contains the same test label with hidden visibility, reflecting two separate user saves during diagnosis. Restore previous is therefore expected to restore that earlier test revision first; it must not be described as restoring the original Empty baseline. The predecessor snapshot is retained privately for exact comparison. The user has been prompted to load and apply that revision; exact baseline restoration follows only after verification and a check for unrelated edits. This editor restore acceptance is still pending.

## MK.2 editor restore and cleanup

The user located the icon-only Layout options menu below the right-side device preview on Keys & Dials, loaded Restore previous and saved it. The physical caption disappeared and the Plus remained working. A live read confirmed the MK.2 saved layout exactly matched the retained predecessor; the Plus still matched its original baseline. The initial top-header instructions were incorrect and were corrected to the actual menu location, which is a discoverability issue for later UI cleanup.

After verifying there were no unrelated edits, a revision-guarded save restored the MK.2 original unused key and exact original layout from the private baseline, preserving name, saved page and brightness. Both original saved layouts and both ready sessions were verified. The immediate post-save readiness assertion observed the repaint transition; a read-only follow-up confirmed recovery without a second save or service restart. The user has been prompted for the corresponding Plus caption save on Home Key 8 (Kate); its action and artwork must remain unchanged. That mirrored permanent-editor save/restore acceptance is pending.

## Plus caption save acceptance

The user confirmed that the physical Plus displayed `Plus 🎵` and the MK.2 remained unchanged and working. Read-only verification found exactly two changes on Plus Home Key 8: label and label position (Bottom). Action and artwork were unchanged; the MK.2 matched its original baseline exactly. Both device sessions remained connected, input/display ready and error-free.

The Plus predecessor contains the test label with hidden visibility, reflecting separate saves for label and appearance. That exact predecessor has been retained for comparison. The user has been prompted to load/apply it through Restore previous; cleanup to the original Kate baseline remains pending after verification.

## Plus restore and completed paired editor check

The user confirmed the Plus caption disappeared after Restore previous and the MK.2 stayed unchanged and working. The saved Plus layout exactly matched its retained predecessor. A revision-guarded save then restored the original Kate layout from the private baseline, refusing unrelated edits. Both original saved layouts were verified exactly, with names/pages/brightness preserved and both device sessions ready. This closes the physical label-save/previous-layout restore acceptance on both devices; it does not claim arbitrary action reassignment coverage. Controlled quiet/loaded resource qualification and release artifact/installer/security gates remain.
