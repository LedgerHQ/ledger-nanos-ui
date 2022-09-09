#![no_std]

// Guard against compiling for anything other than a Nano S for now
#[cfg(not(target_os = "nanos"))]
mod non_existent;

pub mod bagls;
pub mod ui;