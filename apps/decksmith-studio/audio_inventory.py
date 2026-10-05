"""Read-only audio inventory caching, invalidated by libpulse subscriptions.

Only long-lived helpers use this cache. Actions still discover targets afresh.
An unready/disconnected listener disables caching; a five-second refresh also
bounds missed notifications. No audio samples or commands pass through here.
"""
import ctypes as C
import os
import threading
import time


class Watcher:
    def __init__(self):
        self._lock = threading.Lock()
        self._revision = 0
        self._ready = False
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._thread.start()

    def stamp(self):
        with self._lock:
            return self._revision if self._ready else None

    def _changed(self):
        with self._lock:
            self._revision += 1

    def _set_ready(self, ready):
        with self._lock:
            self._ready = ready
            self._revision += 1

    def close(self):
        self._stop.set()
        self._thread.join(timeout=2)
        self._set_ready(False)

    def _run(self):
        try:
            lib = C.CDLL('libpulse.so.0')
            ptr = C.c_void_p
            success_type = C.CFUNCTYPE(None, ptr, C.c_int, ptr)
            event_type = C.CFUNCTYPE(None, ptr, C.c_int, C.c_uint32, ptr)

            def bind(name, result, *args):
                f = getattr(lib, name)
                f.restype = result
                f.argtypes = list(args)
                return f

            new = bind('pa_mainloop_new', ptr)
            api = bind('pa_mainloop_get_api', ptr, ptr)
            iterate = bind('pa_mainloop_iterate', C.c_int, ptr, C.c_int, ptr)
            free = bind('pa_mainloop_free', None, ptr)
            ctxnew = bind('pa_context_new', ptr, ptr, C.c_char_p)
            connect = bind('pa_context_connect', C.c_int, ptr, C.c_char_p, C.c_int, ptr)
            state = bind('pa_context_get_state', C.c_int, ptr)
            disconnect = bind('pa_context_disconnect', None, ptr)
            unref = bind('pa_context_unref', None, ptr)
            set_event = bind('pa_context_set_subscribe_callback', None, ptr, event_type, ptr)
            subscribe = bind('pa_context_subscribe', ptr, ptr, C.c_int, success_type, ptr)
            op_unref = bind('pa_operation_unref', None, ptr)
            server = ('unix:' + os.environ.get('XDG_RUNTIME_DIR', f'/run/user/{os.getuid()}') + '/pulse/native').encode()

            @event_type
            def event(_context, _kind, _index, _data):
                self._changed()

            @success_type
            def subscribed(_context, success, _data):
                self._set_ready(bool(success))

            while not self._stop.is_set():
                loop = context = None
                self._set_ready(False)
                try:
                    loop = new()
                    if not loop:
                        raise RuntimeError('Audio listener unavailable')
                    context = ctxnew(api(loop), b'Decksmith audio inventory')
                    if not context or connect(context, server, 0, None) < 0:
                        raise RuntimeError('Audio listener unavailable')
                    requested = False
                    deadline = time.monotonic() + 1
                    while not self._stop.is_set():
                        if iterate(loop, 0, None) < 0 or state(context) in (5, 6):
                            break
                        if state(context) == 4 and not requested:
                            set_event(context, event, None)
                            # Sink, source, sink input, server/defaults, card. Client
                            # and source-output events would include our own queries.
                            operation = subscribe(context, 1 | 2 | 4 | 128 | 512, subscribed, None)
                            if not operation:
                                break
                            op_unref(operation)
                            requested = True
                        if self.stamp() is None and time.monotonic() >= deadline:
                            break
                        self._stop.wait(.05)
                except Exception:
                    pass
                finally:
                    self._set_ready(False)
                    if context:
                        disconnect(context)
                        unref(context)
                    if loop:
                        free(loop)
                self._stop.wait(.5)
        except Exception:
            self._set_ready(False)


class Inventory:
    def __init__(self, watcher=None, clock=time.monotonic):
        self.watcher = watcher if watcher is not None else Watcher()
        self.clock = clock
        self.values = {}

    def get(self, key, load):
        revision = self.watcher.stamp()
        now = self.clock()
        previous = self.values.get(key)
        if revision is not None and previous is not None:
            old_revision, checked, value = previous
            if revision == old_revision and 0 <= now - checked < 5:
                return value
        # Discard on failure; never retain a successful but stale snapshot after
        # an unavailable server or a notification arriving during the query.
        self.values.pop(key, None)
        value = load()
        if revision is not None and self.watcher.stamp() == revision:
            self.values[key] = (revision, now, value)
        return value

    def close(self):
        self.watcher.close()
