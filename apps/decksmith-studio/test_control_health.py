import unittest
from unittest.mock import patch
from gi.repository import GLib
from control_health import probe
from control_feedback import entries
from audio_targets import Snapshot

def check(kind='audio',target='app:application.process.binary=brave',command='',slot=0):
    return {'page':0,'slot':slot,'label':'Brave','kind':kind,'target':target,'command':command}
def node(index=1,name='Speakers',binary='brave'):
    return {'index':index,'name':name,'description':name,'properties':{'application.process.binary':binary,'application.name':'Brave'},'volume':{'front-left':{'value':32768}},'mute':False}
class ControlHealthTests(unittest.TestCase):
    def test_stream_absence_restart_and_multiple_streams_use_fresh_matching(self):
        current=[]
        with patch('audio_targets.listing',side_effect=lambda kind:list(current)):
            self.assertEqual(probe([check()])[0]['status'],'idle')
            current.extend([node(71),node(88)])
            self.assertEqual(probe([check()])[0]['status'],'available')
            current[:]=[node(99,binary='unrelated')]
            self.assertEqual(probe([check()])[0]['status'],'idle')
            current[:]=[node(121)]
            self.assertEqual(probe([check()])[0]['status'],'available')
    def test_named_device_reconnect_and_system_default_changes(self):
        devices=[node(name='first')];default={'default_sink_name':'first'}
        with patch('audio_targets.listing',side_effect=lambda _:devices),patch('audio_targets.command',side_effect=lambda *a:__import__('json').dumps(default)):
            c=check(target='default_output');self.assertEqual(probe([c])[0]['target_name'],'first')
            devices[:]=[node(name='second')];default['default_sink_name']='second'
            self.assertEqual(probe([c])[0]['target_name'],'second')
            named=check(target='output:first');self.assertEqual(probe([named])[0]['status'],'missing')
            devices.append(node(name='first'));self.assertEqual(probe([named])[0]['status'],'available')
    def test_unknown_status_is_not_success_and_does_not_change_audio(self):
        with patch('audio_targets.listing',side_effect=RuntimeError('offline')),patch('audio_targets.execute') as execute:
            result=probe([check()]);self.assertEqual(result[0]['status'],'unknown');execute.assert_not_called()
    def test_media_unsupported_does_not_switch_to_other_player_and_recovers(self):
        good={'PlaybackStatus':'Playing','CanControl':True,'CanGoNext':True}
        data=[('org.mpris.MediaPlayer2.mpz',dict(good,CanGoNext=False)),('org.mpris.MediaPlayer2.brave',good)]
        class Bus:
            def call_sync(self,*args):return GLib.Variant('(v)',(GLib.Variant('s','MPZ Music Player'),))
        with patch('media.players',side_effect=lambda bus:list(data)),patch('media.preferred_player',return_value='org.mpris.MediaPlayer2.mpz'):
            c=check('media','automatic','next');result=probe([c],bus=Bus())[0]
            self.assertEqual(result['status'],'unsupported');self.assertEqual(result['target_name'],'MPZ Music Player')
            data[0][1]['CanGoNext']=True;self.assertEqual(probe([c],bus=Bus())[0]['status'],'available')
            data[:]=[data[1]];explicit=check('media','org.mpris.MediaPlayer2.mpz','next')
            self.assertEqual(probe([explicit],bus=Bus())[0]['status'],'missing')
            data.append(('org.mpris.MediaPlayer2.mpz.instance77',good));self.assertEqual(probe([explicit],bus=Bus())[0]['status'],'available')
    def test_panel_reports_saved_failure_and_hides_disconnected_notices(self):
        issue={'check':check(slot=9),'target_name':'Brave','status':'failed','detail':'Timed out; completion unknown','hint':'Check before retrying'}
        status={'running':True,'connected':True,'attention':[issue,issue]}
        rows=entries(status);self.assertEqual(len(rows),1);self.assertIn('Dial 2',rows[0][0]);self.assertIn('Last attempt:',rows[0][1])
        status['connected']=False;self.assertEqual(entries(status),[])


class PersistentHealth(unittest.TestCase):
    def test_each_request_is_fresh_and_invalid_input_recovers(self):
        import io
        from unittest.mock import patch
        import control_health
        source=io.StringIO('[]\ninvalid\n[]\n');output=io.StringIO()
        with patch.object(control_health,'probe',side_effect=[[],[{'fresh':True}]]) as probe:
            control_health.serve(source,output)
        self.assertEqual(probe.call_count,2)
        self.assertEqual(output.getvalue().splitlines(),['[]','null','[{"fresh": true}]'])
    def test_oversized_request_terminates(self):
        import io
        import control_health
        output=io.StringIO();control_health.serve(io.StringIO('x'*65537+'\n'),output)
        self.assertEqual(output.getvalue(),'')
