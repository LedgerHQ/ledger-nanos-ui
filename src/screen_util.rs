#![allow(dead_code)]

pub fn screen_clear() {
    #[cfg(not(target_os = "nanos"))]
    unsafe {
        ledger_sdk_sys::screen_clear();
    }
}

#[cfg(not(target_os = "nanos"))]
pub fn set_keepout(x: u32, y: u32, width: u32, height: u32) {
    unsafe {
        ledger_sdk_sys::screen_set_keepout(x, y, width, height);
    }
}

pub fn bagl_hal_draw_bitmap_within_rect(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    inverted: bool,
    bitmap: &[u8],
) {
    let inverted = [inverted as u32, !inverted as u32];
    unsafe {
        ledger_sdk_sys::bagl_hal_draw_bitmap_within_rect(
            x,
            y,
            width,
            height,
            2,
            inverted.as_ptr(),
            1,
            bitmap.as_ptr(),
            width * height,
        )
    }
}

pub fn bagl_hal_draw_rect(color: u32, x: i32, y: i32, width: u32, height: u32) {
    unsafe {
        ledger_sdk_sys::bagl_hal_draw_rect(color, x, y, width, height);
    }
}

pub fn draw(x_pos: i32, y_pos: i32, w: u32, h: u32, inv: bool, bmp: &[u8]) {
    bagl_hal_draw_bitmap_within_rect(x_pos, y_pos, w, h, inv, bmp);
}

pub fn fulldraw(x_pos: i32, y_pos: i32, bmp: &[u8]) {
    draw(x_pos, y_pos, 128, 64, false, bmp);
}

pub fn screen_update() {
    #[cfg(not(target_os = "nanos"))]
    unsafe {
        ledger_sdk_sys::screen_update();
    }
}

#[cfg(not(feature = "speculos"))]
pub fn seph_setup_ticker(interval_ms: u16) {
    let ms = interval_ms.to_be_bytes();
    ledger_sdk_sys::seph::seph_send(&[0x4e, 0, 2, ms[0], ms[1]]);
}
