"""Theme-aware Rune media artwork for newly assigned controls."""
from functools import lru_cache
from icon_library import Library

LABELS={'media_play_pause':'Play Pause','media_previous':'Previous Track','media_next':'Next Track'}
ICONS={'media_play_pause':'player-play-pause','media_previous':'player-skip-back','media_next':'player-skip-forward'}

@lru_cache(maxsize=16)
def icon(action, background='#1e2227'):
    if action not in ICONS:raise ValueError('Not a media action')
    library=Library();return library.image(library.preferred(ICONS[action]))

def populate(key, background='#1e2227', preserve_label=False):
    action=key['action']['type']
    item=Library().preferred(ICONS[action])
    key.update(label=key.get('label',LABELS[action]) if preserve_label else LABELS[action],artwork='application_icon',icon_png=list(icon(action,background)),
               icon_source=item['id'],icon_tint=True,label_position='bottom',label_background='transparent',follow_page_name=False)
