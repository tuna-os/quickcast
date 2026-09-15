import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';

export default class QuickcastDesktop extends Extension {
    enable() {
        this._settings = this.getSettings();
        this._windows = new Map();
        this._timers = new Set();
        this._created = global.display.connect('window-created', (_display, window) => {
            let attempts = 0;
            const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                if (this._track(window) || ++attempts >= 20) {
                    this._timers.delete(id);
                    return GLib.SOURCE_REMOVE;
                }
                return GLib.SOURCE_CONTINUE;
            });
            this._timers.add(id);
        });
        for (const actor of global.get_window_actors())
            this._track(actor.meta_window);
        this._button = new PanelMenu.Button(0.0, 'Quickcast');
        this._button.add_child(new St.Icon({icon_name: 'camera-video-symbolic', style_class: 'system-status-icon'}));
        this._button.menu.addAction('Start / stop recording', () => this._activate('toggle-record', []));
        Main.panel.addToStatusArea(this.uuid, this._button);
    }

    _activate(action, parameters) {
        Gio.DBus.session.call('org.tunaos.Quickcast', '/org/tunaos/Quickcast',
            'org.gtk.Actions', 'Activate',
            new GLib.Variant('(sava{sv})', [action, parameters, {}]),
            null, Gio.DBusCallFlags.NONE, 2000, null, (connection, result) => {
                try { connection.call_finish(result); }
                catch (error) { console.debug(`Quickcast: ${error.message}`); }
            });
    }

    _track(window) {
        if (window.get_gtk_application_id() !== 'org.tunaos.Quickcast' ||
            window.get_title() !== 'Quickcast Webcam')
            return false;
        if (this._windows.has(window))
            return true;
        window.make_above();
        window.stick();
        const monitor = window.get_monitor();
        const area = global.workspace_manager.get_active_workspace().get_work_area_for_monitor(monitor);
        const width = Math.min(this._settings.get_int('width'), area.width, area.height);
        const height = width;
        window.move_resize_frame(false,
            area.x + Math.round(this._settings.get_double('x') * (area.width - width)),
            area.y + Math.round(this._settings.get_double('y') * (area.height - height)),
            width, height);
        const changed = () => this._update(window);
        const signals = [window.connect('position-changed', changed), window.connect('size-changed', changed)];
        signals.push(window.connect('unmanaged', () => {
            this._windows.delete(window);
        }));
        this._windows.set(window, signals);
        this._update(window);
        return true;
    }

    _update(window) {
        const frame = window.get_frame_rect();
        const area = Main.layoutManager.monitors[window.get_monitor()];
        if (!area || frame.width <= 0 || frame.height <= 0)
            return;
        const clamp = value => Math.max(0, Math.min(1, value));
        const x = clamp((frame.x - area.x) / Math.max(1, area.width - frame.width));
        const y = clamp((frame.y - area.y) / Math.max(1, area.height - frame.height));
        this._settings.set_double('x', x);
        this._settings.set_double('y', y);
        this._settings.set_int('width', frame.width);
        this._settings.set_int('height', frame.height);
        this._activate('bubble-position', [new GLib.Variant('(ddd)', [x, y, frame.width / area.width])]);
    }

    disable() {
        global.display.disconnect(this._created);
        for (const timer of this._timers)
            GLib.Source.remove(timer);
        this._timers.clear();
        for (const [window, signals] of this._windows) {
            for (const signal of signals)
                window.disconnect(signal);
            window.unmake_above();
            window.unstick();
        }
        this._windows.clear();
        this._button.destroy();
        this._settings = null;
    }
}
