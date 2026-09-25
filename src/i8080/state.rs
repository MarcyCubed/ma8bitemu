//! The internal state of an 8080 processor

use crate::i8080::I8080FamilyState;
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

    /// Write the state to the screen.
    ///
    /// Also shows the opcode if it's known.
    #[cfg(feature = "std")]
    pub fn dump(&self, opcode: u8) {
        print!("pc={:04x}h", self.pc);
        print!(",sp={:04x}h", self.sp);
        print!(",op={:02x}h", opcode);
        print!(",a={:02x}h", self.a);
        print!(",bc={:04x}h", self.get_bc());
        print!(",de={:04x}h", self.get_de());
        print!(",hl={:04x}h", self.get_hl());
        print!(",cf={}", self.cf as u8);
        print!(",pf={}", self.pf as u8);
        print!(",af={}", self.af as u8);
        print!(",zf={}", self.zf as u8);
        print!(",sf={}", self.sf as u8);
        print!(",iff={}", self.inte as u8);

        println!();
    }
}

impl I8080FamilyState for State {
    #[inline]
    fn get_a(&self) -> Wrapping<u8> {
        self.a
    }

    #[inline]
    fn get_b(&self) -> Wrapping<u8> {
        self.b
    }

    #[inline]
    fn get_c(&self) -> Wrapping<u8> {
        self.c
    }

    #[inline]
    fn get_d(&self) -> Wrapping<u8> {
        self.d
    }

    #[inline]
    fn get_e(&self) -> Wrapping<u8> {
        self.e
    }

    #[inline]
    fn get_h(&self) -> Wrapping<u8> {
        self.h
    }

    #[inline]
    fn get_l(&self) -> Wrapping<u8> {
        self.l
    }

    #[inline]
    fn get_a_mut(&mut self) -> &mut Wrapping<u8> {
        &mut self.a
    }

    #[inline]
    fn get_b_mut(&mut self) -> &mut Wrapping<u8> {
        &mut self.b
    }

    #[inline]
    fn get_c_mut(&mut self) -> &mut Wrapping<u8> {
        &mut self.c
    }

    #[inline]
    fn get_d_mut(&mut self) -> &mut Wrapping<u8> {
        &mut self.d
    }

    #[inline]
    fn get_e_mut(&mut self) -> &mut Wrapping<u8> {
        &mut self.e
    }

    #[inline]
    fn get_h_mut(&mut self) -> &mut Wrapping<u8> {
        &mut self.h
    }

    #[inline]
    fn get_l_mut(&mut self) -> &mut Wrapping<u8> {
        &mut self.l
    }

    #[inline]
    fn get_pc(&self) -> Wrapping<u16> {
        self.pc
    }

    #[inline]
    fn get_pc_mut(&mut self) -> &mut Wrapping<u16> {
        &mut self.pc
    }

    #[inline]
    fn get_sp(&self) -> Wrapping<u16> {
        self.sp
    }

    #[inline]
    fn get_sp_mut(&mut self) -> &mut Wrapping<u16> {
        &mut self.sp
    }

    #[inline]
    fn flags_from_value(&mut self, value: u8) {
        self.zf = value == 0;
        self.sf = value & (1 << 7) != 0;
        self.pf = value.count_ones() & 1 == 0;
    }

    #[inline]
    fn overflow_flag(&mut self, _overflow: bool) {
        // No overflow in 8080
    }

    #[inline]
    fn parity_flag(&mut self, _value: u8) {
        // Already done by flags_from_value
    }

    /// Turn the flags into the F register
    fn serialize_flags(&self) -> u8 {
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
    fn deserialize_flags(&mut self, flags: u8) {
        self.cf = flags & (1 << Self::C_FLAG_BIT) != 0;
        self.af = flags & (1 << Self::A_FLAG_BIT) != 0;
        self.sf = flags & (1 << Self::S_FLAG_BIT) != 0;
        self.zf = flags & (1 << Self::Z_FLAG_BIT) != 0;
        self.pf = flags & (1 << Self::P_FLAG_BIT) != 0;
    }

    #[inline]
    fn get_cf(&self) -> bool {
        self.cf
    }

    #[inline]
    fn set_cf(&mut self, flag: bool) {
        self.cf = flag
    }

    #[inline]
    fn get_pf(&self) -> bool {
        self.pf
    }

    #[inline]
    fn set_pf(&mut self, flag: bool) {
        self.pf = flag
    }

    #[inline]
    fn get_zf(&self) -> bool {
        self.zf
    }

    #[inline]
    fn set_zf(&mut self, flag: bool) {
        self.zf = flag
    }

    #[inline]
    fn get_sf(&self) -> bool {
        self.sf
    }

    #[inline]
    fn set_sf(&mut self, flag: bool) {
        self.sf = flag
    }

    #[inline]
    fn get_hf(&self) -> bool {
        self.af
    }

    #[inline]
    fn set_hf(&mut self, flag: bool) {
        self.af = flag
    }

    fn set_nf(&mut self, _flag: bool) {
        // Do nothing
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
