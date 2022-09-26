use crate::screen_util::draw;

pub struct Glyph<'a> {
    pub bitmap: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub inverted: bool,
}

impl<'a> Glyph<'a> {
    pub const fn new(bitmap: &'a [u8], width: u32, height: u32) -> Glyph<'a> {
        Glyph {
            bitmap,
            width,
            height,
            inverted: false,
        }
    }
    pub const fn from_include(packed: (&'a [u8], u32, u32)) -> Glyph<'a> {
        Glyph {
            bitmap: packed.0,
            width: packed.1,
            height: packed.2,
            inverted: false,
        }
    }
    pub const fn invert(self) -> Glyph<'a> {
        Glyph {
            inverted: true,
            ..self
        }
    }
    pub fn draw(&self, x: i32, y: i32) {
        draw(x, y, self.width, self.height, self.inverted, &self.bitmap);
    }
}

pub fn manual_screen_clear() {
    nanos_sdk::screen::sdk_bagl_hal_draw_bitmap_within_rect(0, 0, 128, 64, false, &BLANK);
}

use include_gif::include_gif;

pub const BLANK: [u8; 1024] = [0u8; 1024];

pub const BACK: Glyph = Glyph::from_include(include_gif!("icons/badge_back.gif"));
pub const CHECKMARK: Glyph = Glyph::from_include(include_gif!("icons/badge_check.gif"));
pub const CROSS: Glyph = Glyph::from_include(include_gif!("icons/icon_cross_badge.gif"));
pub const DOWN_ARROW: Glyph = Glyph::from_include(include_gif!("icons/icon_down.gif"));
pub const LEFT_ARROW: Glyph = Glyph::from_include(include_gif!("icons/icon_left.gif"));
pub const RIGHT_ARROW: Glyph = Glyph::from_include(include_gif!("icons/icon_right.gif"));
pub const UP_ARROW: Glyph = Glyph::from_include(include_gif!("icons/icon_up.gif"));
