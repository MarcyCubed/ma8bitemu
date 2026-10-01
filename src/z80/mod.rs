//! Zilog Z80 emulator

mod bits;
pub mod emulator;
mod indexed;
mod misc;
mod z8080;

pub use emulator::Emulator;
