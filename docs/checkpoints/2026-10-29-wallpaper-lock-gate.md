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
and the existing one-second stale-state cutoff is unchanged. The longer deadline can delay recognition of a
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

## Signed candidate and desktop verification

The fix and these testing notes were committed to Gitea at
`eee591c88f5422ac14955578d13c5158d9b9d8ff` and GitHub at
`77316139c8a84bc952d3cad2015ea2ec7615b3b9`; their source trees matched. The
[quality run](https://github.com/infamous-pattern/decksmith/actions/runs/40944234273)
and [Fedora build and signing run](https://github.com/infamous-pattern/decksmith/actions/runs/40944239983)
passed. The downloaded Fedora candidate **1.5.2-13e2b24be597** was independently
verified against the retained signing bundle and public commit; checksums and all
508 manifest files passed. Its archive SHA-256 is
`075c21d4f13c3d8315ee28a54aea118ca9c2a2629e8008b89cb0ef86a7a91bba`.
An isolated staging installation passed without activating desktop controls.

After the maintainer saved and closed Studio and approved installation, the
candidate was installed over **1.5.2-8a523b8af67f**. A fresh snapshot of the
configuration, plugin data, service settings and previous build was hash-verified
beforehand; all 618 configuration files matched afterward. Both Decks reported
ready. The maintainer confirmed page and safe-key operation on both Decks, Plus
audio and media controls, and an existing Homebridge control.

At the 16:08:39 EDT wallpaper change, the maintainer observed neither Deck
redraw. The GNOME lock reply took 262 ms: beyond the old 250 ms deadline and
within the new deadline. The diagnostic recorded no lifecycle-gate or full-frame
submission in the observed window. With separate maintainer approval, the
high-volume temporary redraw diagnostics were disabled and controls restarted;
both Decks again reported ready, the configuration remained unchanged, and the
maintainer confirmed both work. Later the maintainer reported not noticing further
refreshes. The exact duration and number of subsequent wallpaper transitions were
not measured. This is encouraging physical evidence, not proof that delays beyond
650 ms cannot still cause a fail-closed redraw or that all reliability gates are
complete.
