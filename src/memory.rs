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
