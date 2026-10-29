# Wallpaper change and lock-gate redraw — October 29, 2026

## Observation

On Fedora GNOME, both Decks redrew when the slideshow changed wallpaper with
Auto Accent Colour enabled. During one comparison with that extension disabled,
the user observed a wallpaper change without either Deck redrawing. The extension
was restored afterward. This comparison alone does not establish why GNOME was
slow or prove behavior for every wallpaper.

At 15:28:39 EDT, the installed diagnostic build recorded gate invalidations for
both devices. A read-only sample observed the GNOME lock gate change from
available/unlocked (epoch 4) to unavailable/blocked (epoch 5) and back to
available/unlocked (epoch 6). The associated `GetActive` call began at
15:28:39.693 and its response arrived approximately 396 ms later. The existing
GNOME call limit was 250 ms. The gate invalidation and subsequent full frame
submissions match this timeout-and-recovery sequence; no user content, device
serials or wallpaper paths were captured.

## Source change

The GNOME query deadline is now 650 ms. GNOME and logind queries run concurrently
so their independent deadlines do not add together. A positive lock result
from either source blocks immediately, without waiting for the other query.
Unknown and timed-out GNOME state continues to fail closed after the deadline,
and the existing one-second
stale-state cutoff is unchanged. The longer deadline can delay recognition of a
failed GNOME query by up to 400 ms compared with the prior 250 ms deadline; it
does not ignore an explicit locked result. Delays longer than the new bound can
still cause a safe gate transition and redraw.

## Isolated checks

On a private test D-Bus, a 400 ms unlocked GNOME response left the gate epoch
unchanged; a slow explicit locked response blocked; a response exceeding the
650 ms limit blocked as unavailable; and a subsequent successful response
recovered. Simultaneous 400 ms simulated GNOME and logind responses completed in
approximately 400 ms rather than adding their delays, and an explicit lock
from either source did not wait for the other slow query. Existing unknown/stale,
brief-lock and unsignalled logind-state tests passed. A new two-device test
confirmed that an unchanged unlocked gate does not repaint either Deck, and 14
existing simulated two-device execution and lifecycle tests passed. The normal
project check now runs the slow-query regressions on a private D-Bus session. The
hardware-enabled daemon passed Clippy with warnings denied; formatting and diff
checks passed.

These are isolated results. The source change has not been installed on the
desktop or physically verified. A signed candidate and explicit desktop approval
are required before validating the fix against the actual slideshow and Decks.
