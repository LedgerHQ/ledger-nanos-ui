use crate::bitmaps::Glyph;
use crate::fonts::OPEN_SANS;
use crate::layout::*;

const fn middle_y(glyph: &Glyph) -> i16 {
    ((crate::SCREEN_HEIGHT as u32 - glyph.height) / 2) as i16
}

pub struct Label<'a> {
    pub text: &'a str,
    pub bold: bool,
    pub loc: Location,
    layout: Layout,
}

impl<'a> From<&'a str> for Label<'a> {
    fn from(s: &'a str) -> Label<'a> {
        Label {
            text: s,
            bold: false,
            loc: Location::Middle,
            layout: Layout::Centered,
        }
    }
}

impl<'a> Label<'a> {
    pub const fn from_const(s: &'a str) -> Label<'a> {
        Label {
            text: s,
            bold: false,
            loc: Location::Middle,
            layout: Layout::Centered,
        }
    }

    pub const fn location(self, loc: Location) -> Label<'a> {
        Label { loc, ..self }
    }

    pub const fn layout(self, layout: Layout) -> Label<'a> {
        Label { layout, ..self }
    }

    pub const fn bold(&self) -> Label<'a> {
        Label {
            bold: true,
            ..*self
        }
    }
}

impl Draw for Label<'_> {
    fn display(&self) {
        self.text.place(self.loc, self.layout, self.bold);
    }
    fn erase(&self) {
        let total_width = self.text.compute_width(self.bold);
        let c_height = OPEN_SANS[self.bold as usize].height as usize;
        let x = self.layout.get_x(total_width);
        let y = self.loc.get_y(c_height);
        pic_draw(
            x as i32,
            y as i32,
            total_width as u32,
            c_height as u32,
            false,
            &crate::bitmaps::BLANK,
        )
    }
}

pub struct Icon<'a> {
    icon: &'a Glyph<'a>,
    pos: (i16, i16),
}

impl<'a> From<&'a Glyph<'a>> for Icon<'a> {
    fn from(glyph: &'a Glyph) -> Icon<'a> {
        Icon {
            icon: glyph,
            pos: (0, middle_y(glyph)),
        }
    }
}

impl<'a> Icon<'a> {
    const fn from(glyph: &'a Glyph<'a>) -> Icon<'a> {
        Icon {
            icon: glyph,
            pos: (0, middle_y(glyph)),
        }
    }

    /// Set specific x-coordinate
    pub const fn set_x(self, x: i16) -> Icon<'a> {
        Icon {
            pos: (x, self.pos.1),
            ..self
        }
    }

    /// Shift horizontally
    pub const fn shift_h(self, n: i16) -> Icon<'a> {
        Icon {
            pos: (self.pos.0 + n, self.pos.1),
            ..self
        }
    }

    /// Shift vertically
    pub const fn shift_v(self, n: i16) -> Icon<'a> {
        Icon {
            pos: (self.pos.0, self.pos.1 + n),
            ..self
        }
    }
}

extern "C" {
    fn bagl_hal_draw_bitmap_within_rect(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color_count: u32,
        colors: *const u32,
        bit_per_pixel: u32,
        bitmap: *const u8,
        bitmap_length_bits: u32,
    );
}
use core::ffi::c_void;

#[inline(never)]
fn pic_draw(x: i32, y: i32, width: u32, height: u32, inverted: bool, bitmap: &[u8]) {
    let inverted = [inverted as u32, !inverted as u32];
    unsafe {
        let pic_bmp = nanos_sdk::bindings::pic(bitmap.as_ptr() as *mut c_void);
        bagl_hal_draw_bitmap_within_rect(
            x,
            y,
            width,
            height,
            2,
            inverted.as_ptr(),
            1,
            pic_bmp as *const u8,
            (bitmap.len() * 8) as u32,
        )
    }
}

impl<'a> Draw for Icon<'a> {
    fn display(&self) {
        let icon = nanos_sdk::pic_rs(self.icon);
        pic_draw(
            self.pos.0 as i32,
            self.pos.1 as i32,
            icon.width,
            icon.height,
            icon.inverted,
            icon.bitmap,
        );
    }

    fn erase(&self) {
        let icon = nanos_sdk::pic_rs(self.icon);
        pic_draw(
            self.pos.0 as i32,
            self.pos.1 as i32,
            icon.width,
            icon.height,
            icon.inverted,
            &crate::bitmaps::BLANK,
        );
    }
}

use crate::bitmaps;

pub const OUTER_PADDING: usize = 2;
pub const SCREENW: i16 = (crate::SCREEN_WIDTH - OUTER_PADDING) as i16;

pub const DOWN_ARROW: Icon =
    Icon::from(&bitmaps::DOWN_ARROW).set_x(SCREENW - bitmaps::DOWN_ARROW.width as i16);
pub const LEFT_ARROW: Icon = Icon::from(&bitmaps::LEFT_ARROW).set_x(OUTER_PADDING as i16);
pub const RIGHT_ARROW: Icon =
    Icon::from(&bitmaps::RIGHT_ARROW).set_x(SCREENW - bitmaps::RIGHT_ARROW.width as i16);
pub const UP_ARROW: Icon = Icon::from(&bitmaps::UP_ARROW).set_x(OUTER_PADDING as i16);
pub const DOWN_S_ARROW: Icon = DOWN_ARROW.shift_v(4);
pub const LEFT_S_ARROW: Icon = LEFT_ARROW.shift_h(4);
pub const RIGHT_S_ARROW: Icon = RIGHT_ARROW.shift_h(-4);
pub const UP_S_ARROW: Icon = UP_ARROW.shift_v(-4);

pub const CHECKMARK_ICON: Icon = Icon::from(&bitmaps::CHECKMARK);
pub const CROSS_ICON: Icon = Icon::from(&bitmaps::CROSS);
