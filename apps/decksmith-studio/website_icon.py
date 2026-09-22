"""Prefer high-resolution website icons over small browser favicons."""
from html.parser import HTMLParser
from urllib.parse import urljoin,urlsplit
from urllib.request import Request,urlopen
from time import monotonic
from artwork import dimensions

class Icons(HTMLParser):
    def __init__(self):
        super().__init__(); self.links=[]
    def handle_starttag(self,tag,attrs):
        attrs=dict(attrs)
        if tag.lower()=='link' and 'icon' in attrs.get('rel','').lower() and attrs.get('href'):
            sizes=[]
            for size in attrs.get('sizes','').split():
                try: sizes.append(min(map(int,size.lower().split('x'))))
                except ValueError: pass
            score=max(sizes,default=180 if 'apple-touch-icon' in attrs.get('rel','') else 0)
            self.links.append((score,attrs['href']))

def read(url,limit):
    if urlsplit(url).scheme not in ('http','https'): raise ValueError('Not a website URL')
    with urlopen(Request(url,headers={'User-Agent':'Decksmith/0.1'}),timeout=2) as response:
        data=response.read(limit+1)
        if len(data)>limit: raise ValueError('Resource too large')
        return data,response.geturl()

def fetch(url):
    deadline=monotonic()+10
    candidates=[]
    try:
        data,base=read(url,1048576)
        parser=Icons();parser.feed(data.decode('utf-8',errors='replace'))
        candidates=[urljoin(base,link) for _,link in sorted(parser.links,key=lambda entry:entry[0],reverse=True)[:6]]
    except Exception: pass
    candidates.append(urljoin(url,'/favicon.ico'))
    best=None; best_size=0
    for candidate in dict.fromkeys(candidates):
        if monotonic()>deadline: break
        try:
            data,_=read(candidate,262144)
            size=min(dimensions(data))
            if size>best_size: best,best_size=data,size
            if size>=120: break
        except Exception: continue
    return best
