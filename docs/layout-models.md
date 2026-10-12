# Model-aware layouts and offline migration tools

Implemented as V1.5 groundwork on October 12, 2026. These tools prepare and
validate documents; the normal app still uses its existing Plus/V1 layout path.
They do not enable physical MK.2 control or a multi-device daemon.

## Saved formats

| Layout format | Model declaration | Keys per page | Dials / touch |
| --- | --- | --- | --- |
| V1 | Implicit Plus; no `model` field | 8 | Existing four-dial defaults/overrides and Plus touch behavior |
| V2 | `"model": "plus"` | 8 | Same supported Plus assignments |
| V2 | `"model": "mk2"` | 15 | No dials, dial overrides or touch surface |

V2 requires an explicit known model. Unknown versions/models, wrong key counts,
incompatible control assignments and host-identity fields are rejected rather
than remapped or discarded. V1 remains accepted by the existing control API;
V2 documents are deliberately rejected there until the device-addressed API and
renderer are ready. Rust is authoritative for semantic validation.

The MK.2 factory creates its own **Home, Media and System** pages, each with
15 keys, bottom-row Previous/Home/Next navigation and Maker's Mark appearance.
Most keys remain unassigned; Media includes playback actions and System includes
Lock. See [the default document](../config/mk2-default.json). It does not copy
the user's Plus pages or assignments.

Migration changes only the top-level version/model declaration. Nested JSON
values are kept as raw tokens, including custom Unicode labels, artwork, page
links, themes, dial overrides and unavailable-provider settings. Original files
are retained byte-for-byte separately. Saved PNG artwork remains at the current
canonical 120-pixel asset resolution; model-sized frame rendering and physical
orientation qualification are later integration gates.

Built-in action definitions have stable `decksmith.builtin` provider IDs,
explicit action IDs and settings schema 1, independent of labels/translations.
The projection retains existing enum execution semantics and preferred media
players, and preserves existing plugin identities/schemas. This is an inspection
and compatibility seam, not a generic plugin executor or permission system.

## Development commands

Build the hardware-enabled daemon from the workspace; this enables its document
tools without requiring a connected device:

```sh
cargo build -p decksmithd --features hardware --locked
target/debug/decksmithd --layout-info ./layout.json
```

The report shows model, descriptor, version and action-definition counts. It
does not print labels, settings, URLs or credentials. It starts no background
workers and makes no HID, audio or desktop-bus connection.

Use a saved fixture configuration directory to prepare a migration bundle:

```sh
target/debug/decksmithd --prepare-layout-store ./legacy-test-config ./migration-test
target/debug/decksmithd --restore-legacy-layout-store ./migration-test ./restored-test-config
```

The source must contain a valid V1 `layout.json`. All bounded regular files in
that directory, including nested artwork and preferences, are copied into a
private backup. A changing source snapshot is rejected. Destinations must not
already exist or be inside the source; the tools never overwrite an existing
configuration directory. Source files are never edited. Restore produces a
separate V1 directory; selecting that directory with a matching prior build is
a separate deployment step. Systemd/login integration, installed executables and
external Secret Service data are outside this directory snapshot.

The bundle contains independent unassigned Plus/MK.2 documents and a checksummed
legacy backup in a generation directory. Files/directories are synced and verified
before an atomic `current.json` pointer swap. A failed preparation leaves an
unpublished generation; it is not adopted automatically. Existing active
generations survive failures before publication. A sync failure after a pointer
swap is reported as uncertain and is never automatically replayed. Neither this
pointer nor the tool report activates the bundle in the running Decksmith service.

Owner checks, private store directories, no-follow file opens, exclusive creation,
cooperative writer locking, stale-generation rejection and relative-path checks
protect the migration boundary. SHA-256 detects changed files; it is not an
authentication signature or an OS sandbox against the same user.

## Bounds and packages

- Layout: 1 MiB and 1–16 pages; plugin settings retain their existing 4 KiB bound.
- Snapshot: 256 regular files, 512 entries, depth 8, 2 MiB per file and 32 MiB total.
  Empty-directory metadata and original timestamps/modes are not a restore contract;
  restored content uses private permissions.
- Store: at most 16 generations, including failed preparations. It stops at the
  bound instead of deleting backups automatically; retention UI/policy remains work.
- Package: 2 MiB compressed/uncompressed aggregate. V1 retains its 160-entry
  limit; V2 permits at most 256 entries for the 15-key/16-page asset grid.

V2 package manifests declare model and exact key/dial/touch requirements, checked
against the reconstructed document. Exports omit host serial/device identity and
retain existing artwork-license notices and website-URL redaction. V1 package
import is preserved. Explicit legacy export is available only for Plus layouts
using current action/settings shapes; MK.2 keys are never truncated. Container
checks do not replace the native semantic/image validator before applying a layout.

## Remaining integration

Bind profiles to stable device IDs, migrate Plus brightness/preferences and keep
global login/Auto-Lock preferences, provide the device-addressed API and compiled
runtime/renderer, and add independent drafts/selection in the integrated editor.
Qualify actual deployed cross-build rollback, physical MK.2 behavior and Plus/MK.2
simultaneous recovery before release. The experimental plugin permission and
generic contract gates remain separate.
