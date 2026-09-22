import unittest
from media import choose,execute,IFACE
from gi.repository import GLib

def props(state):
    return {'PlaybackStatus':state,'CanControl':True,'CanPause':True,'CanPlay':True,'CanGoNext':True,'CanGoPrevious':True}

class MediaTests(unittest.TestCase):
    def test_prefers_playing_and_does_not_skip_to_another_player_for_unsupported_action(self):
        playing=props('Playing');playing['CanGoNext']=False
        candidates=[('org.mpris.MediaPlayer2.a',props('Paused')),('org.mpris.MediaPlayer2.z',playing)]
        self.assertEqual(choose(candidates,'play_pause'),'org.mpris.MediaPlayer2.z')
        with self.assertRaises(ValueError):choose(candidates,'next')
        with self.assertRaises(ValueError):choose([],'play_pause')
        with self.assertRaises(ValueError):choose(candidates,'run')
    def test_resume_and_next_stay_with_the_previously_controlled_player(self):
        browser='org.mpris.MediaPlayer2.browser'
        audio='org.mpris.MediaPlayer2.music'
        candidates=[(browser,props('Paused')),(audio,props('Playing'))]
        selected=choose(candidates,'play_pause')
        self.assertEqual(selected,audio)
        candidates[1][1]['PlaybackStatus']='Paused'
        self.assertEqual(choose(candidates,'play_pause',selected),audio)
        self.assertEqual(choose(candidates,'next',selected),audio)
        self.assertEqual(choose(candidates[:1],'play_pause',selected),browser)

    def test_sends_exact_method_to_one_player(self):
        class Bus:
            def __init__(self):self.sent=[]
            def call_sync(self,name,path,interface,method,*args):
                if method=='ListNames':return GLib.Variant('(as)',(['org.mpris.MediaPlayer2.test'],))
                if method=='GetAll':return GLib.Variant('(a{sv})',({k:GLib.Variant('s' if isinstance(v,str) else 'b',v) for k,v in props('Playing').items()},))
                self.sent.append((name,interface,method))
        for action,method in [('play_pause','PlayPause'),('next','Next'),('previous','Previous')]:
            bus=Bus();execute(action,bus)
            self.assertEqual(bus.sent,[('org.mpris.MediaPlayer2.test',IFACE,method)])

    def test_explicit_player_stays_in_app_and_uses_unique_owner(self):
        from media import family,valid_player
        self.assertEqual(family('org.mpris.MediaPlayer2.brave.instance123'),'org.mpris.MediaPlayer2.brave')
        self.assertFalse(valid_player('org.mpris.MediaPlayer2.brave;other'))
        class Bus:
            def __init__(self):
                self.data={'org.mpris.MediaPlayer2.brave.instance123':props('Playing'),'org.mpris.MediaPlayer2.mpz':props('Paused')};self.sent=[]
            def call_sync(self,name,path,interface,method,*args):
                if method=='ListNames':return GLib.Variant('(as)',(list(self.data),))
                if method=='GetAll':return GLib.Variant('(a{sv})',({k:GLib.Variant('s' if isinstance(v,str) else 'b',v) for k,v in self.data[name].items()},))
                if method=='GetNameOwner':return GLib.Variant('(s)',(':1.42' if args[0].unpack()[0].endswith('mpz') else ':1.43',))
                self.sent.append((name,method))
        bus=Bus();execute('play_pause',bus,target='org.mpris.MediaPlayer2.mpz')
        self.assertEqual(bus.sent,[(':1.42','PlayPause')])
        bus.sent=[];del bus.data['org.mpris.MediaPlayer2.mpz']
        with self.assertRaises(ValueError):execute('play_pause',bus,target='org.mpris.MediaPlayer2.mpz')
        self.assertEqual(bus.sent,[])
        bus.data['org.mpris.MediaPlayer2.mpz']=props('Paused');bus.data['org.mpris.MediaPlayer2.mpz']['CanGoNext']=False
        with self.assertRaises(ValueError):execute('next',bus,target='org.mpris.MediaPlayer2.mpz')
        self.assertEqual(bus.sent,[])
        execute('play_pause',bus,target='org.mpris.MediaPlayer2.brave')
        self.assertEqual(bus.sent,[(':1.43','PlayPause')])
