"""Opt-in local lab bridge; no plugin execution or credential storage in Studio."""
import json
import os
import re
import stat
import urllib.request
from urllib.parse import urlsplit


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


class LabClient:
    def __init__(self, path):
        self.path = path
        self.opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
        self.reload()

    def reload(self):
        path = self.path
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd) as stream:
            info = os.fstat(stream.fileno())
            if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
                raise ValueError('Lab connection file must be private and owned by this user')
            data = json.loads(stream.read(4097))
        url = urlsplit(data.get('url', ''))
        token = data.get('token', '')
        if (data.get('version') != 1 or url.scheme != 'http' or url.hostname != '127.0.0.1'
                or not url.port or url.username or url.password or url.path or url.query or url.fragment
                or not isinstance(token, str) or not re.fullmatch(r'[A-Za-z0-9_-]{32,128}', token)
                or data.get('provider') != 'OpenHomeB' or data.get('target') != 'Main_LED’s'):
            raise ValueError('Unsupported lab connection')
        self.url = data['url']
        self.token = token

    def request(self, command=None):
        payload = None if command is None else json.dumps({'command': command}).encode()
        if command is not None and command not in ('on', 'off', 'up', 'down', 'reconnect','enable','disable'):
            raise ValueError('Unsupported lab command')
        self.reload()  # A supervised restart rotates the loopback port and token.
        request = urllib.request.Request(self.url + ('/state' if command is None else '/command'),
            data=payload, headers={'Content-Type': 'application/json', 'X-Lab-Token': self.token})
        with self.opener.open(request, timeout=3) as response:
            raw = response.read(65537)
        if len(raw) > 65536:
            raise ValueError('Lab response too large')
        data = json.loads(raw)
        if not isinstance(data, dict):
            raise ValueError('Invalid lab response')
        if command is not None:
            if type(data.get('ok')) is not bool:
                raise ValueError('Invalid command result')
            return data
        if 'connection' in data:
            if data['connection'] not in ('not_configured','connected','disabled','connecting','disconnected','device_unavailable','unconfirmed','authentication_required','settings_error'):raise ValueError('Invalid connection status')
            if type(data.get('enabled')) is not bool or type(data.get('device_count')) is not int or not 0<=data['device_count']<=256:raise ValueError('Invalid connection metadata')
            if 'has_saved_password' in data and type(data['has_saved_password']) is not bool:raise ValueError('Invalid password metadata')
            if data['connection']=='not_configured' and (data.get('server')!='' or data['enabled']):raise ValueError('Invalid removed connection')
            server=urlsplit(data.get('server',''))
            if data['connection']!='not_configured' and (server.scheme not in ('http','https') or not server.hostname or server.username or server.password or server.query or server.fragment):raise ValueError('Invalid server address')
        else:
            raise ValueError('Connection manager requires updated runtime')
        if 'setup' in data:
            result=data['setup']
            if not isinstance(result,dict) or set(result)-{'state','message','candidate','server','username','functions'}:raise ValueError('Invalid setup status')
            if result.get('state') not in ('removed','idle','expired','tested','required','rejected','two_factor','keyring','invalid','offline','saved','failed'):raise ValueError('Invalid setup state')
            if not isinstance(result.get('message',''),str) or len(result.get('message',''))>512:raise ValueError('Invalid setup message')
            if 'candidate' in result and (not isinstance(result['candidate'],str) or not re.fullmatch('[0-9a-f]{32}',result['candidate'])):raise ValueError('Invalid connection test')
        if data.get('target') != 'Main_LED’s':
            raise ValueError('Unexpected accessory')
        for key in ('ready', 'busy', 'fresh', 'feedback_fresh'):
            if type(data.get(key)) is not bool:
                raise ValueError('Invalid status')
        actual = data.get('actual')
        if actual is not None and (not isinstance(actual, dict) or type(actual.get('on')) is not bool
                or type(actual.get('brightness')) not in (int, float) or not 0 <= actual['brightness'] <= 100):
            raise ValueError('Invalid accessory state')
        for key in ('message', 'feedback'):
            if data.get(key) is not None and (not isinstance(data[key], str) or len(data[key]) > 512):
                raise ValueError('Invalid status text')
        return data

    def setup(self,data):
        if not isinstance(data,dict) or data.get('operation') not in ('test','apply','remove'):raise ValueError('Invalid setup operation')
        self.reload()
        raw=json.dumps(data).encode()
        if len(raw)>8192:raise ValueError('Setup fields too long')
        request=urllib.request.Request(self.url+'/setup',data=raw,headers={'Content-Type':'application/json','X-Lab-Token':self.token})
        with self.opener.open(request,timeout=3) as response:result=json.loads(response.read(1024))
        if type(result.get('ok')) is not bool:raise ValueError('Invalid setup reply')
        return result

    def catalog(self):
        self.reload()
        with self.opener.open(urllib.request.Request(self.url+'/catalog',headers={'X-Lab-Token':self.token}),timeout=3) as response:
            raw=response.read(524289)
        if len(raw)>524288:raise ValueError('Catalogue too large')
        data=json.loads(raw)
        if not isinstance(data,dict) or type(data.get('available')) is not bool or not isinstance(data.get('items'),list) or len(data['items'])>256:raise ValueError('Invalid catalogue')
        for item in data['items']:
            if not isinstance(item,dict) or not isinstance(item.get('id'),str) or not re.fullmatch('[0-9a-f]{64}',item['id']):raise ValueError('Invalid accessory identity')
            for key in ('name','kind','icon'):
                if not isinstance(item.get(key),str) or len(item[key])>128:raise ValueError('Invalid accessory description')
            if not isinstance(item.get('operations'),list) or any(x not in ('toggle','on','off','level','status') for x in item['operations']):raise ValueError('Invalid capability')
        return data
