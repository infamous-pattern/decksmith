import subprocess
import unittest
from unittest.mock import Mock,patch
from audio_apps import discover
from audio_targets import inventory

def app(identity,name,executable,categories='AudioVideo;',visible=True):
    item=Mock()
    item.get_id.return_value=identity;item.get_display_name.return_value=name
    item.get_executable.return_value=executable;item.get_categories.return_value=categories
    item.should_show.return_value=visible
    return item

class AudioAppsTests(unittest.TestCase):
    def test_only_current_visible_audio_apps_and_no_wrapper_targets(self):
        entries=[app('com.brave.Browser.desktop','Brave','/usr/bin/flatpak','WebBrowser;'),
                 app('writer.desktop','Writer','writer','Office;'),
                 app('unknown.desktop','Unknown','/usr/bin/flatpak'),
                 app('hidden.desktop','Hidden','hidden',visible=False)]
        self.assertEqual(discover(entries),[{'id':'app:application.process.binary=brave','name':'Brave','desktop_id':'com.brave.Browser.desktop'}])
        self.assertEqual(discover([]),[])
    def test_native_apps_and_known_runtime_aliases(self):
        result=discover([app('google-chrome.desktop','Chrome','google-chrome-stable','WebBrowser;'),app('mpv.desktop','mpv','/usr/bin/mpv')])
        self.assertEqual([i['id'] for i in result],['app:application.process.binary=chrome','app:application.process.binary=mpv'])
    def test_duplicate_installations_share_runtime_target(self):
        result=discover([app('com.brave.Browser.desktop','Brave','flatpak'),app('brave-browser.desktop','Brave','brave-browser')])
        self.assertEqual(len(result),1)
    def test_installed_live_merge_and_uninstall(self):
        installed=[{'id':'app:application.process.binary=brave','name':'Brave'}]
        stream={'properties':{'application.process.binary':'brave','application.id':'com.brave.Browser','application.name':'Brave'}}
        with patch('audio_apps.discover',return_value=installed),patch('audio_targets.listing',return_value=[]):
            result=inventory();self.assertEqual(result[-1]['label'],'App · Brave · Installed')
        with patch('audio_apps.discover',return_value=installed),patch('audio_targets.listing',side_effect=lambda kind:[stream] if kind=='sink-inputs' else []):
            result=[i for i in inventory() if i['id'].startswith('app:')]
            self.assertEqual(len(result),1)
            self.assertEqual(result[0]['label'],'App · Brave · Audio detected')
            self.assertEqual(result[0]['id'],installed[0]['id'])
        with patch('audio_apps.discover',return_value=[]),patch('audio_targets.listing',return_value=[]):
            self.assertFalse(any(i['id'].startswith('app:') for i in inventory()))
    def test_installed_apps_remain_available_when_audio_service_is_down(self):
        with patch('audio_apps.discover',return_value=[{'id':'app:application.process.binary=brave','name':'Brave'}]),patch('audio_targets.listing',side_effect=subprocess.TimeoutExpired('pactl',.4)):
            self.assertEqual(inventory()[-1]['label'],'App · Brave · Installed')
