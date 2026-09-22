# Desktop integration

Enable **Start Decksmith at login** in the main panel to run background device
controls at sign-in. No editor window opens. This changes future login behavior;
it does not start, stop, or restart the current service. Existing installations
retain their preference, and the installer never silently enables startup.

The optional GNOME menu shows service/device status, opens Decksmith, and starts
or stops background controls. It does not need the editor to stay open. Startup
also works without this extension. GNOME 50 and 51 are declared targets; GNOME 51
Beta has passed activation and background-only login checks. GNOME 50 acceptance
remains pending. Profiles, pause, diagnostics, and other desktop indicators are
future work.

## Install the GNOME menu

From a runtime bundle or checkout, run:

```sh
python3 scripts/install-gnome-extension.py --enable
```

GNOME may require signing out and back in to discover a newly installed extension.
If prompted, enable **Decksmith** in Extensions after your next login. Installation
does not restart GNOME or alter background controls. Disable/remove the companion
through Extensions independently of the main application.

## Validation

Fedora 45 Beta: native preference on/off test confirmed saved state and unchanged
service invocation. After reboot with startup enabled, the service was active,
the GNOME extension was active, and no Decksmith panel process existed. Extension
disable/re-enable succeeded. This VM has no physical Stream Deck; these checks do
not establish physical device behavior or replace host desktop acceptance.

On September 17, the user rebooted the host to refresh the application and GNOME panel icons and confirmed everything was working as expected afterward. This records host user acceptance; it does not expand the VM compatibility results above.
