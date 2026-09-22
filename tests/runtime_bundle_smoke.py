#!/usr/bin/env python3
"""Run under dbus-run-session with isolated XDG paths; only VirtualDeck is opened."""
import json,os,subprocess,sys,time
from pathlib import Path
sys.dont_write_bytecode=True
root=Path(sys.argv[1]).resolve()
assert os.environ.get('DECKSMITH_ISOLATED_TEST')=='1'
assert os.environ.get('DBUS_SESSION_BUS_ADDRESS','').startswith('unix:path=/tmp/dbus-'), 'Use dbus-run-session'
sys.path.insert(0,str(root/'apps/decksmith-studio'))
from panel import call,GLib
log=Path(os.environ['XDG_STATE_HOME'])/'virtual-service.log';log.parent.mkdir(parents=True,exist_ok=True)
with log.open('w') as output:
    daemon=subprocess.Popen([str(root/'bin/decksmithd'),'--virtual-service',str(root/'config/navigation.json'),'--seconds','25'],stdout=output,stderr=output)
    try:
        deadline=time.monotonic()+12
        while True:
            try:
                status=json.loads(call('GetStatus')[0])
                if status['connected'] and status.get('display_ready'):break
            except Exception:pass
            if daemon.poll() is not None or time.monotonic()>deadline:raise RuntimeError(log.read_text())
            time.sleep(.1)
        layout=call('GetLayout')[0]
        assert len(call('PreviewKeys',GLib.Variant('(sy)',(layout,0))))==8*120*120*3
        assert len(call('PreviewTouch',GLib.Variant('(sy)',(layout,0)))[0])==800*100*3
        call('ShowPage',GLib.Variant('(y)',(1,)))
        assert json.loads(call('GetStatus')[0])['active_page']==1
        call('SaveLayout',GLib.Variant('(s)',(layout,)))
        saved=Path(os.environ['XDG_CONFIG_HOME'])/'decksmith/layout.json'
        assert saved.is_file()
        subprocess.run([str(root/'bin/decksmithd'),'--validate-layout',str(saved)],check=True)
        from icon_library import Library
        assert len(Library().items('terminal'))>0
        if '--native' in sys.argv:
            call('ShowPage',GLib.Variant('(y)',(1,)))
            import panel
            from editor import Editor
            panel.read_status=lambda:dict(json.loads(call('GetStatus')[0]),running=True)
            app=panel.Panel();app.set_application_id('cc.senecal.Decksmith.BundleTest')
            errors=[];began=time.monotonic();selected=[]
            def inspect():
                try:
                    if time.monotonic()-began>12:raise RuntimeError('Editor preview timeout')
                    ed=app.editor
                    if ed.draft is None or ed.pending or ed.key_preview.rendered!=ed.key_preview.revision or ed.touch_preview.rendered_revision!=ed.touch_preview.revision:return True
                    assert app.edit_button.get_sensitive()
                    assert ed.page==1
                    if not selected:
                        ed.select_dial(0);ed.select_key(0);selected.append(True);return True
                    paint=panel.Gtk.WidgetPaintable.new(ed);snapshot=panel.Gtk.Snapshot();paint.snapshot(snapshot,ed.get_width(),ed.get_height())
                    node=snapshot.to_node()
                    if node is None:return True
                    ed.get_renderer().render_texture(node,None).save_to_png(str(log.with_suffix('.png')))
                    print('PASS relocated native panel/editor, keys, strip and icons',flush=True)
                    app.quit();return False
                except BaseException as error:errors.append(error);app.quit();return False
            def show(_):app.editor=Editor(app,call);GLib.timeout_add(300,inspect)
            app.connect('activate',show);app.run(None)
            if errors:raise errors[0]
        print('PASS private-bus VirtualDeck startup, shared previews, page navigation, save and layout validation',flush=True)
    finally:
        daemon.terminate()
        daemon.wait(timeout=10)
    assert daemon.returncode==0,log.read_text()
