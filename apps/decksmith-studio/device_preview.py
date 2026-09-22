"""Hardware-shaped editor decorations; controls remain native GTK buttons."""
import math
import cairo
from gi.repository import Gtk

CSS = '''
.device-preview {background: linear-gradient(150deg, #363839, #202223 70%, #292b2c); border: 1px solid #494c4e; border-radius: 28px; padding: 18px 22px 18px; box-shadow: 0 8px 14px alpha(black,0.24), inset 0 1px #626566;}
.device-brand {color: #c9cbcc; font-size: 12px; font-weight: bold; letter-spacing: 2px; margin-bottom: 2px;}
.device-preview .deck-key {border-radius: 16px; border: 2px solid #101112; box-shadow: 0 3px 3px alpha(black,0.75), inset 0 2px 2px alpha(white,0.3);}
.device-preview .deck-key.selected-key {border-color: #eab85d; box-shadow: 0 0 0 2px alpha(#eab85d,0.3), inset 0 2px 2px alpha(white,0.3);}
.device-preview .deck-key:hover {border-color: #989a9c;}
.device-preview .deck-key:focus-visible {outline: 2px solid #fafafa; outline-offset: 3px;}
.device-strip {background: #080b0e; border: 2px solid #111315; border-radius: 9px; padding: 5px; box-shadow: inset 0 1px 2px #62666a, 0 1px #515456;}
.device-strip button {background: transparent; background-image: none; color: #e8eef1; border-radius: 0; border: 2px solid transparent; border-right-color: transparent; padding: 0; min-width: 0; min-height: 0; box-shadow: none; font-size: 12px;}
.device-strip button:first-child {border-radius: 5px 0 0 5px;}
.device-strip button:last-child {border-radius: 0 5px 5px 0; border-right-color: transparent;}
.device-strip button.suggested-action {background: transparent; border-color: #eab85d; color: white;}
.device-strip button:hover {background: transparent; border-color: #a1a5a8;}
.device-strip button:focus-visible {outline: 2px solid white; outline-offset: -4px;}
.device-dials {background: linear-gradient(#161819,#292b2c); border-radius: 18px; padding: 10px 0; box-shadow: inset 0 2px 4px alpha(black,0.7), 0 1px alpha(white,0.1);}
.device-dials button {background: transparent; background-image: none; border: 2px solid transparent; border-radius: 50%; padding: 3px; min-width: 0; min-height: 0; box-shadow: none;}
.device-dials button.suggested-action {border-color: #eab85d; box-shadow: 0 0 0 2px alpha(#eab85d,0.2);}
.device-dials button:hover {border-color: #a1a5a8;}
.device-dials button:focus-visible {outline: 2px solid white; outline-offset: 2px;}
'''

class DialFace(Gtk.DrawingArea):
    def __init__(self):
        super().__init__(content_width=56,content_height=56)
        self.set_draw_func(self.draw)

    def draw(self,_area,cr,width,height):
        cr.translate(width/2,height/2)
        scale=min(width,height)/72
        cr.scale(scale,scale)
        cr.arc(0,2,34,0,math.tau);cr.set_source_rgba(0,0,0,.5);cr.fill()
        cr.arc(0,0,33,0,math.tau);cr.set_source_rgb(.09,.10,.11);cr.fill()
        for i in range(64):
            angle=i*math.tau/64
            cr.move_to(math.cos(angle)*30,math.sin(angle)*30)
            cr.line_to(math.cos(angle)*33,math.sin(angle)*33)
            cr.set_source_rgb(.28,.30,.32);cr.set_line_width(.7);cr.stroke()
        gradient=cairo.LinearGradient(-24,-28,22,30)
        gradient.add_color_stop_rgb(0,.55,.57,.58)
        gradient.add_color_stop_rgb(.38,.25,.27,.28)
        gradient.add_color_stop_rgb(.66,.13,.15,.16)
        gradient.add_color_stop_rgb(1,.34,.36,.37)
        cr.arc(0,0,29,0,math.tau);cr.set_source(gradient);cr.fill_preserve()
        cr.set_source_rgb(.43,.45,.46);cr.set_line_width(1);cr.stroke()
        cr.move_to(0,-23);cr.line_to(0,-17)
        cr.set_source_rgb(.8,.82,.83);cr.set_line_width(2);cr.stroke()
