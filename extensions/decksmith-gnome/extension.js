import {ForegroundReporter} from './foreground.js';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import Shell from 'gi://Shell';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const SERVICE = 'org.freedesktop.systemd1';
const MANAGER = '/org/freedesktop/systemd1';
const CONTROL = 'cc.senecal.Decksmith';

export default class DecksmithExtension extends Extension {
    enable() {
        this._cancel = new Gio.Cancellable();
        this._busy = false;
        this._refreshing = false;
        this._running = false;
        this._timer = 0;
        this._button = new PanelMenu.Button(0, 'Decksmith');
        this._button.add_child(new St.Icon({gicon: Gio.FileIcon.new(this.dir.get_child('decksmith-symbolic.svg')), style_class: 'system-status-icon'}));
        this._status = new PopupMenu.PopupMenuItem('Checking background controls…', {reactive: false});
        this._device = new PopupMenu.PopupMenuItem('Device status unavailable', {reactive: false});
        this._button.menu.addMenuItem(this._status);
        this._button.menu.addMenuItem(this._device);
        this._button.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        this._button.menu.addAction('Open Decksmith', () => {
            const app = Shell.AppSystem.get_default().lookup_app('cc.senecal.Decksmith.Studio.desktop');
            if (app) app.activate();
            else Main.notify('Decksmith', 'Install Decksmith Studio to open its controls.');
        });
        this._toggle = this._button.menu.addAction('Start background controls', () => this._change());
        this._button.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        this._quit = this._button.menu.addAction('Quit Decksmith', () => this._quitDecksmith());
        this._button.menu.connect('open-state-changed', (_menu, open) => {
            this._clearTimer();
            if (open) {
                this._refresh();
                this._timer = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 3, () => {
                    this._refresh();
                    return GLib.SOURCE_CONTINUE;
                });
            }
        });
        Main.panel.addToStatusArea(this.uuid, this._button);
        this._tracker = Shell.WindowTracker.get_default();
        this._foreground = new ForegroundReporter(id => this._call(CONTROL, '/cc/senecal/Decksmith', CONTROL + '.Control1', 'ReportForeground', new GLib.Variant('(s)', [id])));
        this._sessionSignal = Main.sessionMode.connect('updated', () => this._reportFocus());
        this._focusSignal = this._tracker.connect('notify::focus-app', () => this._reportFocus());
        this._focusOwner = Gio.bus_own_name_on_connection(Gio.DBus.session, 'cc.senecal.Decksmith.Foreground', Gio.BusNameOwnerFlags.NONE,
            () => {this._focusOwned=true;this._reportFocus(true);}, () => {this._focusOwned=false;});
        this._watch = Gio.bus_watch_name(Gio.BusType.SESSION, CONTROL, Gio.BusNameWatcherFlags.NONE,
            () => {this._refresh();this._reportFocus(true);}, () => this._refresh());
        this._refresh();
    }

    _reportFocus(force=false) {
        if (!this._focusOwned || Main.sessionMode.isLocked) return;
        const app=this._tracker.focus_app;
        const id=app?.get_id() ?? '';
        // Only desktop-file identity; never window titles or content.
        this._foreground.report(id.endsWith('.desktop') ? id : '', force);
    }

    _call(name, path, iface, method, args = null) {
        const cancel = this._cancel;
        return new Promise((resolve, reject) => {
            Gio.DBus.session.call(name, path, iface, method, args, null,
                Gio.DBusCallFlags.NO_AUTO_START, 2500, cancel, (connection, result) => {
                    try { resolve(connection.call_finish(result).deep_unpack()); }
                    catch (error) { reject(error); }
                });
        });
    }

    async _refresh() {
        if (!this._button || this._refreshing || this._busy) return;
        this._refreshing = true;
        const cancel = this._cancel;
        try {
            const [path] = await this._call(SERVICE, MANAGER, SERVICE + '.Manager', 'LoadUnit',
                new GLib.Variant('(s)', ['decksmith.service']));
            const [value] = await this._call(SERVICE, path, 'org.freedesktop.DBus.Properties', 'Get',
                new GLib.Variant('(ss)', [SERVICE + '.Unit', 'ActiveState']));
            if (cancel.is_cancelled()) return;
            const state = value.deep_unpack();
            this._running = state === 'active';
            this._status.label.text = this._running ? 'Background controls running' : `Background controls ${state}`;
            this._toggle.label.text = this._running ? 'Stop background controls' : 'Start background controls';
            this._toggle.setSensitive(!['activating', 'deactivating'].includes(state));
            this._device.label.text = 'Start controls to connect your device';
            if (this._running) {
                try {
                    const [json] = await this._call(CONTROL, '/cc/senecal/Decksmith', CONTROL + '.Control1', 'GetStatus');
                    if (cancel.is_cancelled()) return;
                    const status = JSON.parse(json);
                    if (status.api_version !== 1) throw new Error('Unsupported control interface');
                    this._device.label.text = status.auto_lock?.locked
                        ? (status.auto_lock.available ? 'Session locked · controls disabled' : 'Lock state unavailable · controls held locked')
                        : status.connected
                        ? (status.display_ready ? 'Stream Deck connected' : 'Updating device displays…')
                        : 'Waiting for device';
                } catch (error) {
                    if (!cancel.is_cancelled()) this._device.label.text = 'Control interface unavailable';
                }
            }
        } catch (error) {
            if (!cancel.is_cancelled()) {
                this._status.label.text = 'Background service unavailable';
                this._device.label.text = 'Open Decksmith to check installation';
                this._toggle.setSensitive(false);
            }
        } finally {
            if (this._cancel === cancel) this._refreshing = false;
        }
    }

    async _change() {
        if (this._busy || !this._button) return;
        this._busy = true;
        this._toggle.setSensitive(false);
        const cancel = this._cancel;
        try {
            await this._call(SERVICE, MANAGER, SERVICE + '.Manager', this._running ? 'StopUnit' : 'StartUnit',
                new GLib.Variant('(ss)', ['decksmith.service', 'replace']));
        } catch (error) {
            if (!cancel.is_cancelled()) Main.notify('Decksmith', 'Could not change background controls. Open Decksmith to check the service.');
        } finally {
            if (this._cancel === cancel && !cancel.is_cancelled()) {
                this._busy = false;
                this._refresh();
            }
        }
    }

    async _quitDecksmith() {
        if (this._busy || !this._button) return;
        this._busy = true;
        this._quit.setSensitive(false);
        const cancel = this._cancel;
        try {
            const [open] = await this._call('org.freedesktop.DBus', '/org/freedesktop/DBus',
                'org.freedesktop.DBus', 'NameHasOwner', new GLib.Variant('(s)', ['cc.senecal.Decksmith.Studio']));
            if (open) {
                // The editor owns save/discard/cancel and stops controls only after consent.
                await this._call('cc.senecal.Decksmith.Studio', '/cc/senecal/Decksmith/Studio',
                    'org.gtk.Actions', 'Activate', new GLib.Variant('(sava{sv})', ['quit-decksmith', [], {}]));
            } else {
                await this._call(SERVICE, MANAGER, SERVICE + '.Manager', 'StopUnit',
                    new GLib.Variant('(ss)', ['decksmith.service', 'replace']));
            }
        } catch (error) {
            if (!cancel.is_cancelled()) Main.notify('Decksmith', 'Could not quit Decksmith. Open the app to check background controls.');
        } finally {
            if (this._cancel === cancel && !cancel.is_cancelled()) {
                this._busy = false;
                this._quit.setSensitive(true);
                this._refresh();
            }
        }
    }

    _clearTimer() {
        if (this._timer) GLib.Source.remove(this._timer);
        this._timer = 0;
    }

    disable() {
        this._foreground?.stop();
        if (this._sessionSignal) Main.sessionMode.disconnect(this._sessionSignal);
        this._sessionSignal=0;
        if (this._focusSignal) this._tracker.disconnect(this._focusSignal);
        this._focusSignal=0;
        if (this._focusOwner) Gio.bus_unown_name(this._focusOwner);
        this._focusOwner=0;this._focusOwned=false;
        this._cancel?.cancel();
        this._clearTimer();
        if (this._watch) Gio.bus_unwatch_name(this._watch);
        this._watch = 0;
        this._button?.destroy();
        this._button = null;
        this._status = null;
        this._device = null;
        this._toggle = null;
    }
}
