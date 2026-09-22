"""Installed desktop applications and portable 120px PNG key icons."""
import gi
gi.require_version('Gtk','4.0')
from gi.repository import Gio, Gtk, Gdk, GdkPixbuf

def installed():
    return sorted([app for app in Gio.AppInfo.get_all() if app.should_show() and app.get_id()],key=lambda app:app.get_display_name().casefold())

def icon_png(app):
    try:
        icon = Gtk.IconTheme.get_for_display(Gdk.Display.get_default()).lookup_by_gicon(app.get_icon(),120,1,Gtk.TextDirection.NONE,Gtk.IconLookupFlags.FORCE_REGULAR)
        source = GdkPixbuf.Pixbuf.new_from_file_at_scale(icon.get_file().get_path(),104,104,True)
        canvas = GdkPixbuf.Pixbuf.new(GdkPixbuf.Colorspace.RGB,False,8,120,120)
        canvas.fill(0x1e2227ff)
        x,y=(120-source.get_width())//2,(120-source.get_height())//2
        source.composite(canvas,x,y,source.get_width(),source.get_height(),x,y,1,1,GdkPixbuf.InterpType.BILINEAR,255)
        ok,data=canvas.save_to_bufferv('png',[],[])
        return list(data) if ok else None
    except Exception:
        return None


def website_png(data):
    from artwork import render
    return render(data)

def audio_icon_png(target):
    """Resolve installed audio application artwork once when a target is chosen."""
    from audio_apps import cached,discover
    match=next((a for a in cached() or discover() if a['id']==target),None)
    if not match:return None
    app=Gio.DesktopAppInfo.new(match['desktop_id'])
    if app is None:return None
    try:
        icon=Gtk.IconTheme.get_for_display(Gdk.Display.get_default()).lookup_by_gicon(app.get_icon(),32,1,Gtk.TextDirection.NONE,Gtk.IconLookupFlags.FORCE_REGULAR)
        source=GdkPixbuf.Pixbuf.new_from_file_at_scale(icon.get_file().get_path(),32,32,True)
        canvas=GdkPixbuf.Pixbuf.new(GdkPixbuf.Colorspace.RGB,True,8,32,32);canvas.fill(0)
        source.copy_area(0,0,source.get_width(),source.get_height(),canvas,(32-source.get_width())//2,(32-source.get_height())//2)
        ok,data=canvas.save_to_bufferv('png',[],[])
        return list(data) if ok else None
    except Exception:return None
