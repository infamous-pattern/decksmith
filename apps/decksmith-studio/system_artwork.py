"""Use the bundled offline icon library for system actions."""
from functools import lru_cache
from icon_library import Library
ICONS={'lock':'lock','suspend':'moon','reboot':'refresh','shutdown':'power','dnd':'bell-off','night_light':'moon','bluetooth':'bluetooth','power_cycle':'battery-charging','power_saver':'battery','power_balanced':'battery-charging','power_performance':'battery-charging'}
@lru_cache(maxsize=12)
def image(command):
    library=Library();item=library.preferred(ICONS[command])
    return library.image(item)
def populate(key):
    command=key['action']['command'];item=Library().preferred(ICONS[command])
    key.update(artwork='application_icon',icon_png=list(image(command)),icon_source=item['id'],icon_tint=True,follow_page_name=False)
