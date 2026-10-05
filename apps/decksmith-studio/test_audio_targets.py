from copy import deepcopy
import unittest
from unittest.mock import patch
from audio_targets import Snapshot,execute,read,inventory,validate

def node(index,name,volume=50,mute=False,binary=None):
    return {'index':index,'name':name,'description':name,'volume':{'left':{'value':round(volume*65536/100)}},'mute':mute,'properties':{'application.process.binary':binary} if binary else {}}

def snapshot_data(data):
    return {'sinks':data.get('sinks',[]),'sources':data.get('sources',[]),'sink_inputs':data.get('sink-inputs',[])}

class AudioTargetTests(unittest.TestCase):
    def setUp(self):
        self.data={'sinks':[node(1,'speakers')],'sources':[node(2,'microphone'),node(3,'speakers.monitor')],
                   'sink-inputs':[node(4,'music',30,binary='mpz'),node(5,'browser',70,binary='brave'),node(6,'music2',60,mute=True,binary='mpz')]}
    def test_inventory_separates_apps_and_excludes_monitors(self):
        with patch('audio_targets.listing',side_effect=lambda kind:deepcopy(self.data[kind])):
            options=inventory()
        self.assertNotIn('input:speakers.monitor',[o['id'] for o in options])
        self.assertEqual(sum(o['id']=='app:application.process.binary=mpz' for o in options),1)
    def test_app_mute_uses_matching_streams_and_consistent_state(self):
        with patch('audio_targets.list_all',return_value=snapshot_data(self.data)),patch('audio_targets.command') as cmd:
            execute({'type':'audio_mute','target':'app:application.process.binary=mpz'})
        self.assertEqual([c.args for c in cmd.call_args_list],[('set-sink-input-mute','4','1'),('set-sink-input-mute','6','1')])
    def test_volume_is_bounded_and_never_changes_other_apps(self):
        with patch('audio_targets.list_all',return_value=snapshot_data(self.data)),patch('audio_targets.command') as cmd:
            execute({'type':'audio_adjust','target':'app:application.process.binary=mpz','percent':-20})
        self.assertEqual([c.args for c in cmd.call_args_list],[('set-sink-input-volume','4','10%'),('set-sink-input-volume','6','40%')])
    def test_missing_target_does_not_fall_back_to_system(self):
        with patch('audio_targets.list_all',return_value=snapshot_data({})),patch('audio_targets.command') as cmd:
            with self.assertRaises(ValueError):execute({'type':'audio_mute','target':'app:application.process.binary=missing'})
            self.assertEqual(read(['app:application.process.binary=missing']),[None])
        cmd.assert_not_called()
    def test_select_device_checks_existence_and_uses_stable_name(self):
        with patch('audio_targets.list_all',return_value=snapshot_data(self.data)),patch('audio_targets.command') as cmd:
            execute({'type':'audio_select','target':'input:microphone'})
        cmd.assert_called_once_with('set-default-source','microphone')
    def test_rejects_invalid_targets_and_steps(self):
        for target in ('','output:-x','app:bad=value','output:','app:application.id=',None):
            with self.assertRaises(ValueError):validate(target)
        with patch('audio_targets.list_all',return_value=snapshot_data(self.data)),patch('audio_targets.command') as cmd:
            with self.assertRaises(ValueError):execute({'type':'audio_adjust','target':'app:application.process.binary=mpz','percent':100})
        cmd.assert_not_called()

    def test_device_status_follows_defaults_and_missing_devices(self):
        import json
        self.data['sinks'].append(node(7,'headphones'))
        defaults={'default_sink_name':'speakers','default_source_name':'microphone'}
        with patch('audio_targets.list_all',return_value=snapshot_data(self.data)),patch('audio_targets.command',side_effect=lambda *args:json.dumps(defaults)):
            targets=['output:speakers','output:headphones','input:microphone','output:missing']
            states=read(targets)
            self.assertEqual([s['active'] if s else None for s in states],[True,False,True,None])
            defaults['default_sink_name']='headphones'
            self.assertEqual([s['active'] if s else None for s in read(targets)],[False,True,True,None])

    def test_device_icons_follow_metadata_not_names(self):
        from audio_targets import device_icon
        speaker=node(1,'Headphones Fake Name')
        self.assertEqual(device_icon('output:device',[speaker]),'unknown')
        speaker['properties']['device.form_factor']='speaker'
        self.assertEqual(device_icon('output:device',[speaker]),'speaker')
        speaker['active_port']='headphone-port'
        speaker['ports']=[{'name':'headphone-port','type':'Headphones'}]
        self.assertEqual(device_icon('output:device',[speaker]),'headphones')
        self.assertEqual(device_icon('input:device',[speaker]),'microphone')
        self.assertEqual(device_icon('app:application.name=Music',[speaker]),'application')
        speaker['properties']['device.class']='monitor'
        self.assertEqual(device_icon('input:device',[speaker]),'unknown')
    def test_metadata_change_is_part_of_live_state(self):
        target='output:speakers'
        with patch('audio_targets.list_all',return_value=snapshot_data(self.data)),patch('audio_targets.command',return_value='{}'):
            self.assertEqual(read([target])[0]['icon'],'unknown')
            self.data['sinks'][0]['properties']['device.icon_name']='audio-headphones'
            self.assertEqual(read([target])[0]['icon'],'headphones')
            self.data['sinks'][0]['mute']=True
            muted=read([target])[0]
            self.assertTrue(muted['muted']);self.assertEqual(muted['icon'],'headphones')

class PersistentReaderTests(unittest.TestCase):
    def test_shared_snapshot_is_fresh_between_requests(self):
        from io import StringIO
        from audio_targets import serve_cached_read
        import json
        output=StringIO()
        with patch('audio_targets.list_all',side_effect=[snapshot_data({'sinks':[node(1,'speakers',20)]}),snapshot_data({'sinks':[node(1,'speakers',80)]})] ) as listing:
            serve_cached_read(StringIO('["output:speakers","output:speakers"]\n["output:speakers"]\n'),output,None)
        values=[json.loads(line) for line in output.getvalue().splitlines()]
        self.assertEqual(values[0][0]['percent'],20)
        self.assertEqual(values[1][0]['percent'],80)
        self.assertEqual(listing.call_count,2)

    def test_one_combined_listing_serves_all_audio_target_kinds(self):
        values=snapshot_data({'sinks':[node(1,'speakers')],
                              'sources':[node(2,'microphone')],
                              'sink-inputs':[node(3,'music',binary='mpz')]})
        with patch('audio_targets.list_all',return_value=values) as listing,patch('audio_targets.command',return_value='{}'):
            states=read(['output:speakers','input:microphone','app:application.process.binary=mpz','system'])
        self.assertEqual([state['percent'] if state else None for state in states[:3]],[50,50,50])
        self.assertEqual(listing.call_count,1)
    def test_bad_request_does_not_poison_next_read(self):
        from io import StringIO
        from audio_targets import serve_cached_read
        output=StringIO()
        serve_cached_read(StringIO('invalid\n[]\n'),output,None)
        self.assertEqual(output.getvalue(),'null\n[]\n')
    def test_oversized_request_stops_reader(self):
        from io import StringIO
        from audio_targets import serve_cached_read
        output=StringIO()
        serve_cached_read(StringIO(' '*8193+'\n[]\n'),output,None)
        self.assertEqual(output.getvalue(),'')

class PersistentActionTests(unittest.TestCase):
    def serve(self,requests):
        import json
        from io import StringIO
        from audio_targets import serve_execute
        output=StringIO()
        serve_execute(StringIO(''.join(json.dumps(r)+'\n' for r in requests)),output)
        return [json.loads(line) for line in output.getvalue().splitlines()]

    def test_fresh_resolution_preserves_clamping_reversal_and_external_changes(self):
        frames=[snapshot_data({'sinks':[node(1,'speakers',100)]}),
                snapshot_data({'sinks':[node(1,'speakers',100)]}),
                snapshot_data({'sinks':[node(1,'speakers',40)]})]
        with patch('audio_targets.list_all',side_effect=frames) as listing,patch('audio_targets.command') as cmd:
            replies=self.serve([{'type':'audio_adjust','target':'output:speakers','percent':amount} for amount in (1,-1,1)])
        self.assertEqual(replies,[{'status':'ok'}]*3)
        self.assertEqual(listing.call_count,3)
        self.assertEqual([c.args for c in cmd.call_args_list],
                         [('set-sink-volume','1','100%'),('set-sink-volume','1','99%'),('set-sink-volume','1','41%')])

    def test_missing_target_timeout_and_later_request_are_distinct_without_retry(self):
        import subprocess
        with patch('audio_targets.execute',side_effect=[ValueError('Audio target unavailable'),subprocess.TimeoutExpired('pactl',.4),None]) as action:
            replies=self.serve([{'type':'audio_mute','target':'output:speakers'}]*3)
        self.assertEqual(replies,[{'status':'error','code':'unavailable'},
                                  {'status':'error','code':'timeout'},{'status':'ok'}])
        self.assertEqual(action.call_count,3)

    def test_bad_protocol_does_not_execute_or_poison_later_request(self):
        bad=[[],{'type':'audio_adjust','target':'output:speakers'},
             {'type':'audio_mute','target':'output:speakers','extra':True},
             {'type':'shutdown','target':'output:speakers'}]
        with patch('audio_targets.execute') as action:
            replies=self.serve(bad+[{'type':'audio_mute','target':'output:speakers'}])
        self.assertEqual(replies,[{'status':'error','code':'failed'}]*4+[{'status':'ok'}])
        action.assert_called_once_with({'type':'audio_mute','target':'output:speakers'})

    def test_boolean_step_and_oversized_or_unterminated_requests_cannot_write(self):
        from io import StringIO
        from audio_targets import serve_execute
        with patch('audio_targets.list_all',return_value=snapshot_data({'sinks':[node(1,'speakers')]})),patch('audio_targets.command') as cmd:
            self.assertEqual(self.serve([{'type':'audio_adjust','target':'output:speakers','percent':True}]),[{'status':'error','code':'failed'}])
        cmd.assert_not_called()
        with patch('audio_targets.execute') as action:
            for text in (' '*8193+'\n','{"type":"audio_mute","target":"output:speakers"}'):
                output=StringIO();serve_execute(StringIO(text),output)
                self.assertEqual(output.getvalue(),'')
        action.assert_not_called()
