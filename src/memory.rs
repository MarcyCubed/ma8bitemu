//! Useful tools to manage access to memory

/// Unified interface for memory
pub trait Memory {
    /// Load a byte from memory
    fn load(&self, address: u16) -> u8;

    /// Store a byte to memory
    fn store(&mut self, address: u16, data: u8);
}

impl<const N: usize> Memory for [u8; N] {
    fn load(&self, address: u16) -> u8 {
        self[address as usize]
    }

    fn store(&mut self, address: u16, data: u8) {
        self[address as usize] = data;
    }
}

impl Memory for [u8] {
    fn load(&self, address: u16) -> u8 {
        self[address as usize]
    }

    fn store(&mut self, address: u16, data: u8) {
        self[address as usize] = data;
    }
}
