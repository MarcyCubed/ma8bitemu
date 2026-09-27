//! Zilog Z80 emulator

pub mod emulator;
pub mod state;
mod z8080;

pub use emulator::Emulator;
pub use state::State;
