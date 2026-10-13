# Single workspace and verified profile history — October 13, 2026

The preceding editor/rendering integration was committed/published to Gitea as
`3fc023b` and the separate sanitized GitHub history as `194facb`, with matching
source tree `8ba5872f67be3e88de7c52cbb3e85251a459c60c`.

## Implemented and isolated

The explicit [VirtualDeck workspace](../profile-editor-lab.md) now reuses the
approved native Home/Pages/Keys/Plugins/About shell and one editor, preview pair
and worker set. Switching devices does not create another window. Bounded shelves
retain each layout's dirty edits, undo/redo, page/key view and pending Home name/
brightness. Unchanged saved data can use a fresh context; a remote layout change
keeps the old expected edit revision instead of silently overwriting it.

The selector shows owned device names. Home settings save only that device's
metadata, preserve its layout/current page, and fence only its binding. Device
model geometry is known before first layout load; Plus has eight keys/touch/dials,
MK.2 fifteen keys without touch/dials. Late preview replies are invalidated when
rebinding. All sections retain the same preview/settings arrangement. Live action
execution and desktop login/lock controls remain explicitly unavailable in the lab.

[Verified archives](../profile-history.md) retain old generations separately before
pruning. Full validation and sync precede unlink; a private mirrored journal blocks
writes during interrupted cleanup and resumes only against the verified archive.
Corrupt/incomplete/unexpected history is rejected; no unknown file or old archive
is automatically deleted. Selection no-ops consume no generation. Archiving leaves
active layout, edit revision, page, frame and device contexts unchanged.

## Qualification

Eight new Rust fixtures cover archival restore of identity/legacy bytes, safe
pre-prune failure, interrupted cleanup/write blocking, corrupt/wrong archive
rejection, stale/nested/incomplete/unexpected history, no-op selection/writer lock,
owned metadata/page preservation and unchanged live contexts during archiving.
Five Python fixtures cover independent dirty and clean/redo histories, exact
revision conflicts, metadata-only rebasing and bounded non-evicting draft retention.

Native acceptance checks the same editor/window dimensions across selection,
fifteenth-key editing/saving, retained Plus layout and Home drafts, independent
brightness, model-sized textures, unavailable live actions and explicit archiving.
The existing V1 main-workspace regression also runs on the isolated desktop.

Local and AI-server hardware-enabled Rust checks each passed **256 tests**, with
zero failures and three environment/hardware cases ignored. Build, formatting and
all-target/all-feature Clippy checks passed with warnings denied. The final Studio
suite ran 191 tests (one conditional skip on the AI server); the script suite ran
111 (one conditional skip on the AI server). Both the new single-workspace and
existing V1 native checks passed at 1920 × 1200 on the isolated display.

The full Rust/build qualification snapshot is
`1485ef723df61d3bf423d6d82a2e0b67f97e0230a701974e429fc904722432b5`.
The final Python/native snapshot is
`f7d3e445698c215db88f7c2b0d0ddbf2142567614832e6ef0ff62e84ba63e6b1`.
Rust modules were compared byte-for-byte with the full qualification source;
current changed files were compared with the final archive. Private evidence
retains the complete outcomes and captured native views. The isolated accessibility
bus helper was unavailable; these checks do not certify Orca or physical timing.
No physical HID, real audio, production user configuration, VM, scheduled checkout
or installed desktop build was changed. Native screen-reader, physical lifecycle
and resource/response thresholds are not inferred from these fixtures.

## Remaining activation gates

Review production archive quota/automatic policy and async disk/recovery workers;
wire service/login/lock and dynamic discovery/roster; connect shared audio/media/
microphone/plugin dispatch and asynchronous model display scheduling; implement
physical MK.2 ownership/transport. Then qualify the Plus/MK.2 pair, matched deployment
recovery, accessibility and sustained resource behavior. This subsequent work
remains local/uncommitted until explicitly authorized for publication.
