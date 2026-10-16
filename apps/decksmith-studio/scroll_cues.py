"""Native scrolling with persistent gutters and event-driven overflow cues."""
from gi.repository import Gtk


class ScrollCues(Gtk.Box):
    """Wrap an existing scroller without changing its child or input handling."""
    def __init__(self, scroll):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, hexpand=scroll.get_hexpand(),
                         vexpand=scroll.get_vexpand())
        self.scroll = scroll
        scroll.set_overlay_scrolling(False)
        scroll.set_policy(scroll.get_policy()[0], Gtk.PolicyType.EXTERNAL)
        scroll.add_css_class('ds-scroll')
        self.above = self.cue('↑ More above', -1)
        body=Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL,hexpand=True,vexpand=True)
        scroll.set_hexpand(True);body.append(scroll)
        self.bar=Gtk.Scrollbar(orientation=Gtk.Orientation.VERTICAL,adjustment=scroll.get_vadjustment())
        self.bar.add_css_class('ds-scrollbar');body.append(self.bar);self.append(body)
        self.below = self.cue('↓ More below', 1)
        self.adjustment = scroll.get_vadjustment()
        self.handlers = []
        self.connect('map', self.observe)
        self.connect('unmap', self.disconnect_adjustment)

    def cue(self, label, direction):
        # A homogeneous stack reserves the native button's actual themed height,
        # even when no cue is needed (including enlarged text).
        slot = Gtk.Stack(hhomogeneous=True, vhomogeneous=True)
        button = Gtk.Button(label=label, halign=Gtk.Align.CENTER, hexpand=True)
        button.add_css_class('flat');button.add_css_class('caption')
        button.set_tooltip_text('Scroll to additional controls ' + ('above' if direction < 0 else 'below'))
        button.connect('clicked', lambda *_: self.move(direction))
        slot.add_named(button, 'cue');slot.add_named(Gtk.Box(), 'blank')
        button.cue_slot=slot;self.append(slot)
        return button

    def observe(self, *_):
        if not self.handlers:
            self.handlers = [self.adjustment.connect(signal, self.sync)
                             for signal in ('changed', 'value-changed')]
        self.sync()

    def disconnect_adjustment(self, *_):
        for handler in self.handlers:self.adjustment.disconnect(handler)
        self.handlers.clear()

    def sync(self, *_):
        a = self.adjustment
        end = max(a.get_lower(), a.get_upper() - a.get_page_size())
        overflow = end - a.get_lower() > 1
        self.bar.set_opacity(1 if overflow else 0)
        self.bar.set_sensitive(overflow)
        for button in (self.above, self.below):button.cue_slot.set_visible_child_name('cue' if overflow else 'blank')
        self.above.set_sensitive(a.get_value() > a.get_lower() + 1)
        self.below.set_sensitive(a.get_value() < end - 1)

    def move(self, direction):
        a = self.adjustment
        a.set_value(max(a.get_lower(), min(a.get_upper() - a.get_page_size(),
                    a.get_value() + direction * max(40, a.get_page_size() * .65))))
