# Device-workspace restart and pressure testing — October 13, 2026

After publishing the device foundation and saved workspace in source commit
`58bdfea`, continued isolated AI-server testing found two issues in the new profile
endpoints. Installed V1, production configuration and physical controls were not
changed by these tests.

## Endpoint ownership

A duplicate workspace process could replace the first service's bus name and
proceed with a different store. The pinned bus library defaults include both
allowing replacement and replacing an existing owner. Profile endpoints now
explicitly disable both flags; the nonqueueing request remains enabled.

The permanent native check verifies that a duplicate exits unsuccessfully before
creating its destination and cannot replace the original runtime token. The
restart check verifies the same behavior against a reopened persistent workspace.
The change covers the saved workspace, its isolated acceptance fixture and the
virtual profile lab. Existing production Control1 behavior is outside this slice.

## Concurrent replies

A burst of preview requests exposed a client timeout. Synchronous waits inside
bus handlers could block the executor needed to process/transmit other replies.
Profile handlers now await a bounded asynchronous reply channel and deadline.
The service operation queue remains eight; each reply channel holds one result.
Overflow returns a clear limit/busy error. The five-second deadline still cancels
queued work and reports an uncertain outcome if execution may already have started.
No automatic replay or default-device fallback was added.

The asynchronous channel/timer/future libraries were already present through the
bus dependency. Their existing locked versions are reused; no new dependency
version or separate runtime was introduced. Persistent writes and rendering still
execute serially in the workspace host. This fixes bus reply starvation; it does
not complete asynchronous physical rendering or persistent-write scheduling.

## Results

The final immutable candidate source snapshot
`37df542ae9fd1cc2caccde95e2c8a699d57987d706ece81e1ebd869348768042`
passed the complete AI-server checks: **273 Rust cases, zero failures, four
intentionally ignored environment/hardware/stress cases**, strict Clippy, format,
build, **192 Studio** and **111 script** cases, and all three native workspace
regressions. One conditional Python skip remains in each Python suite.

The permanent native check additionally issued **48 preview requests through
16 concurrent callers**: nine completed and 39 received bounded busy rejection,
with no timeout or unexpected error. A new Rust regression proves the reply wait
yields before the host answers, rejects a full queue and marks an expired queued
request cancelled with its receiver closed.

The repeated persistent-workspace check passed **20 service restarts across ten
edit/recovery cycles**. It retained opaque device IDs, saved selection, page,
brightness and edits to Key 15; rejected previous-runtime and stale same-owner
saves; preserved the other profile; archived verified history; and stopped cleanly
with the bus name released. Its separate 48-request preview burst also returned
nine completed replies and 39 bounded busy rejections without timeouts.

Separately, the initial source commit passed the explicit **300-second maximum
roster stress**: **131,854 reconnect cycles across eight alternating virtual Plus/
MK.2 devices**, reversed inventories, held-input fences, stale feedback rejection
and unchanged other-device contexts. Its overall combined job recorded failure
because that run found the endpoint-ownership bug; only the independent stress
check is accepted from that job. The intermediate retry's timeout is retained as
failure evidence; the final corrected restart/pressure outcomes are accepted.

All outcomes and logs are retained privately. No VM, physical HID handle, real
audio target or scheduled checkout was changed. The test services were stopped.
These results qualify the new development workspace seams, not full production
multi-device action execution or physical paired lock/suspend/resource behavior.
