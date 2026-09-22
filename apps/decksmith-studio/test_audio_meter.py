import unittest
from unittest.mock import patch
import audio_meter as m

class MeterTests(unittest.TestCase):
    def test_db_scale_is_signal_not_volume(self):
        self.assertEqual([m.display_level(v) for v in (None,0,.001,.01,.1,1)], [None,0,0,33,67,100])
    def test_missing_stale_and_silent_are_distinct(self):
        self.assertIsNone(m.aggregate([],{},1))
        self.assertIsNone(m.aggregate(['a'],{'a':(0.,.8)},1))
        self.assertEqual(m.aggregate(['a'],{'a':(.9,0)},1),0)
        self.assertIsNone(m.aggregate(['a','b'],{'a':(.9,.8)},1))
        self.assertEqual(m.aggregate(['a','b'],{'a':(.9,.1),'b':(.9,.01)},1),67)
    def test_device_and_multiple_app_streams_resolve_exact_sources(self):
        class Snapshot:
            cache={}
            def nodes(self,target):
                return {'system':('sinks',[{'monitor_source':'out.monitor'}]),'microphone':('sources',[{'name':'mic'}]),'app:x':('sink-inputs',[{'sink':3,'index':8},{'sink':4,'index':9}]),'missing':('sources',[])}[target]
        with patch.object(m,'Snapshot',Snapshot),patch.object(m,'listing',return_value=[{'index':3,'monitor_source':'a.monitor'},{'index':4,'monitor_source':'b.monitor'}]):
            self.assertEqual(m.resolve(['system','microphone','app:x','missing']),{'system':[('out.monitor',None)],'microphone':[('mic',None)],'app:x':[('a.monitor',8),('b.monitor',9)],'missing':[]})
    def test_inventory_failure_is_not_zero_or_other_source(self):
        with patch.object(m,'Snapshot',side_effect=RuntimeError):
            with self.assertRaises(RuntimeError):m.resolve(['system'])
        class Snapshot:
            def nodes(self,target):raise RuntimeError()
        with patch.object(m,'Snapshot',Snapshot):self.assertEqual(m.resolve(['system']),{'system':[]})

    def test_startup_retry_is_bounded_and_does_not_restart_ready_streams(self):
        for state in (0,1):
            self.assertFalse(m.needs_restart(state,10,12.99))
            self.assertTrue(m.needs_restart(state,10,13))
            # A recreated stream gets a fresh deadline.
            self.assertFalse(m.needs_restart(state,13,13.99))
        self.assertFalse(m.needs_restart(2,10,100))
        for state in (None,3,4):self.assertTrue(m.needs_restart(state,10,10))

    def test_source_move_or_failure_cannot_substitute_another_device(self):
        class Pulse:
            state=2
            name=b'wave'
            def sstate(self,stream):return self.state
            def source(self,stream):return self.name
        pulse=Pulse();key=('wave',None)
        self.assertTrue(m.correct_source(pulse,1,key))
        pulse.name=b'other-input'
        self.assertFalse(m.correct_source(pulse,1,key))
        pulse.name=b'wave'
        for state in (0,1,3,4):
            pulse.state=state;self.assertFalse(m.correct_source(pulse,1,key))
        self.assertFalse(m.correct_source(pulse,None,key))

    def test_brief_packet_gap_decays_but_eventually_becomes_unavailable(self):
        sample={'a':(10.,.1)}
        levels=[m.aggregate(['a'],sample,t) for t in (10.,10.1,10.5,10.9)]
        self.assertEqual(levels[:2],[67,67])
        self.assertGreater(levels[1],levels[2])
        self.assertGreater(levels[2],levels[3])
        self.assertIsNone(m.aggregate(['a'],sample,11.))
        self.assertIsNone(m.aggregate(['a'],{},10.1))
        self.assertIsNone(m.aggregate(['a'],sample,9.))
        self.assertEqual(m.aggregate(['a'],{'a':(10.,0.)},10.9),0)

    def test_callback_coalesces_peaks_and_consumes_once(self):
        pulse=object.__new__(m.Pulse);pulse.peaks={}
        values=iter([.2,.8,.3,None]);pulse.read_peak=lambda stream:next(values)
        pulse.sstate=lambda stream:2;pulse.suspended=lambda stream:0
        for _ in range(4):pulse.capture(1,4,None)
        self.assertEqual(pulse.level(1),.8)
        self.assertIsNone(pulse.level(1))
        pulse.peaks[1]=.9;pulse.sstate=lambda stream:3
        self.assertIsNone(pulse.level(1));self.assertEqual(pulse.peaks,{})
        pulse.peaks[1]=.9;pulse.sstate=lambda stream:2;pulse.suspended=lambda stream:1
        self.assertEqual(pulse.level(1),0.);self.assertEqual(pulse.peaks,{})
