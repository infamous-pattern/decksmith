"""Coalesce validation speech without polling, moving focus or repeating errors."""

class ValidationAnnouncements:
    def __init__(self, schedule, cancel, speak):
        self.schedule = schedule
        self.cancel = cancel
        self.speak = speak
        self.message = None
        self.pending = None

    def update(self, message):
        if message == self.message:
            return
        self.close()
        self.message = message
        if message:
            self.pending = self.schedule(700, self.deliver)

    def deliver(self):
        self.pending = None
        if self.message:
            self.speak(self.message)
        return False

    def close(self):
        if self.pending is not None:
            self.cancel(self.pending)
            self.pending = None
        self.message = None
