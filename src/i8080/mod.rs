//! Intel 8080 emulator

pub mod emulator;
mod jump;
pub(crate) mod load;
mod math;
mod state;

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
    fn get_bc(&self) -> u16;

    /// Set the value of the BC register pair
    fn set_bc(&mut self, value: u16);

    /// Get the value of the DE register pair
    fn get_de(&self) -> u16;

    /// Set the value of the DE register pair
    fn set_de(&mut self, value: u16);

    /// Get the value of the HL register pair
    fn get_hl(&self) -> u16;

    /// Set the value of the HL register pair
    fn set_hl(&mut self, value: u16);

    /// Get the value of the PC register
    fn get_pc(&self) -> Wrapping<u16>;

    /// Mutable access the PC register
    fn get_pc_mut(&mut self) -> &mut Wrapping<u16>;

    /// Set the value of the PC register
    fn set_pc(&mut self, value: u16);

    /// Get the value of the Stack Pointer register
    fn get_sp(&self) -> Wrapping<u16>;

    /// Get the value of the Stack Pointer register as an u16
    fn get_sp_u16(&self) -> u16 {
        self.get_sp().0
    }

    /// Set the value of the Stack Pointer register
    fn set_sp(&mut self, value: u16);

    /// Set the flags from a value
    fn flags_from_value(&mut self, value: u8);

    /// Set the overflow flag to the boolean value
    fn overflow_flag(&mut self, overflow: bool);

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
    fn get_af(&self) -> bool;

    /// Set the auxiliary carry flag
    fn set_af(&mut self, flag: bool);

    /// Set the subtraction flag
    fn set_nf(&mut self, flag: bool);

    /// Get the value of some register as specified in instructions like MOV and ADD
    ///
    /// Return the value if the source is a register or `None` if it's a memory operation
    fn source_from_opcode(&self, opcode: u8) -> Option<u8> {
        match opcode & 0b111 {
            0b000 => Some(self.get_b().0),
            0b001 => Some(self.get_c().0),
            0b010 => Some(self.get_d().0),
            0b011 => Some(self.get_e().0),
            0b100 => Some(self.get_h().0),
            0b101 => Some(self.get_l().0),
            0b111 => Some(self.get_a().0),
            _ => None,
        }
    }

    /// Get the value of a condition for conditional jumps, calls and returns
    fn opcode_to_condition(&self, opcode: u8) -> bool {
        match (opcode >> 3) & 0b111 {
            0b000 => !self.get_zf(),
            0b001 => self.get_zf(),
            0b010 => !self.get_cf(),
            0b011 => self.get_cf(),
            0b100 => !self.get_pf(),
            0b101 => self.get_pf(),
            0b110 => !self.get_sf(),
            0b111 => self.get_sf(),
            _ => unreachable!("No opcode condition bigger than 7"),
        }
    }
}

impl<T: I8080FamilyState> Fetch for T {
    #[inline]
    fn fetch_byte(&mut self, memory: &impl Memory) -> u8 {
        let address = self.get_pc().0;
        *self.get_pc_mut() += 1;
        memory.load(address)
    }
}
