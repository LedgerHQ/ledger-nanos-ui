
#[cfg(target_os = "nanos")]
pub mod mcu;
#[cfg(target_os = "nanos")]
pub use self::mcu::*;

#[cfg(not(target_os = "nanos"))]
pub mod se;
#[cfg(not(target_os = "nanos"))]
pub use self::se::*;

pub struct RectFull {
    pos: (i32, i32),
    width: u32,
    height: u32
}

impl RectFull {
    pub const fn new() -> RectFull {
        RectFull {
            pos: (0, 0),
            width: 1,
            height: 1
        }
    }
    pub const fn pos(self, x: i32, y: i32) -> RectFull {
        RectFull {
            pos: (x, y),
            ..self
        }
    }
    pub const fn width(self, width: u32) -> RectFull {
        RectFull {
            width,
            ..self
        }
    }
    pub const fn height(self, height: u32) -> RectFull {
        RectFull {
            height,
            ..self
        }
    }
}

