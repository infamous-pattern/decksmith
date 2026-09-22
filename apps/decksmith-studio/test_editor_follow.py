import unittest
from types import SimpleNamespace
from copy import deepcopy
from editor import Editor
from editor_model import Draft

class ForegroundFollowTests(unittest.TestCase):
    def test_device_page_follow_retains_unsaved_draft_and_does_not_send_navigation(self):
        page={'name':'HOME','background':[1,2,3],'keys':[{'label':'Test','action':{'type':'none'}} for _ in range(8)]}
        second=deepcopy(page);second['name']='WORK'
        draft=Draft({'version':1,'pages':[page,second]})
        draft.data['pages'][0]['keys'][0]['label']='Unsaved'
        draft.data['pages'][1]['application']='teams.desktop'
        before=deepcopy(draft.data);rebuilds=[]
        editor=SimpleNamespace(pending=False,draft=draft,device_layout='custom',device_page=0,page=0,key=3,rebuild=lambda:rebuilds.append(True))
        Editor.follow_status(editor,{'display_ready':True,'layout':'custom','active_page':1})
        self.assertEqual(editor.page,1);self.assertEqual(draft.data,before);self.assertTrue(draft.dirty)
        self.assertEqual(rebuilds,[True])
        Editor.follow_status(editor,{'display_ready':True,'layout':'custom','active_page':1})
        self.assertEqual(rebuilds,[True])
