# Homebridge assignment and cross-device key copy — October 17, 2026

## Report and cause

The maintainer reported that the installed `1.5.1-9d7719a68ed2` desktop build
allowed Homebridge/accessory selection on MK.2 without an apparent save step,
and could not copy keys between the Plus and MK.2.

The accessory picker stages its choice only after a separate **Use assignment**
button at the bottom of its expanded settings. Choosing an accessory alone did
not dirty the draft or enable Save and Apply. That staging step was too easy to
miss. Separately, replacing a page-link key retained `follow_page_name`; the
native parser rejected the resulting Homebridge key as `invalid_config`. An old
media-player selector could likewise conflict with a replacement action.

Copy state belonged to each `Draft`. Switching the selected Deck retained
independent drafts but left the destination with no copied key.

## Prepared correction

- **Assign to key / Assign to dial** is in the accessory chooser header. It
  stages the choice; the fixed header's **Save and Apply** updates only the
  selected Deck. Assignment feedback states that the draft is staged.
- Replacing a key with Homebridge removes settings and generated artwork
  provenance belonging to its former action, preserving unrelated appearance.
- One private clipboard belongs to the editor window and works across saved
  device selections. Key data, opaque plugin settings and artwork are copied
  independently; pasting still requires Save and Apply and has normal undo/redo.
- A page link pasted onto another Deck resolves by a unique case-insensitive
  destination page name. It never adopts the source Deck's numeric page index.
  Missing/ambiguous names produce an explanation and leave the target unchanged.
  Same-device links still track page reorder/rename and reject deleted targets.

The maintainer approved committing/pushing this V1.5.2 editor correction to both
repositories and building a signed Fedora candidate for desktop review. Candidate
preparation does not install it on the daily desktop or publish a release. It does not link navigation between
Decks or copy entire layouts/pages.

## Verification

- All **244 editor tests passed**, including six new regressions covering both
  model directions, conflicting prior action settings, independent drafts,
  undo/redo, safe page resolution and unchanged targets on rejection.
- The **exact installed desktop V1.5.1 daemon** was copied and hash-verified for
  a private-bus, virtual-device integration test on the AI server. It reproduced
  the original MK.2 navigation/plugin rejection without changing the saved
  profile, then accepted corrected MK.2 and Plus assignments and cross-model
  copy/save in both directions. The other Deck's saved layout stayed intact.
- Native UI captures cover the visible assignment step, active Save and Apply,
  stable window geometry and a compact 1100×700 window. These are fixture
  accessories on virtual devices; no physical HID, credentials or accessory
  commands were used. Daily services and saved layouts were left unchanged.
- The first private UI check ran before the UI's reconnection status settled;
  its failure is retained. The corrected test waits for the genuine enabled
  Save button. Final fixture discovery is explicitly stubbed before constructing
  any companion client, avoiding an unintended unavailable-catalogue display.

Private evidence: `outputs/mk2-homebridge-clipboard-20261017/evidence/`.
The source publication, signed Fedora candidate verification and approved desktop
acceptance are recorded below. Next: continue exact-package Ubuntu/Debian testing;
physical Ubuntu acceptance remains deferred until the maintainer is ready.


## Signed candidate and desktop acceptance

Both repositories received the same source tree: canonical commit `05e8dfed`
and public commit `e479d956`. Quality run `39063555536` and Fedora build/signing
run `39063576701` passed. Signed Fedora candidate **1.5.2-46ed033e0491** was
verified against the exact public source and all signed downloads. Its packaged
editor files match the approved tree. The actual candidate also passed the
private native save/copy/undo/redo and compact-window checks on AI_server.

After explicit saved/closed-editor approval, the candidate was installed on the
maintainer's Fedora desktop. The complete configuration backup was verified,
V1.5.1 and the existing Homebridge companion were retained, and both devices'
saved layouts, selection, pages and brightness passed the post-install guard.

The maintainer then confirmed physical acceptance:

- MK.2 Homebridge accessory selection, Assign to key, Save and Apply and the
  physical accessory action work; the Plus remains working.
- MK.2-to-Plus key copying saves and operates the same accessory; MK.2 remains
  working. A shared binding exists in the saved profiles. The first two saved
  observations did not isolate a new Plus revision (Plus unchanged, MK.2 changed),
  so those observations alone do not prove destination revision/source isolation
  for this direction. Physical success is the maintainer's report.
- A different Plus key copied to MK.2 saves, displays and executes correctly;
  the original Plus key remains working. Read-only verification confirms MK.2's
  saved layout changed, the Plus layout stayed unchanged, and exact assigned-key
  payloads match across the saved device profiles.
- Both physical device sessions remain connected with input/display ready and
  no error. The app and companion retain their post-install PIDs with zero
  automatic restarts.

The user's saved test assignments remain in place. This verifies the reported
Fedora desktop symptoms; it does not publish V1.5.2, qualify another OS or replace
its remaining release review. Private receipt: the task's `physical-review.json`.
