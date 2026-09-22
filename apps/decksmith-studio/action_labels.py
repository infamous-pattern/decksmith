"""Defaults for action assignment; custom labels remain owned by the user."""
from media_artwork import LABELS as MEDIA_LABELS

LABELS = {
    'none': 'EMPTY', 'volume_up': 'Volume Up', 'volume_down': 'Volume Down',
    'mute_toggle': 'Mute Output', 'volume_adjust': 'Adjust Volume',
    'next_page': 'Next Page', 'previous_page': 'Previous Page',
    'open_application': 'Open App', 'open_website': 'Open Website',
    'audio_adjust': 'Adjust Volume', 'audio_mute': 'Toggle Mute',
    'audio_select': 'Select Device', 'push_to_talk': 'Push to Talk',
    **MEDIA_LABELS,
}

def default_label(action, pages):
    if action['type']=='system':
        from system_controls import LABELS as SYSTEM_LABELS
        return SYSTEM_LABELS.get(action.get('command'),'System')
    if action['type']=='go_to_page':
        index=action.get('page',0)
        return pages[index]['name'] if 0<=index<len(pages) else 'Go to Page'
    return LABELS[action['type']]

def uses_default(label, action, pages):
    # Older layouts have no label provenance. Recognize their placeholder and
    # the prior action's own default; do not replace other user-entered text.
    return not label.strip() or label.casefold()=='empty' or label.casefold()==default_label(action,pages).casefold()


def application_label(name):
    """Fit discovered names to the device's current label character/length limits."""
    import unicodedata
    ascii_name=unicodedata.normalize('NFKD',name).encode('ascii','ignore').decode()
    return ' '.join(''.join(c if c.isalnum() else ' ' for c in ascii_name).split())[:24].rstrip() or 'Open App'
