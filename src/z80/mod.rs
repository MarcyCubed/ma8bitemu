//! Zilog Z80 emulator

mod bits;
pub mod emulator;
mod misc;
pub mod state;
mod z8080;

pub use emulator::Emulator;
pub use state::State;
