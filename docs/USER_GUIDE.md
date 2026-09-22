# Decksmith user guide

This guide describes the current Fedora/Stream Deck + development build. The
v0.4 architecture documents include future features as well as implemented ones.

## Workspace navigation

**Home** contains device, startup, Auto-Lock and brightness controls. **Pages**
organizes pages. **Keys & Dials** edits controls. **About** contains the guide,
project and support links. Choose Device shows the current Stream Deck +; this
version controls one device at a time. Section colors follow GNOME light/dark mode.

Selecting a preview control only edits it. **Test action** explicitly runs the
saved key action or dial press; save edits first. Test push-to-talk holds on the
physical device. Ctrl+S saves, Ctrl+Z undoes and Ctrl+Shift+Z redoes. Closing with
unsaved changes offers Keep editing, Discard and close, or Save and close.

Drafts survive tab changes and disconnection. Unlock or start background controls
to Save and Apply. Transparent key background uses the page color; it does not
make the physical display transparent. See [workspace behavior](workspace-ui.md).

## Start and edit

1. Open **Decksmith** from the application menu.
2. Start **Background controls** if they are stopped. Keep OpenDeck closed while
   Decksmith owns the Stream Deck.
3. Choose **Keys & Dials**. The editor follows the device's current page. Use
   **Pages** to organize pages and application assignments.
4. Select a key to edit its label, action, and appearance. Expand **Appearance**
   for artwork, text placement/color, transparency, and background color.
5. Choose **Save and Apply** to send the draft to the device.

Turning off **Edit keys** lets preview keys navigate between pages. It does not
turn the editor into a general launcher for every assigned desktop action.
Use the page menu to rename, duplicate, reorder, or delete pages. Page links track
renames. Key labels support mixed case, words that wrap, and up to 24 characters.
Page names support uppercase letters, digits, and spaces up to eight characters.

For dials, click a dial or its touch-strip section in the device preview. Its
rotation and press settings open beside the canvas, in the same editor as keys.
Use the searchable action chooser to find compatible actions. Key actions are
grouped by Audio, Media, Navigation, Applications, and General; dial actions are
grouped by Turn and Press. Changes join the layout draft immediately; **Save and
Apply** sends them to the device. Switching controls preserves edits, and Undo,
Redo, and Discard changes include dial edits. The touch strip shows the same rendered content as the physical device, scaled to
fit the preview. Saved, active layouts display the last image sent to the device;
unsaved layouts use the same Rust renderer with the latest known audio/brightness
state. Refresh runs in the background without blocking editing. The note below the
device distinguishes live, draft, disconnected and unavailable previews. New audio
targets that the device is not yet monitoring show -- until the layout is applied.
Invalid drafts retain the last valid touch-strip image with a clear status note;
Save and Apply stays disabled until the draft is valid.
Clicking an image section still selects its dial; it does not execute its action.
The bar fill shows live signal activity; the numerical percentage shows configured
volume. See [Live audio meters](audio-meters.md) for details.
Dial settings start as shared defaults across pages. Select a dial, then choose
**Customize for this page** to change only that page. The editor shows **This page**
for an override; **Use shared settings** removes it and restores the latest shared
value. Editing **Shared settings** updates every page still inheriting that dial.
Overrides include the dial action, target, label, step and press settings. Undo/redo,
page duplication and layout export/import preserve them. **Save and Apply** makes
the draft active; switching pages then switches the dial actions and feedback
together. Existing layouts keep their assignments unchanged.

Selecting an audio target fills
the touch-strip label with its name; selecting Device brightness fills it with
Brightness. You can edit those labels afterward.

## Themes and icons

Use **Edit layout → Theme** for Maker’s Mark, Dark, Light and High Contrast, shared
fonts and color defaults. Existing custom styles remain until you choose
**Appearance → Reset to theme** on a control. For key artwork, choose
**Appearance → Icon library…** to search the 134-icon Tabler starter set or import
a local image. **Cancel** or **Escape** closes the picker without changing your
icon or pending edits. **Icon size** starts at 100%; use the compact minus/plus buttons
to adjust it in five-point steps between 10% and 100%, or type a valid percentage.
These changes preserve action assignments and join the normal
draft/undo/Save and Apply workflow. See [Appearance presets and local icons](appearance-and-icons.md)
for sharing, licenses and the link to download the full Tabler collection.

## Live control feedback

When a saved control needs attention, the panel identifies its target, problem and
next step below **Edit layout**. Amber key/dial warnings indicate unavailable or
unsupported targets. Failed actions briefly show red feedback; timeouts do not imply
success. Availability warnings recover automatically when the app/device returns.
The panel separates **Last attempt** failures from live availability, and unsaved
previews do not inherit saved-binding warnings. See [Everyday control feedback](control-feedback.md)
for the status meanings and safe checks.

## Audio targets and volume

Choose Audio volume for a dial, or an Audio action for a key, then select its
Audio target. Installed audio apps are discovered at launch and refreshed while
the choices are visible. **Installed** means an app can be assigned in advance;
**Audio detected** means a matching playback stream exists. A saved missing target
stays unavailable rather than silently switching to another device or app.
Discovery includes a five-second recovery check and a manual refresh button.

An app-volume control changes its playback streams through Fedora. It does not
move the app or website's own volume slider. For example:

**YouTube volume → Brave app volume → Schiit Magni output volume**

Each stage affects what you hear. Adjusting Brave in Decksmith leaves YouTube's
slider unchanged and affects matching Brave streams, including other tabs.
Browser-based calling apps can share that same browser audio target. The
percentage on the touch strip is the selected audio target's level.

**System output** follows Fedora's default output device. When the Magni is the
default, System output and the named Magni control change the same setting.
An independent master mixer is not implemented yet.

Tap an audio touch-strip section to toggle its assigned target's mute state.
Horizontal swipes still change pages. Brightness-only sections ignore taps.
Named device sections and Select device keys have a bottom marker:

- Green: current default input or output.
- Gray: available, but not the default.
- Amber: unavailable or status unknown.

These markers identify the default device, not whether sound is playing. Apps
explicitly routed elsewhere may remain there when the default is changed.

## Push-to-talk

Assign **Microphone · Push to talk** to a key and select a named microphone.
To mirror it on a dial, choose **Dial press → Push to talk** and the same microphone
as its Audio target. Rotation can adjust microphone volume or have No action.

Hold a key or dial down to unmute; release to mute. If both hold the same mic,
it remains open until the last hold ends. The matching touch-strip section shows
**Muted / Live**. A push-to-talk key's marker uses green for live, gray for muted,
and amber for unavailable.

A touch-strip tap remains a mute toggle, not hold-to-talk. It can leave the mic
live until toggled or a push-to-talk hold is released. Saving an assignment does
not change the initial mute state. Check the displayed state before a call.
These controls affect Fedora's microphone source; the calling app's own mute
button is separate.

Page/layout changes and disconnects cancel holds. The microphone guard also
attempts to mute on lost heartbeats or background-process shutdown. An audio
service or operating-system failure can prevent that operation; the displayed
state comes from the audio service rather than assuming success.

## Media playback

Choose **Media · Play / Pause**, **Previous track**, or **Next track** on a key
or dial press. Then choose **Media player**:

- **Automatic:** initially prefers a playing player, then stays with the last
  successfully controlled player session, including while paused.
- **A named player:** controls only that app. If missing or unsupported, it does
  not fall back to another app.

Start the player and open/play media if it is not listed. Missing saved choices
remain marked Unavailable. A browser choice targets its exposed media session,
not a particular YouTube tab. Next/Previous behavior depends on the player's
playlist and supported commands.

Media actions and Audio Target volume controls use separate interfaces. Play/Pause
can work before an app exposes an audio stream for volume adjustment.

## Preserve your layout

The editor menu offers **Export layout…**, **Import layout…**, and
**Restore previous**. Export saves a portable `.decksmith` package. Imports load
as drafts: inspect them, then Save and Apply. Restore previous is one saved
previous layout, not a full version history. Undo/redo and key copy/paste operate
within the editor draft.

After an editor update, save open edits and close/reopen Decksmith to load the
new code. Restart Background controls only when the daemon itself was updated
or troubleshooting calls for it. Closing the panel does not stop background
hardware controls. Enable **Start Decksmith at login** in the main panel to start
background controls automatically when you sign in. The window stays closed;
opening the editor is optional. Changing this preference does not start or stop
the currently running controls. Existing installations keep their startup setting.

The optional GNOME panel menu opens Decksmith and starts or stops background
controls. See [desktop integration](desktop-integration.md) for installation.

Enable **Auto-Lock** under **Your device** to disable the Stream Deck controls
when your session locks. The display shows Locked and restores your page after
unlocking. If lock detection is unavailable, enabled controls stay locked until
it recovers or you turn Auto-Lock off. See [Auto-Lock](auto-lock.md).

The **Pages** sidebar in Edit layout keeps page names, assigned apps, and the default
page visible. Use **Add**, the up/down arrows, **Duplicate**, or **Delete** to manage
your pages; edit the selected page below the list. Key and dial settings appear
beside the device preview in wide windows, or above it in compact windows.
The editing area scrolls independently from the sidebar. Changes
remain in your draft until **Save and Apply**.

To switch pages with your foreground application, open **Edit layout → Pages**,
select a page and choose an app under **Switch here for**, then **Save and Apply**. The GNOME extension
must be enabled. Unassigned apps select the chosen default page; manual navigation lasts
until the next app change. Select **Use as default page** in the Pages sidebar and
Save and Apply to change the default. See [automatic application pages](automatic-pages.md).

## More detail

- [Audio controls](audio-controls-foundation.md)
- [Media controls](media-controls.md)
- [Visual editor](visual-editor.md)
- [Layout packages](layout-packages.md)
- [Latest checkpoint](checkpoints/2026-09-12.md)
- [Remaining roadmap](ROADMAP.md)

### Audio icons and mute appearance

Each audio touch-strip panel shows a small device-type icon beside its label.
Microphones and apps have their own symbols; speakers and headphones are identified
when Fedora provides that information. Otherwise, a neutral audio symbol appears.
App dials can use their installed application icon. Select the target or use Refresh application icon, then Save and Apply. Missing artwork uses a common application symbol. Muted targets keep the normal border, show a red crossed-out icon, and replace
the volume percentage with red “Muted” text, including when mute changes outside Decksmith. Unavailable
targets show --. Unmuted input-device volume/Live readouts are green. The editor mirrors the physical strip's rendering.

The bar fill shows measured signal activity while the numerical percentage shows
configured volume. An empty bar means silence; a dash means measurement is
unavailable. Muted targets have no activity fill. Brightness retains its ordinary
set-value bar and percentage.

Brightness panels show a light-bulb icon beside the label, using the same style as
audio icons. The percentage and bar continue to show device brightness.

A thin line beneath the volume bar appears for explicitly assigned input/output
devices: green means the device is Fedora's current default, gray means it is not
the default, and amber means the default status could not be determined. This line
is not a sound-level meter and does not indicate whether audio is playing.

Audio activity fill is blue below 80, yellow from 80, orange from 90, and red from
95 on the normalized signal scale. These colors follow measured activity, not the
volume percentage; brightness stays blue.

## System controls

Key actions now include System control: Lock, Do Not Disturb, immediate Night Light, power profiles, Suspend, confirmed Reboot/Shutdown, and Bluetooth radio toggling. Configuration stays inline. See [System actions](system-actions.md) for behavior and state indicators.

### Editing context and diagnostics

Keys & Dials identifies the page and selected control in its heading. Unsaved preview means edits have not yet been sent to the device. Dial assignments identify All pages (shared defaults) or This page (an override); Appearance expands inline. Layout appearance settings affect all pages. About shows the installed build and Copy diagnostic information copies a compact status summary without layout contents, URLs or application targets.

Touch-strip titles are left-aligned with a consistent gap after the device icon. Status and percentage readouts remain centered. No Audio uses the same typography as the volume readout without a yellow outline; missing devices and failed actions retain their warning treatment.
