import json,unittest
from profile_client import DeviceClient
from editor_model import Draft
from pathlib import Path

class ProfileClientTests(unittest.TestCase):
    def client(self,identity='a'*32,model='mk2'):
        self.calls=[]
        def transport(method,args=None):
            values=args.unpack() if args is not None else ()
            self.calls.append((method,values))
            if method=='Preview':return bytes(15*72*72*3)
            if method=='ListDevices':return (json.dumps({'sessions':[{'id':identity,'connected':True,'page':1}]}),)
            return (json.dumps({'id':identity,'token':{'runtime':'c'*32,'revision':2,'connection':1,'binding':3},'layout':{'n':184467440737095516160001}}),)
        return DeviceClient({'id':identity,'model':model,'name':'Test 🎵','brightness':52},transport)
    def test_exact_owner_on_preview_save_page_and_read_with_no_fallback(self):
        from gi.repository import GLib
        c=self.client();self.assertIn('184467440737095516160001',c('GetLayout')[0]);c.page=lambda:1
        c('SaveLayout',GLib.Variant('(s)',('{}',)));c('ShowPage',GLib.Variant('(y)',(1,)))
        c('PreviewKeys',GLib.Variant('(sy)',('{}',0)))
        for method,args in self.calls:
            if method!='ListDevices':self.assertEqual(args[0],'a'*32)
        self.assertEqual(self.calls[1][1][3],1)
        self.assertEqual(json.loads(self.calls[2][1][1])['revision'],2)
        with self.assertRaises(ValueError):c('TestControl')
    def test_wrong_device_reply_never_updates_revision(self):
        c=self.client();self.assertIsNone(c.token)
        with self.assertRaises(ValueError):c.accept((json.dumps({'id':'b'*32,'token':{'revision':17}}),))
        self.assertIsNone(c.token)
    def test_save_without_load_never_sends(self):
        from gi.repository import GLib
        c=self.client()
        with self.assertRaises(ValueError):c('SaveLayout',GLib.Variant('(s)',('{}',)))
        self.assertEqual(self.calls,[])
    def test_connected_client_never_suppresses_explicit_page_navigation(self):
        from gi.repository import GLib
        c=self.client();c.connected_workspace=True;c.page=lambda:1
        c('GetLayout');c('SaveLayout',GLib.Variant('(s)',('{}',)))
        c('ShowPage',GLib.Variant('(y)',(1,)))
        self.assertEqual([method for method,_ in self.calls],['GetProfile','SaveProfile','ShowPage'])
        c('ShowPage',GLib.Variant('(y)',(0,)))
        self.assertEqual(self.calls[-1][0],'ShowPage')
    def test_status_uses_display_acknowledgement_not_input_readiness(self):
        def transport(method,args=None):return (json.dumps({'sessions':[{'id':'a'*32,'connected':True,'input_ready':True,'display_ready':False,'page':0}]}),)
        c=DeviceClient({'id':'a'*32,'model':'mk2','name':'Test','brightness':50},transport)
        self.assertFalse(json.loads(c('GetStatus')[0])['display_ready'])
    def test_mk2_page_operations_keep_fifteen_and_reject_dials(self):
        doc=json.loads((Path(__file__).resolve().parents[2]/'config/mk2-default.json').read_text())
        draft=Draft(doc);draft.add_page();self.assertEqual(len(draft.data['pages'][-1]['keys']),15);draft.validate()
        draft.data['dials']=[]
        with self.assertRaises(ValueError):draft.validate()

    def test_workspace_status_never_claims_physical_connection_or_action_readiness(self):
        c=self.client();c.workspace_only=True
        status=json.loads(c('GetStatus')[0]);self.assertTrue(status['workspace_only']);self.assertFalse(status['running'])
        self.assertFalse(c.actions_available)
        from workspace_shell import save_reason
        self.assertIsNone(save_reason(True,True,False,{'workspace_only':True,'running':False,'connected':False}))
        self.assertIsNotNone(save_reason(True,False,False,{'workspace_only':True,'running':False}))
