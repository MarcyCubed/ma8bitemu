//! The internal state of an 8080 processor

use core::num::Wrapping;

/// The internal state of an 8080 processor
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct State {
    /// Interrupt enable flip-flop
    pub inte: bool,
    /// The accumulator
    pub a: Wrapping<u8>,
    /// General purpose B register
    pub b: Wrapping<u8>,
    /// General purpose C register
    pub c: Wrapping<u8>,
    /// General purpose D register
    pub d: Wrapping<u8>,
    /// General purpose E register
    pub e: Wrapping<u8>,
    /// H register
    pub h: Wrapping<u8>,
    /// L register
    pub l: Wrapping<u8>,
    /// Stack pointer
    pub sp: Wrapping<u16>,
    /// Program counter
    pub pc: Wrapping<u16>,
    /// Carry flag
    pub cf: bool,
    /// Parity flag
    pub pf: bool,
    /// Zero flag
    pub zf: bool,
    /// Sign flag
    pub sf: bool,
    /// Auxiliary carry flag
    pub af: bool,
}

impl State {
    /// Carry flag bit
    pub const C_FLAG_BIT: u8 = 0;

    /// Parity flag bit
    pub const P_FLAG_BIT: u8 = 2;

    /// Zero flag bit
    pub const Z_FLAG_BIT: u8 = 6;

    /// Sign flag bit
    pub const S_FLAG_BIT: u8 = 7;

    /// Auxiliary carry flag bit
    pub const A_FLAG_BIT: u8 = 4;

    pub fn new() -> Self {
        Self {
            inte: false,
            a: Default::default(),
            b: Default::default(),
            c: Default::default(),
            d: Default::default(),
            e: Default::default(),
            h: Default::default(),
            l: Default::default(),
            sp: Default::default(),
            pc: Default::default(),
            cf: false,
            pf: false,
            zf: false,
            sf: false,
            af: false,
        }
    }

    /// Get the Z, S and P flags from a value
    pub fn flags_from_value(&mut self, value: u8) {
        self.zf = value == 0;
        self.sf = value & (1 << 7) != 0;
        self.pf = value.count_ones() & 1 == 0;
    }

    /// Get the Z, S and P flags from the accumulator
    pub fn flags_from_accumulator(&mut self) {
        let a = self.a.0;
        self.flags_from_value(a);
    }

    /// Get the value of the BC register pair
    pub fn bc(&self) -> u16 {
        u16::from_le_bytes([self.c.0, self.b.0])
    }

    /// Set the value of the BC register pair
    pub fn set_bc(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.c = Wrapping(bytes[0]);
        self.b = Wrapping(bytes[1]);
    }

    /// Get the value of the DE register pair
    pub fn de(&self) -> u16 {
        u16::from_le_bytes([self.e.0, self.d.0])
    }

    /// Set the value of the DE register pair
    pub fn set_de(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.e = Wrapping(bytes[0]);
        self.d = Wrapping(bytes[1]);
    }

    /// Get the value of the HL register pair
    pub fn hl(&self) -> u16 {
        u16::from_le_bytes([self.l.0, self.h.0])
    }

    /// Set the value of the HL register pair
    pub fn set_hl(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.l = Wrapping(bytes[0]);
        self.h = Wrapping(bytes[1]);
    }

    /// Get the value of some register as specified in instructions like MOV and ADD
    ///
    /// Return the value if the source is a register or `None` if it's a memory operation
    pub(crate) fn source_from_opcode(&self, opcode: u8) -> Option<u8> {
        match opcode & 0b111 {
            0b000 => Some(self.b.0),
            0b001 => Some(self.c.0),
            0b010 => Some(self.d.0),
            0b011 => Some(self.e.0),
            0b100 => Some(self.h.0),
            0b101 => Some(self.l.0),
            0b111 => Some(self.a.0),
            _ => None,
        }
    }

    /// Get the value of a condition for conditional jumps, calls and returns
    pub(crate) fn opcode_to_condition(&self, opcode: u8) -> bool {
        match (opcode >> 3) & 0b111 {
            0b000 => !self.zf,
            0b001 => self.zf,
            0b010 => !self.cf,
            0b011 => self.cf,
            0b100 => !self.pf,
            0b101 => self.pf,
            0b110 => !self.sf,
            0b111 => self.sf,
            _ => unreachable!("No opcode condition bigger than 7"),
        }
    }

    /// Turn the flags into the F register
    pub fn serialize_flags(&self) -> u8 {
        let mut val = 1 << 1; // Bit 1 is set
        if self.cf {
            val |= 1 << Self::C_FLAG_BIT
        }
        if self.sf {
            val |= 1 << Self::S_FLAG_BIT
        }
        if self.zf {
            val |= 1 << Self::Z_FLAG_BIT
        }
        if self.pf {
            val |= 1 << Self::P_FLAG_BIT
        }
        if self.af {
            val |= 1 << Self::A_FLAG_BIT
        }

        val
    }

    /// Load the flags from the bit flags
    pub fn deserialize_flags(&mut self, flags: u8) {
        self.cf = flags & (1 << Self::C_FLAG_BIT) != 0;
        self.af = flags & (1 << Self::A_FLAG_BIT) != 0;
        self.sf = flags & (1 << Self::S_FLAG_BIT) != 0;
        self.zf = flags & (1 << Self::Z_FLAG_BIT) != 0;
        self.pf = flags & (1 << Self::P_FLAG_BIT) != 0;
    }

    /// Write the state to the screen.
    ///
    /// Also shows the opcode if it's known.
    #[cfg(feature = "std")]
    pub fn dump(&self, opcode: u8) {
        print!("pc={:04x}h", self.pc);
        print!(",sp={:04x}h", self.sp);
        print!(",op={:02x}h", opcode);
        print!(",a={:02x}h", self.a);
        print!(",bc={:04x}h", self.bc());
        print!(",de={:04x}h", self.de());
        print!(",hl={:04x}h", self.hl());
        print!(",cf={}", self.cf as u8);
        print!(",pf={}", self.pf as u8);
        print!(",af={}", self.af as u8);
        print!(",zf={}", self.zf as u8);
        print!(",sf={}", self.sf as u8);
        print!(",iff={}", self.inte as u8);

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_serialization() {
        const BOOL_ARRAY: [bool; 2] = [true, false];
        for cf in BOOL_ARRAY {
            for pf in BOOL_ARRAY {
                for af in BOOL_ARRAY {
                    for zf in BOOL_ARRAY {
                        for sf in BOOL_ARRAY {
                            let mut state = State::new();
                            state.cf = cf;
                            state.pf = pf;
                            state.af = af;
                            state.zf = zf;
                            state.sf = sf;

                            let serialized = state.serialize_flags();
                            assert_eq!(serialized & (1 << 3), 0, "Bit 3 must be 0");
                            assert_eq!(serialized & (1 << 5), 0, "Bit 3 must be 0");
                            assert_eq!(serialized & (1 << 1), 1 << 1, "Bit 1 must be 1");
                            // Make a new state;
                            let mut state2 = State::new();
                            state2.deserialize_flags(serialized);
                            assert_eq!(state, state2);
                        }
                    }
                }
            }
        }
    }
}
