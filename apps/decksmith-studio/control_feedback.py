"""Saved-layout feedback; never interprets a draft as live device state."""
import gi
gi.require_version('Gtk','4.0');gi.require_version('Adw','1')
from gi.repository import Gtk,Adw

def entries(status):
    if not status.get('running') or not status.get('connected'):return []
    result=[];seen=set()
    for notice in status.get('attention',[])[:25]:
        c=notice['check'];slot=c['slot'];place=f'Key {slot+1}' if slot<8 else f'Dial {slot-7}'
        title=f"{place} · {c['label']}"+(f" · {notice['target_name']}" if c['label'].casefold()!=notice['target_name'].casefold() else "")
        detail=('Last attempt: ' if notice['status']=='failed' else '')+notice['detail']
        subtitle=detail+'\n'+notice['hint']
        key=(title,subtitle)
        if key not in seen:result.append((title,subtitle));seen.add(key)
    return result
class FeedbackView(Adw.PreferencesGroup):
    def __init__(self):
        super().__init__(title='Live control feedback',description='Applies to the saved layout on the device, not unsaved edits.')
        self.rows=[];self.previous=None;self.set_visible(False)
    def update(self,status):
        data=entries(status)
        if self.previous==data:return
        self.previous=data
        for row in self.rows:self.remove(row)
        self.rows=[]
        for title,subtitle in data:
            row=Adw.ActionRow(title=title,subtitle=subtitle,use_markup=False)
            row.add_prefix(Gtk.Image.new_from_icon_name('dialog-warning-symbolic'));self.add(row);self.rows.append(row)
        self.set_visible(bool(data))
