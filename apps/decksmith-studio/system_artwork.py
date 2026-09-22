"""Use the bundled offline icon library for system actions."""
from functools import lru_cache
from icon_library import Library
ICONS={'lock':'lock','suspend':'moon','reboot':'refresh','shutdown':'power','dnd':'bell-off','night_light':'sun','bluetooth':'bluetooth','power_cycle':'battery-charging','power_saver':'battery','power_balanced':'battery-charging','power_performance':'battery-charging'}
@lru_cache(maxsize=12)
def image(command):
    library=Library();item=next(i for i in library.items() if i['id']=='tabler:'+ICONS[command])
    return library.image(item)
def populate(key):
    key.update(artwork='application_icon',icon_png=list(image(key['action']['command'])),icon_source='tabler:'+ICONS[key['action']['command']],icon_tint=True,follow_page_name=False)
