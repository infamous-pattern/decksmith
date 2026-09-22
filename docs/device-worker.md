# Device worker checkpoint

A manually controlled long-running session is now available; see background-app.md.
Earlier bounded-only descriptions below record previous checkpoints.


Display scheduling and completion semantics have changed; see display-scheduler.md.
Earlier synchronous-rendering descriptions below document prior checkpoints.


Build with `cargo build --locked -p decksmithd --features hardware`.
With OpenDeck closed, run:

```sh
./target/debug/decksmithd --hardware --exclusive --seconds 60
```

This is a bounded 1..120-second foreground diagnostic, not an installed persistent
service. The existing `--virtual-once` command remains available. It does not write
images, reset the device, change brightness or restore artwork after unplugging.

A dedicated thread owns the physical device. The output thread consumes JSON records
from a bounded 256-record channel. Records are `waiting`, `connected`, `input` and
`disconnected`. Each successful open increments a process-local session number;
input timestamps remain relative to that session. Consumers must cancel all active
controls on `disconnected`, and discard prior session state before accepting another
`connected`. Semantic trigger dispatch is not yet integrated in this worker.

Any input error drops the old handle and its queued events. Opening is retried once
per second, with interruptible waits; repeated unavailable-device status is suppressed.
Exactly one Plus is required. Reconnect does not prove that it is the same physical
unit; persistent device identity and multiple-device selection are still pending.

A full channel stops the worker with `event_queue_full`, rather than silently losing
input. The command reports failure and consumers must invalidate all its sessions.
The normal deadline signals the worker to stop and joins it. USB read calls currently
use a 20ms timeout; enumeration/open calls and blocked output are not independently
cancelable. There is no installed autostart or signal-handling service in this change.

Tests cover fresh session IDs after disconnect, suppressed retry status, and
fail-closed channel overflow. The complete 30-test local gate passes, including
formatting, Clippy and dependency audit. Hardware results are recorded below.

## Live recovery — 2026-09-10

The user unplugged and reconnected the Plus during the capture. The worker emitted
connected session 1, disconnected session 1, waiting, then connected session 2.
Press/release events from keys 0–3 arrived on session 2 with fresh elapsed timestamps.
This verifies one real unplug/replug cycle and resumed input without restarting the
command. Raw logs remain ignored under local/. No artwork restoration was attempted.

An opt-in `--pages-demo` mode now adds synchronous page rendering and page_changed/
render_failed records. See page-navigation.md. The --hardware diagnostic retains
its read-only behavior; semantic key dispatch remains pending.

## Display clearing on stop

A graceful daemon stop writes black images to every key and the full touch strip
before releasing the device. Quit Decksmith, Stop Background controls, and normal
session/system shutdown all use that service-stop path. Each surface is attempted
even if another write fails. Saved brightness and layout are unchanged and are
restored by normal startup. Closing only the editor continues to leave background
controls running. An abrupt power loss, forced kill, or unavailable USB transport
cannot guarantee a final display write.
