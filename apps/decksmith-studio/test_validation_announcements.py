import unittest
from validation_announcements import ValidationAnnouncements

class ValidationSpeechTests(unittest.TestCase):
    def setUp(self):
        self.callbacks={};self.spoken=[];self.serial=0
        def schedule(delay,callback):
            self.assertEqual(delay,700);self.serial+=1;self.callbacks[self.serial]=callback;return self.serial
        self.speech=ValidationAnnouncements(schedule,self.callbacks.pop,self.spoken.append)
    def fire(self):
        callbacks=list(self.callbacks.values());self.callbacks.clear()
        for callback in callbacks:self.assertFalse(callback())
    def test_repeated_status_updates_announce_once(self):
        for _ in range(10):self.speech.update('Cannot save: Label is empty.')
        self.assertEqual(len(self.callbacks),1);self.fire()
        self.speech.update('Cannot save: Label is empty.');self.fire()
        self.assertEqual(len(self.spoken),1)
    def test_correcting_before_delay_cancels_stale_error(self):
        self.speech.update('Invalid');self.speech.update(None);self.fire()
        self.assertEqual(self.spoken,[])
    def test_latest_error_replaces_previous(self):
        self.speech.update('Label');self.speech.update('URL');self.fire()
        self.assertEqual(self.spoken,['URL'])
    def test_corrected_then_invalid_again_announces_again(self):
        self.speech.update('Label');self.fire();self.speech.update(None)
        self.speech.update('Label');self.fire();self.assertEqual(self.spoken,['Label','Label'])
    def test_close_cancels_and_is_safe_twice(self):
        self.speech.update('Label');self.speech.close();self.speech.close();self.fire()
        self.assertEqual(self.spoken,[])
