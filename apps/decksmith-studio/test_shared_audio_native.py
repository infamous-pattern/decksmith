import io
import unittest
from unittest.mock import patch
import shared_audio_native as shared
import system_sounds


def node(index=3, serial='41', name='Fixture', muted=False):
    return {'index': index, 'properties': {'object.serial': serial, 'application.name': name},
            'volume': {'mono': {'value': 32768}}, 'mute': muted}


class SharedAudioTests(unittest.TestCase):
    target = 'app:application.name=Fixture'

    def run_action(self, action, inventories, decisions, sockets=None, event=None):
        reads = iter(inventories)
        class Snapshot:
            def nodes(self, target):
                return 'sink-inputs', next(reads)
        writes = []
        output = io.StringIO()
        sockets = iter(sockets) if sockets else None
        shared.execute(action, io.StringIO(''.join(__import__('json').dumps({'decision': d})+'\n' for d in decisions)), output,
                       snapshot=Snapshot, command=lambda *args: writes.append(args),
                       socket=lambda: next(sockets) if sockets else (1, 2), event=event)
        return writes, output.getvalue()

    def test_multiple_streams_mute_together_with_per_write_authorization(self):
        first = [node(muted=True), node(4, '42', muted=False)]
        writes, reply = self.run_action({'type': 'audio_mute', 'target': self.target}, [first, first, first], ['apply', 'apply'])
        self.assertEqual(writes, [('set-sink-input-mute', '3', '1'), ('set-sink-input-mute', '4', '1')])
        self.assertEqual(reply.count('ready'), 2)

    def test_missing_replaced_stream_and_server_never_mutate_another_target(self):
        for inventories, sockets in [([[]], None), ([[node()], [node(serial='99')]], None), ([[node()], [node()]], [(1,2),(1,3)])]:
            with self.assertRaisesRegex(ValueError, 'unavailable'):
                self.run_action({'type':'audio_mute','target':self.target}, inventories, ['apply'], sockets)

    def test_abort_after_discovery_prevents_write(self):
        with self.assertRaisesRegex(ValueError, 'Stale action scope'):
            self.run_action({'type':'audio_adjust','target':self.target,'percent':1}, [[node()],[node()]], ['abort'])

    def test_adjustment_clamps_and_accepts_coalesced_same_direction_step(self):
        writes, _ = self.run_action({'type':'audio_adjust','target':self.target,'percent':100}, [[node()],[node()]], ['apply'])
        self.assertEqual(writes, [('set-sink-input-volume','3','100%')])

    def test_event_role_record_and_only_event_streams_change(self):
        class Event:
            def __init__(self): self.writes=[]
            def read(self): return system_sounds.default_info()
            def ready(self): pass
            def write(self, info, *, prepared_only=False):
                self.writes.append((system_sounds.state(info), prepared_only))
        event=Event()
        writes, reply=self.run_action({'type':'audio_adjust','target':'system','percent':-23}, [[node()],[node()]], ['apply','apply'], event=event)
        self.assertEqual(event.writes[0][0]['percent'],77)
        self.assertTrue(event.writes[0][1])
        self.assertEqual(writes,[('set-sink-input-volume','3','77%')])
        self.assertEqual(reply.count('ready'),2)

    def test_oversized_event_inventory_is_rejected_before_persisted_write(self):
        event=unittest.mock.Mock()
        event.read.return_value=system_sounds.default_info()
        with self.assertRaisesRegex(ValueError,'Too many'):
            self.run_action({'type':'audio_mute','target':'system'}, [[node(i,str(i)) for i in range(17)]], ['apply'], event=event)
        event.write.assert_not_called()

    def test_lost_write_acknowledgement_is_uncertain_not_retried(self):
        source=io.StringIO('{"type":"audio_mute","target":"app:application.name=Fixture"}\n{"decision":"apply"}\n')
        output=io.StringIO()
        class Snapshot:
            def nodes(self,target): return 'sink-inputs',[node()]
        with patch.object(shared.audio,'Snapshot',Snapshot), patch.object(shared,'socket_identity',return_value=(1,2)), patch.object(shared.audio,'command',side_effect=TimeoutError):
            shared.serve(source,output)
            self.assertEqual(__import__('json').loads(output.getvalue().splitlines()[-1]),{'status':'error','code':'uncertain'})
            shared.audio.command.assert_called_once()

    def test_legacy_master_assignment_keeps_output_semantics(self):
        class Snapshot:
            def nodes(self,target):
                self.assertion = target
                return 'sinks',[node()]
        output=io.StringIO();writes=[]
        shared.execute({'type':'audio_adjust','target':'default_output','percent':1},io.StringIO('{"decision":"apply"}\n'),output,snapshot=Snapshot,command=lambda *args:writes.append(args),socket=lambda:(1,2))
        self.assertEqual(writes,[('set-sink-volume','3','51%')])


class SharedAudioSelectionTests(unittest.TestCase):
    def test_exact_device_selection_uses_grant_after_fresh_identity(self):
        from unittest.mock import Mock
        n=node();n['name']='speakers'
        snapshot=Mock();snapshot.return_value.nodes.return_value=('sinks',[n])
        writes=Mock();output=io.StringIO()
        shared.execute({'type':'audio_select','target':'output:speakers'},io.StringIO('{"decision":"apply"}\n'),output,snapshot=snapshot,command=writes,socket=lambda:(1,2))
        writes.assert_called_once_with('set-default-sink','speakers')
        self.assertEqual(output.getvalue().count('ready'),1)

    def test_missing_ambiguous_replaced_or_stale_selection_does_not_change_default(self):
        from unittest.mock import Mock
        n=node();n['name']='speakers';replacement=dict(n,properties={'object.serial':'99'})
        for inventory,decision in [([[],[]],'apply'),([[n,n],[n]],'apply'),([[n],[replacement]],'apply'),([[n],[n]],'abort')]:
            snapshot=Mock();snapshot.return_value.nodes.side_effect=[('sinks',nodes) for nodes in inventory]
            writes=Mock()
            with self.assertRaises(ValueError):
                shared.execute({'type':'audio_select','target':'output:speakers'},io.StringIO(json_line(decision)),io.StringIO(),snapshot=snapshot,command=writes,socket=lambda:(1,2))
            writes.assert_not_called()


def json_line(decision):return __import__('json').dumps({'decision':decision})+'\n'


if __name__ == '__main__':
    unittest.main()
