# Homebridge failure handling — October 6, 2026

The original physical response recording contained four successful experimental
accessory toggles and two failed inputs. One failure was `plugin_busy`: another
command was still active. The other was `plugin_failed`, which previously combined
multiple failure causes. No companion incident records survived; the underlying
cause of that particular failure cannot be established retrospectively.

Review found that generic accessory assignments treated a terminal plugin
rejection as an uncertain write, requiring manual recovery even when the host and
accessory were still healthy. They now follow the existing legacy-control policy:
one independent read, no retry, and permission for a new input only if the same
authenticated host remains healthy. The read has a one-second recovery deadline.
A host change during that read, lost reply, cancellation or failed confirmation
retains the recovery gate. Rejected commands are never queued or replayed.

The companion keeps a bounded 16-operation trace and writes privacy-safe failure
records to its journal. Fixed operation/phase/reason codes and elapsed time are
included; identities, names, addresses, credentials and raw error text are omitted.
The bridge also distinguishes invalid input, busy/unavailable admission and
client/deadline cancellation in its local diagnostics. No protocol, polling-rate,
layout or device-assignment change is needed.

## Validation

- 49 Python regression tests pass on the Fedora desktop and Fedora 44 VM against
  disposable loopback fixtures. The suite includes the pinned plugin binary
  returning a negative acknowledgement, then accepting a new user input with no
  replay of the rejected command.
- Private Unix-socket tests cover rejection/read recovery, EOF cancellation,
  busy/malformed admission, one-write limits and restricted socket permissions.
- Fault tests cover lost acknowledgement, failed read, host/epoch/authentication
  changes, cancellation, bounded diagnostics and omission of private error data.
- Four companion installer/runtime tests pass, including rollback after a failed
  activation and protection of live sockets and existing files.
- Older read-failure fixtures now inject faults through the current catalogue
  path. Whole-server outage tests permit a supervised reconnect while still
  requiring restored readiness and zero replayed writes.

Desktop activation installed companion `01e396045147cdd7`. Homebridge readiness
and discovery recovered without restarting the main daemon; all five saved JSON
configuration files retained their hashes. The previous package and unit backup
remain available. The user then confirmed that existing Homebridge toggle keys
and the ordinary-speed brightness dial work correctly and remain available.
All four test VMs are verified shut down.

These checks validate the recovery change and ordinary physical operation. They
do not prove the original incident was a terminal rejection; the new diagnostics
are needed to classify any recurrence.
