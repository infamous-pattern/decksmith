"""Draft editing without file, device or desktop side effects."""
from copy import deepcopy
import json
import zlib
from time import monotonic
from urllib.parse import urlsplit

class Draft:
    def __init__(self, data):
        self.clipboard = None
        self.data = deepcopy(data)
        for page in self.data['pages']:
            for key in page['keys']:
                action = key['action']
                if action['type'] == 'go_to_page':
                    name = self.data['pages'][action['page']]['name']
                    if key.get('follow_page_name') or key['label'] == name:
                        key['follow_page_name'] = True
                        key['label'] = name
        self.saved = deepcopy(self.data)
        self.origins = list(range(len(self.data["pages"])))
        self.clear_history()

    def clear_history(self):
        self.undo_stack=[];self.redo_stack=[]
        self.current=deepcopy((self.data,self.origins))
        self.last_group=None;self.last_edit=0

    HISTORY_BYTES = 8 * 1024 * 1024

    @staticmethod
    def pack_history(state):
        # Artwork arrays dominate drafts. Compress older states rather than
        # retaining fifty independent Python lists of image bytes.
        return zlib.compress(json.dumps(state,separators=(',',':')).encode(),1)

    def trim_history(self):
        while len(self.undo_stack)>50:self.undo_stack.pop(0)
        while len(self.redo_stack)>50:self.redo_stack.pop(0)
        size=sum(map(len,self.undo_stack))+sum(map(len,self.redo_stack))
        while size>self.HISTORY_BYTES and (self.undo_stack or self.redo_stack):
            stack=self.undo_stack if self.undo_stack else self.redo_stack
            size-=len(stack.pop(0))

    def checkpoint(self, group=None):
        state=(self.data,self.origins)
        if state==self.current:return
        now=monotonic()
        if group is None or group!=self.last_group or now-self.last_edit>0.75:
            self.undo_stack.append(self.pack_history(self.current))
            self.undo_stack=self.undo_stack[-50:]
        self.current=deepcopy(state)
        self.redo_stack=[];self.last_group=group;self.last_edit=now
        self.trim_history()

    def travel(self, redo=False):
        self.checkpoint()
        source=self.redo_stack if redo else self.undo_stack
        target=self.undo_stack if redo else self.redo_stack
        if not source:return False
        target.append(self.pack_history(self.current))
        data,origins=json.loads(zlib.decompress(source.pop()))
        self.current=(data,origins)
        self.trim_history()
        self.data,self.origins=deepcopy(self.current)
        self.clipboard=None
        self.last_group=None
        return True

    @property
    def dirty(self):
        return self.data != self.saved

    def saved_now(self):
        self.saved = deepcopy(self.data)
        self.origins = list(range(len(self.data["pages"])))
        self.clear_history()

    def reset(self):
        self.clipboard = None
        self.data = deepcopy(self.saved)
        self.origins = list(range(len(self.data["pages"])))
        self.clear_history()

    def copy_key(self, page, index):
        key = deepcopy(self.data['pages'][page]['keys'][index])
        action = key['action']
        destination = self.data['pages'][action['page']] if action['type']=='go_to_page' else None
        self.clipboard = (key, destination)

    def paste_key(self, page, index):
        if self.clipboard is None:
            raise ValueError('Copy a key first.')
        stored, destination = self.clipboard
        key = deepcopy(stored)
        if destination is not None:
            target = next((i for i,p in enumerate(self.data['pages']) if p is destination),None)
            if target is None:
                raise ValueError('The copied key links to a deleted page. Copy another key.')
            key['action']['page'] = target
            if key.get('follow_page_name'):
                key['label'] = destination['name']
        self.data['pages'][page]['keys'][index] = key

    def clear_key(self, page, index):
        self.data['pages'][page]['keys'][index] = {'label':'EMPTY','label_position':'hidden','action':{'type':'none'}}

    def add_page(self):
        pages = self.data['pages']
        if len(pages) >= 16:
            raise ValueError('A layout can have up to 16 pages.')
        names = {page['name'] for page in pages}
        name = next(f'PAGE {chr(65+n)}' for n in range(26) if f'PAGE {chr(65+n)}' not in names)
        pages.append({'name': name, 'background': [30, 34, 39], 'keys': [
            {'label': 'EMPTY', 'action': {'type': 'none'}} for _ in range(8)]})
        self.origins.append(None)
        return len(pages) - 1

    def remap_targets(self, mapping):
        for page in self.data['pages']:
            for key in page['keys']:
                action = key['action']
                if action['type'] == 'go_to_page':
                    target = mapping.get(action['page'])
                    if target is None:
                        key['action'] = {'type':'none'}
                        key['follow_page_name'] = False
                    else:
                        action['page'] = target

    def move_page(self, index, target):
        pages = self.data['pages']
        if not 0 <= target < len(pages):
            raise ValueError('Page position is out of range.')
        order = list(range(len(pages)))
        order.insert(target, order.pop(index))
        self.data['pages'] = [pages[i] for i in order]
        self.origins = [self.origins[i] for i in order]
        self.remap_targets({old:new for new,old in enumerate(order)})
        return target

    def default_page(self):
        return next((i for i,page in enumerate(self.data['pages']) if page.get('default')),0)

    def set_default_page(self, index):
        if not 0<=index<len(self.data['pages']):raise ValueError('Choose an existing default page.')
        for i,page in enumerate(self.data['pages']):
            if i==index:page['default']=True
            else:page.pop('default',None)

    def duplicate_page(self, index):
        pages = self.data['pages']
        if len(pages) >= 16:
            raise ValueError('A layout can have up to 16 pages.')
        copy = deepcopy(pages[index])
        copy.pop("application",None)
        copy.pop("default",None)
        names = {page['name'] for page in pages}
        copy['name'] = next(f'COPY {n}' for n in range(1,17) if f'COPY {n}' not in names)
        target = index + 1
        self.remap_targets({i:i if i < target else i+1 for i in range(len(pages))})
        for key in copy['keys']:
            action = key['action']
            if action['type'] == 'go_to_page':
                old = action['page']
                action['page'] = target if old == index else old if old < target else old+1
                if key.get('follow_page_name'):
                    key['label'] = copy['name'] if old == index else pages[old]['name']
        pages.insert(target,copy)
        self.origins.insert(target,None)
        return target

    def references_to(self, index):
        return [(pi,ki) for pi,page in enumerate(self.data['pages']) if pi != index
                for ki,key in enumerate(page['keys']) if key['action'] == {'type':'go_to_page','page':index}]

    def delete_page(self, index, clear_links=False):
        pages = self.data['pages']
        if len(pages) == 1:
            raise ValueError('Keep at least one page.')
        if self.references_to(index) and not clear_links:
            raise ValueError('Other buttons link to this page.')
        count = len(pages)
        pages.pop(index)
        if not any(p.get("default") for p in pages):pages[0]["default"]=True
        self.origins.pop(index)
        self.remap_targets({i:i if i < index else i-1 for i in range(count) if i != index})
        return min(index,len(pages)-1)

    def rename_page(self, index, name):
        pages = self.data['pages']
        old_name = pages[index]['name']
        pages[index]['name'] = name
        for page in pages:
            for key in page['keys']:
                linked = key.get('follow_page_name') and key['action'] == {'type':'go_to_page', 'page':index}
                if linked or key['label'] == old_name:
                    key['label'] = name

    def validate(self):
        from plugin_bindings import validate as validate_plugin
        for page in self.data['pages']:
            for key in page['keys']:
                if key.get('plugin') is not None:
                    validate_plugin(key['plugin'])
                    if key['action'] != {'type':'none'}:raise ValueError('Choose either a plugin or a built-in action.')
        from themes import validate
        validate(self.data)
        defaults=[p.get('default',False) for p in self.data['pages']]
        if any(type(value) is not bool for value in defaults) or sum(defaults)>1:
            raise ValueError('Choose exactly one default page.')
        assigned=set()
        for page in self.data['pages']:
            app=page.get('application')
            if app is not None:
                if not isinstance(app,str) or not app.endswith('.desktop') or len(app)>255 or not all(c.isascii() and (c.isalnum() or c in '._-') for c in app) or app=='cc.senecal.Decksmith.Studio.desktop':
                    raise ValueError('Choose a valid application other than Decksmith.')
                if app in assigned:raise ValueError('An application can be assigned to only one page.')
                assigned.add(app)
        def label_valid(text, mixed=False):
            return 1 <= len(text) <= (24 if mixed else 8) and any(c != ' ' for c in text) and all(c == ' ' or 'A' <= c <= 'Z' or '0' <= c <= '9' or (mixed and 'a' <= c <= 'z') for c in text)
        for page in self.data['pages']:
            if not label_valid(page['name'], mixed=True):
                raise ValueError('Page names need 1–24 letters, numbers or spaces.')
            for index, key in enumerate(page['keys']):
                if not label_valid(key['label'], mixed=True):
                    raise ValueError(f"Key {index+1} on {page['name']} needs 1–24 letters, numbers or spaces.")
                action = key['action']
                if action['type'] == 'go_to_page' and not 0 <= action['page'] < len(self.data['pages']):
                    raise ValueError(f"Key {index+1} on {page['name']}: Choose an existing destination page.")
                if action['type']=='system':
                    from system_controls import COMMANDS
                    if action.get('command') not in COMMANDS:raise ValueError(f"Key {index+1} on {page['name']}: Choose a supported system action.")
                if action['type'] == 'open_application' and not action.get('desktop_id'):
                    raise ValueError(f"Key {index+1} on {page['name']}: Choose an installed application.")
                if action['type'] == 'open_website':
                    value = action.get('url','')
                    try:
                        url = urlsplit(value)
                        valid = url.scheme in ('http','https') and url.hostname and len(value)<=2048 and not any(c.isspace() or ord(c)<32 for c in value)
                    except ValueError:
                        valid = False
                    if not valid:
                        raise ValueError(f"Key {index+1} on {page['name']}: Enter a complete http:// or https:// website URL.")
                if key.get('media_player') is not None:
                    from media import valid_player
                    if action['type'] not in ('media_play_pause','media_next','media_previous') or not valid_player(key['media_player']):raise ValueError(f"Key {index+1} on {page['name']}: Choose a valid media player.")
                if action['type'] in ('audio_adjust','audio_mute','audio_select','push_to_talk'):
                    from audio_targets import validate
                    validate(action.get('target'))
                    if action['type']=='push_to_talk' and not action['target'].startswith('input:'):raise ValueError(f"Key {index+1} on {page['name']}: Choose a named microphone for push to talk.")
                    if action['type']=='audio_select' and not action['target'].startswith(('output:','input:')):raise ValueError(f"Key {index+1} on {page['name']}: Choose an output or microphone device.")
                if action['type'] in ('volume_adjust','audio_adjust') and (action['percent'] == 0 or not -20 <= action['percent'] <= 20):
                    raise ValueError(f"Key {index+1} on {page['name']}: Choose a volume step from −20 to 20, excluding zero.")
        dial_sets=[self.data.get('dials')]
        dial_sets.extend(page.get('dial_overrides') for page in self.data['pages'])
        for dials in dial_sets:
            if dials is not None and (not isinstance(dials,list) or len(dials)!=4):raise ValueError('Dial settings must have four positions.')
        if self.data.get('dials') is not None and any(d is None for d in self.data['dials']):raise ValueError('Shared dials need complete settings.')
        for index, dial in ((i,d) for dials in dial_sets if dials is not None for i,d in enumerate(dials) if d is not None):
            for field in ('plugin_rotation','plugin_press'):
                if dial.get(field) is not None:validate_plugin(dial[field])
            if (dial.get('plugin_rotation') is not None and dial['rotation']!='none') or (dial.get('plugin_press') is not None and dial['press']!={'type':'none'}):
                raise ValueError('Choose either a plugin or a built-in dial action.')
            if dial.get('target_icon_png') is not None:
                try:
                    image=bytes(dial['target_icon_png'])
                    if len(image)>8192 or image[:8]!=b'\x89PNG\r\n\x1a\n' or len(image)<24 or image[16:24]!=b'\0\0\0 \0\0\0 ':raise ValueError()
                except (TypeError,ValueError):raise ValueError('Choose a valid 32-pixel audio application icon.')
            if not label_valid(dial['label'], mixed=True):
                raise ValueError(f'Dial {index+1} needs 1–24 letters, numbers or spaces.')
            if dial.get('rotation') not in ('none','volume','brightness') or not isinstance(dial.get('step'),int) or not 1<=dial['step']<=10:raise ValueError('Invalid dial rotation or step.')
            from audio_targets import validate
            validate(dial.get('audio_target','system'))
            if dial['press']['type']=='push_to_talk':
                target=dial['press'].get('target','')
                if not target.startswith('input:'):
                    raise ValueError(f'Choose a named microphone for Dial {index+1} push to talk.')
                validate(target)
