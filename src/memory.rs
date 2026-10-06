//! Useful tools to manage access to memory

/// Unified interface for memory
pub trait Memory {
    /// Load a byte from memory
    fn load(&self, address: u16) -> u8;

    /// Load a 16-bit word from memory
    fn load_16(&self, address: u16) -> u16 {
        let low = self.load(address);
        let high = self.load(address.wrapping_add(1));
        u16::from_le_bytes([low, high])
    }

    /// Store a byte in memory
    fn store(&mut self, address: u16, data: u8);

    /// Store a 16-bit word in memory
    fn store_16(&mut self, address: u16, data: u16) {
        let [low, high] = data.to_le_bytes();
        self.store(address, low);
        self.store(address.wrapping_add(1), high);
    }
}

/// ROM with a single value
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Constant(pub u8);

/// ROM with a repeating value
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Repeat<const N: usize>(pub [u8; N]);

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

impl Memory for Constant {
    fn load(&self, _address: u16) -> u8 {
        self.0
    }

    fn store(&mut self, _address: u16, _data: u8) {
        // Do nothing
    }
}

impl<const N: usize> Memory for Repeat<N> {
    fn load(&self, address: u16) -> u8 {
        let address = address as usize % N;
        self.0[address]
    }

    fn store(&mut self, _address: u16, _data: u8) {
        // Do nothing
    }
}
