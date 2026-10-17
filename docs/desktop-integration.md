# Desktop integration

Enable **Start Decksmith at login** in the main panel to run background device
controls at sign-in. No editor window opens. This changes future login behavior;
it does not start, stop, or restart the current service. Existing installations
retain their preference, and the installer never silently enables startup.

The optional GNOME menu shows service/device status, opens Decksmith, and starts
or stops background controls. It does not need the editor to stay open. Startup
also works without this extension. GNOME 48, 50 and 51 are declared targets.
Actual GNOME 48.7 (Debian) and 50.1 (Ubuntu) session integration checks passed;
earlier GNOME 51 Beta checks covered activation and background-only login.
These checks do not qualify an entire distribution or physical device lifecycle.
Profiles, pause, diagnostics, and other desktop indicators are
future work.

## Install the GNOME menu

From a runtime bundle or checkout, run:

```sh
python3 scripts/install-gnome-extension.py --enable
```

GNOME may require signing out and back in to discover a newly installed extension.
If prompted, enable **Decksmith** in Extensions after your next login. Ensure the
global extensions switch is also enabled; an extension can remain inactive when
user extensions are globally disabled. Installation
does not restart GNOME or alter background controls. Disable/remove the companion
through Extensions independently of the main application.

## Validation

October 17, 2026: Ubuntu's signed candidate passed real GNOME 50.1 indicator
start/stop/restart, installed-editor open/Quit, independent foreground/default-page
selection on paired virtual Decks, and extension source loss/recovery. Debian's
corrected signed candidate `1.5.2-1272d3a4d717` passed the same checks on GNOME 48.7
without editing the installed extension metadata or bypassing version validation.
The normal no-device runtime separately verified source-liveness detection and
the Waiting for device indicator; virtual mode does not run that monitor.
Original guest data, preferences and service startup state were restored and the
VMs shut down. See the [current qualification checkpoint](checkpoints/2026-10-17-ubuntu-debian-support.md)
for exact signed source, earlier harness failures, cleanup recovery and open gates.
This does not promote Ubuntu or Debian to supported status or replace physical
USB/audio/lock/suspend/login acceptance.

Fedora 45 Beta: native preference on/off test confirmed saved state and unchanged
service invocation. After reboot with startup enabled, the service was active,
the GNOME extension was active, and no Decksmith panel process existed. Extension
disable/re-enable succeeded. This VM has no physical Stream Deck; these checks do
not establish physical device behavior or replace host desktop acceptance.

On September 17, the user rebooted the host to refresh the application and GNOME panel icons and confirmed everything was working as expected afterward. This records host user acceptance; it does not expand the VM compatibility results above.
