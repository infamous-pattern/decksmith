# System key actions

Choose **System control** in the key editor, then select an action inline. Default labels and bundled artwork are editable. These first adapters target Fedora GNOME.

- Lock Desktop locks the current GNOME session.
- Do Not Disturb toggles notification banners; it does not mute audio.
- Night Light switches immediately. Enabling temporarily uses an all-day schedule; disabling restores the saved schedule unless you changed it in GNOME meanwhile. The warm-color feature is then off.
- Power Mode cycles available profiles. Individual Power Saver, Balanced, and Performance assignments are also available; unsupported modes report unavailable. The default Power Mode caption reflects the observed profile.
- Suspend requests normal system sleep and respects authorization/inhibitors.
- Reboot and Shutdown open an explicit desktop confirmation with Cancel, Escape, and no automatic countdown. Confirmation respects GNOME application inhibitors and logind authorization/inhibitors; it never forces shutdown. Save work before confirming.
- Bluetooth toggles the GNOME Bluetooth radio setting. Missing adapters and hardware blocks report unavailable. This is not pairing or per-device connection management.

A small key indicator is green for active, gray for available/inactive, and amber for unavailable or not yet checked. Custom labels are retained. System state is shared by hardware and editor rendering.

The daemon starts a small GIO helper lazily, reuses it for actions, caches observations for two seconds, and bounds requests. It can release the helper when polling a page with no system controls; otherwise an idle helper sleeps until the next request or daemon exit. Actions use a fixed allowlist without shell command expansion. No administrator password is stored.

## Fedora 44 VM acceptance limits

Lock, DND, immediate Night Light, advertised power profiles, explicit restart confirmation/cancel, and confirmed reboot passed in the retained guest. Suspend entered sleep and returned, but the virtual display remained black and the subsequent guest reboot stalled, requiring a libvirt guest reset. VM suspend/resume remains limited by that guest display failure. Bluetooth unavailability was verified without adding host adapter passthrough; physical Bluetooth toggling was subsequently confirmed by the user on the host.

Confirmed Shutdown also passed: the explicit dialog powered off only the Fedora 44 guest, and libvirt reported shut off.

## Suspend recovery

The first physical suspend test left the Stream Deck blank even though USB and the daemon stayed connected. Restarting background controls restored it (user confirmed). The daemon now detects Linux sleep-time changes independently of wall-clock changes, drops the old device handle and display cache, cancels held/queued controls, waits two seconds for USB recovery, and restores the current page and saved brightness. Auto-Lock still applies before rendering. This also covers sleep initiated outside Decksmith.

Regression tests cover reopening an apparently healthy connection, restoring every key and the strip, retaining the current page/brightness, rejecting queued commands, and keeping the locked display protected. On September 17, the user confirmed successful physical suspend/resume recovery with the correction installed. The user subsequently rebooted the host and confirmed everything was working as expected, including the updated icons.

Host feedback also confirms Lock, DND, immediate Night Light, and the Shutdown/Reboot confirmation dialogs. Host shutdown/reboot execution has not been requested as a test.
