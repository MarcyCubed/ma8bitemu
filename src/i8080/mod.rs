//! Intel 8080 emulator

pub mod emulator;
pub(crate) mod jump;
pub(crate) mod load;
pub(crate) mod math;
pub mod state;

use core::num::Wrapping;
// Re-export the emulator with a shorter name
use crate::Fetch;
use crate::memory::Memory;
pub use emulator::Emulator;

/// Abstraction over the state of Intel 8080 processors and its relatives
pub trait I8080FamilyState {
    /// Get the value of the register A
    fn get_a(&self) -> Wrapping<u8>;

    /// Get the value of the register B
    fn get_b(&self) -> Wrapping<u8>;

    /// Get the value of the register C
    fn get_c(&self) -> Wrapping<u8>;

    /// Get the value of the register D
    fn get_d(&self) -> Wrapping<u8>;

    /// Get the value of the register E
    fn get_e(&self) -> Wrapping<u8>;

    /// Get the value of the register H
    fn get_h(&self) -> Wrapping<u8>;

    /// Get the value of the register L
    fn get_l(&self) -> Wrapping<u8>;

    /// Mutable access the register A
    fn get_a_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register B
    fn get_b_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register C
    fn get_c_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register D
    fn get_d_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register E
    fn get_e_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register H
    fn get_h_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register L
    fn get_l_mut(&mut self) -> &mut Wrapping<u8>;

    /// Get the value of the BC register pair
    #[inline]
    fn get_bc(&self) -> u16 {
        u16::from_le_bytes([self.get_c().0, self.get_b().0])
    }

    /// Set the value of the BC register pair
    #[inline]
    fn set_bc(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.get_c_mut().0 = bytes[0];
        self.get_b_mut().0 = bytes[1];
    }

    /// Get the value of the DE register pair
    #[inline]
    fn get_de(&self) -> u16 {
        u16::from_le_bytes([self.get_e().0, self.get_d().0])
    }

    /// Set the value of the DE register pair
    #[inline]
    fn set_de(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.get_e_mut().0 = bytes[0];
        self.get_d_mut().0 = bytes[1];
    }

    /// Get the value of the HL register pair
    #[inline]
    fn get_hl(&self) -> u16 {
        u16::from_le_bytes([self.get_l().0, self.get_h().0])
    }

    /// Set the value of the HL register pair
    #[inline]
    fn set_hl(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.get_l_mut().0 = bytes[0];
        self.get_h_mut().0 = bytes[1];
    }

    /// Get the value of the AF register pair
    #[inline]
    fn get_af(&self) -> u16 {
        u16::from_le_bytes([self.serialize_flags(), self.get_a().0])
    }

    /// Set the value of the AF register pair
    #[inline]
    fn set_af(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.deserialize_flags(bytes[0]);
        self.get_a_mut().0 = bytes[1];
    }

    /// Get the value of the PC register
    fn get_pc(&self) -> Wrapping<u16>;

    /// Mutable access the PC register
    fn get_pc_mut(&mut self) -> &mut Wrapping<u16>;

    /// Set the value of the PC register
    fn set_pc(&mut self, value: u16) {
        self.get_pc_mut().0 = value;
    }

    /// Get the value of the Stack Pointer register
    fn get_sp(&self) -> Wrapping<u16>;

    /// Get the value of the Stack Pointer register as an u16
    fn get_sp_u16(&self) -> u16 {
        self.get_sp().0
    }

    /// Set the value of the Stack Pointer register
    fn set_sp(&mut self, value: u16) {
        self.get_sp_mut().0 = value
    }

    /// Mutable access the Stack Pointer register
    fn get_sp_mut(&mut self) -> &mut Wrapping<u16>;

    /// Set the flags from a value
    fn flags_from_value(&mut self, value: u8);

    /// Get the default flags from the accumulator
    #[inline]
    fn flags_from_accumulator(&mut self) {
        let a = self.get_a().0;
        self.flags_from_value(a);
    }

    /// Set the overflow flag to the boolean value
    fn overflow_flag(&mut self, overflow: bool);

    /// Set the parity flag from a value
    fn parity_flag(&mut self, value: u8);

    /// Get the parity flag from the accumulator
    #[inline]
    fn parity_from_accumulator(&mut self) {
        let a = self.get_a().0;
        self.parity_flag(a);
    }

    /// Turn the flags into a bitmap
    fn serialize_flags(&self) -> u8;

    /// Load the flags from the bit flags
    fn deserialize_flags(&mut self, flags: u8);

    /// Get the carry flag
    fn get_cf(&self) -> bool;

    /// Set the carry flag
    fn set_cf(&mut self, flag: bool);

    /// Get the parity flag
    fn get_pf(&self) -> bool;

    /// Set the parity flag
    fn set_pf(&mut self, flag: bool);

    /// Get the zero flag
    fn get_zf(&self) -> bool;

    /// Set the zero flag
    fn set_zf(&mut self, flag: bool);

    /// Get the sign flag
    fn get_sf(&self) -> bool;

    /// Set the sign flag
    fn set_sf(&mut self, flag: bool);

    /// Get the auxiliary carry flag
    fn get_hf(&self) -> bool;

    /// Set the auxiliary carry flag
    fn set_hf(&mut self, flag: bool);

    /// Set the subtraction flag
    fn set_nf(&mut self, flag: bool);
}

impl<T: I8080FamilyState> Fetch for T {
    #[inline]
    fn fetch_byte(&mut self, memory: &impl Memory) -> u8 {
        let address = self.get_pc().0;
        *self.get_pc_mut() += 1;
        memory.load(address)
    }
}
