# Native profile editor and model rendering — October 13, 2026

The tested saved-profile foundation was committed/published to Gitea as `4348bf0`
and the separate sanitized GitHub history as `7ff3589`, with identical source tree
`9af58856792ece286aaab4b290697ac89cc0aec9`. Its committed Rust modules were checked
against the previously qualified source snapshot, using the final overlay entry
for each file in the archive.

## New integration

The explicit [profile editor lab](../profile-editor-lab.md) now connects native
GTK editing to private V2 profiles and a supplied Plus/MK.2 VirtualDeck pair:

- Device-addressed selection, profile reads, saves, page changes and draft previews.
- Random runtime identity plus saved/connection/binding revisions on mutations.
- Independent retained drafts and undo state; no implicit default-device fallback.
- Eight Plus keys/touch/dials; fifteen MK.2 keys and model-specific page creation.
- Shared style/text composition with actual output geometry; MK.2 uses 72-pixel
  images and never receives touch writes. Legacy Plus previews retain byte parity.
- Complete frame validation and exact owner checks before writes; a failed writer
  blanks/retires its owner while another remains usable. Stop attempts full blanking.
- Bounded request queue, messages and draft renderer cache; no duplicate key/touch
  rendering for a key-only preview, and no repeating JSON/artwork decode for an
  identical cached request. Preview work stays off the GTK main thread.

## Qualification

Six added Rust fixtures cover native model frames/Plus parity, independent saves,
page isolation and saved-data reload, invalid preview/model/page rejection without
writes, stale runtime and lock rejection before disk publication, transport isolation
and blanking, and exact large-integer preservation with serial redaction.

Four Python fixtures cover explicitly addressed requests/replies, wrong-owner reply
rejection, save-before-load rejection, and fifteen-key page creation with dial rejection.
Native GTK acceptance exercises device switching with a dirty Plus draft, editing
and saving the fifteenth MK.2 key, returning to the retained Plus draft, independently
saving it, and actual model-sized textures. It captures both native editor views
and checks the MK.2 last row is inside the window at the test display size.

Local and AI-server hardware-enabled Rust suites each passed **248 tests**, with
zero failures and three environment/hardware checks ignored. Formatting, complete
build and all-target/all-feature Clippy checks passed with warnings denied.
The Studio suite ran 186 tests (one conditional skip on the AI server); the script
suite ran 111 (one conditional skip on the AI server). Native profile and existing
five-section main-workspace checks passed using the isolated 1920 × 1200 display.

The final Python/native snapshot is
`adca40c999d81350b97ac695f699725d94c966aa65a86b1ee8f864c2a429e2ba`.
Rust source modules were checked byte-for-byte against the complete AI qualification
snapshot `61574ba9596210f72f3adf895716e9ded87bc112233c0c28f0c3efab496bd48b`;
only test-launcher isolation changed afterward. Final source files were compared
with the qualified archive. Private evidence retains earlier failures and corrected
runs. The native accessibility-bus helper was unavailable in this isolated display;
these checks do not qualify screen-reader behavior or physical response/resource
thresholds. The test launcher's missing brand texture and a newer-toolchain lint
were corrected before acceptance. Ordinary startup and Control1 remain unchanged. No physical
HID, real audio, VM or scheduled checkout was touched; no desktop update was installed.

## Remaining work

This is an isolated lab, not a deployment of simultaneous physical support. The
production main-window selector/Home controls, retention/compaction, dynamic roster,
shared audio/microphone/media/plugin dispatch, asynchronous display scheduling,
physical MK.2 adapter ownership/transport and pair/lifecycle/resource qualification
remain activation gates. Hidden lab editors preserve independent drafts; the final
production workspace must reuse one visible editor without growing per-device UI
workers indefinitely. This subsequent integration remains local/uncommitted until
explicit publication authorization.
