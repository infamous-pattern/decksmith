"""Shared desktop audio event subscription, active only while a picker is visible."""
import atexit
import os
import re
import subprocess
import weakref
from gi.repository import GLib

TOPOLOGY=re.compile(rb"Event '(?:new|remove)' on (?:sink|source|sink-input|source-output) #|Event 'change' on server #")

class AudioMonitor:
    def __init__(self):
        self.pickers=weakref.WeakSet()
        self.process=None;self.watch=0;self.retry=0;self.debounce=0;self.recheck=0;self.buffer=b''
    def add(self,picker):
        self.pickers.add(picker)
        picker.refresh()
        if not self.recheck:self.recheck=GLib.timeout_add_seconds(2,self.reconcile)
        if self.process is None and not self.retry:self.start()
    def reconcile(self):
        # Events are an optimization, not the only route to recovering stale lists.
        for picker in list(self.pickers):picker.refresh()
        return True
    def remove(self,picker):
        self.pickers.discard(picker)
        if not self.pickers:self.close()
    def start(self):
        self.retry=0
        if not self.pickers:return False
        try:
            self.process=subprocess.Popen(['/usr/bin/pactl','subscribe'],stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,env=dict(os.environ,LC_ALL='C'))
            os.set_blocking(self.process.stdout.fileno(),False)
            self.watch=GLib.io_add_watch(self.process.stdout,GLib.IO_IN|GLib.IO_HUP|GLib.IO_ERR,self.receive)
            self.schedule()
        except OSError:
            self.stop_process()
            self.retry=GLib.timeout_add_seconds(5,self.start)
        return False
    def receive(self,source,condition):
        try:data=os.read(source.fileno(),65536)
        except BlockingIOError:return True
        except OSError:data=b''
        if not data or condition & (GLib.IO_HUP|GLib.IO_ERR):
            self.watch=0
            self.stop_process()
            if self.pickers:self.retry=GLib.timeout_add_seconds(5,self.start)
            return False
        self.buffer+=data
        lines=self.buffer.split(b'\n');self.buffer=lines.pop()[-4096:]
        if any(TOPOLOGY.search(line) for line in lines):self.schedule()
        return True
    def schedule(self):
        # A fixed window coalesces bursts without indefinitely postponing updates.
        if not self.debounce:self.debounce=GLib.timeout_add(250,self.dispatch)
    def dispatch(self):
        self.debounce=0
        for picker in list(self.pickers):picker.refresh()
        return False
    def stop_process(self):
        process,self.process=self.process,None
        self.buffer=b''
        if process is not None:
            if process.poll() is None:process.kill()
            process.wait()
            process.stdout.close()
    def close(self):
        for name in ('watch','retry','debounce','recheck'):
            identifier=getattr(self,name)
            if identifier:GLib.source_remove(identifier);setattr(self,name,0)
        self.stop_process()

MONITOR=AudioMonitor()
atexit.register(MONITOR.close)
