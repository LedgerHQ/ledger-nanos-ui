#![no_std]

#[cfg(target_os = "nanos")]
pub mod bagls;

#[cfg(not(target_os = "nanos"))]
pub mod se_bagls;

#[cfg(not(target_os = "nanos"))]
pub mod string_se;

#[cfg(target_os = "nanos")]
pub mod string_mcu;

pub mod bitmaps;
pub mod fonts;
pub mod layout;

pub mod screen_util;
pub mod ui;

pub const PADDING: usize = 2;
pub const SCREEN_WIDTH: usize = 128;

#[cfg(target_os = "nanos")]
pub const SCREEN_HEIGHT: usize = 32;

#[cfg(not(target_os = "nanos"))]
pub const SCREEN_HEIGHT: usize = 64;
