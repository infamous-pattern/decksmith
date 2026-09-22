# Automatic application pages — GNOME/Wayland

In **Edit layout → Pages**, select a page and choose an app under **Switch here for**, then **Save and Apply**. Each application can belong to one page. Bringing
that application forward selects its saved page immediately, without a debounce
delay or pause control. Bringing Teams forward from Brave is a typical example.
Assignments use installed desktop-file identities, never window titles or content.

Unassigned applications select your chosen default page. In the **Pages** sidebar,
turn on **Use as default page**, then **Save and Apply**. Choosing another page
replaces the previous default. Existing layouts initially use the first page.
The default also becomes the startup page; it moves with reordering, is not copied
when duplicating a page, and falls back to the first remaining page if deleted.
The choice supports undo/redo and layout import/export. Manual navigation continues to
work until a subsequent foreground application change; no sticky manual override
blocks the next assigned application. Switching between windows of the same app
does not override manual navigation. Decksmith itself cannot be assigned a page,
and following device status in the editor does not issue another navigation
request. Unsaved drafts remain intact when the device changes pages.

Assignments move with reordered or renamed pages, are removed with deleted pages,
and are cleared on duplicates to avoid conflicting rules. Missing installed apps
keep their saved identity and are marked in the picker. App discovery runs when
the editor opens. Assignments are included in layout export/import. Older runtimes
may reject layouts with application assignments; use the installer's pre-update
configuration backup with a rollback rather than assuming older schema support.

The Decksmith GNOME extension must be enabled. The main panel reports whether
its application-reporting connection is available. Extension loss leaves manual
navigation working and retains the active page. The extension reports focus
changes as events; a low-rate daemon check verifies source presence only, without
polling foreground windows. Reconnecting the source reports the current app.

Auto-Lock takes precedence. Focus changes cannot switch pages while locked; only
the latest application state can be resolved after confirmed unlock. Automatic
page changes cancel held controls and invalidate stale queued actions. An action
already executing cannot be recalled. There is a short input-queue drain after a
page change for held-input safety; it does not delay selecting or painting the
new page. Saved assignments apply on subsequent focus changes.

Some browser-hosted apps may share the browser's desktop identity. Separate Teams
and Brave assignments require GNOME to identify Teams as a separate desktop app;
window-title matching is deliberately not used as a workaround.

Initial target: the user's GNOME/Wayland desktop. Other compositors and broader
Linux distribution compatibility remain future work. Automated checks cover rule
validation, duplicate/reorder behavior, manual navigation, unmatched apps, lock
precedence, stale requests, focus coalescing and unsaved draft retention. Record
native integration acceptance separately from these checks.

## Integration acceptance — September 16

Fedora 45 Beta / GNOME 51: real test application windows produced identity changes
and selected the assigned VirtualDeck pages. Manual navigation persisted until
another application change. Disabling the extension preserved manual controls;
reenabling reported the current application. The native editor passed application
assignment, unsaved draft retention during device page follow, and Save and Apply.
GNOME's advertised extension reload method is deprecated; the VM used a new login
to load the updated extension. Host activation is verified separately.

Host Fedora / GNOME 50.4: activated the runtime and enabled the previously disabled
extension without logout. Verified real Brave identity reporting, source availability,
physical device connection/display readiness, and unchanged saved configuration,
active page, Auto-Lock setting and login preference. GNOME retains old extension
metadata until a new login, but the new reporting interface is active. Existing
assignments were not invented or changed; the user can now configure their own
app/page choices. Exact Brave/Teams interaction remains user acceptance, and Teams
must have a distinct desktop identity to be assigned separately from a browser.

Picker interaction follow-up: selecting an application now dismisses both the
application list and its parent Page options popover; Page options also provides
Done. The previous nested popup left the outer options open despite a responsive
editor. Native regression exercised a new unsaved PAGE A, visible list selection,
both popup dismissals, preserved draft, Done, Save and Apply, and subsequent real
GNOME application focus selecting that saved page.

Physical acceptance: the user confirmed that foregrounding Joplin selects its
assigned page 3 and that the picker repair works. The subsequent requested policy
revision returns to the chosen default page for an unassigned application such as
Thunderbird; assigned-to-assigned switching remains immediate.

Chosen-default acceptance: native Page options selection, undo/redo, popup
dismissal, Save and Apply, assigned-to-assigned switching, unassigned application
return to a non-first default, and daemon restart persistence passed in GNOME.
Automated coverage also verifies old-layout normalization, duplicate/deletion
handling and layout-package roundtrip of the default and application assignment.

Sidebar acceptance (2026-09-16): page management is now persistent beside the
device preview. The former nested Page options popover is removed. The sidebar
shows names, app assignments and a Default marker; page settings and compact
reorder/duplicate/delete controls remain visible while key/dial settings scroll
independently below the unchanged hardware preview.
