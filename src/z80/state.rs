//! The data stored in a Z80 processor

use crate::i8080;
use crate::i8080::I8080FamilyState;
use core::num::Wrapping;

/// The internal state of a Z80 processor
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct State {
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
    /// Index register X
    pub ix: Wrapping<u16>,
    /// Index register Y
    pub iy: Wrapping<u16>,
    /// Interrupt flip-flop
    pub iff1: bool,
    /// Temporary storage for `iff1``
    pub iff2: bool,
    /// Interruption mode of the processor
    pub interrupt_mode: InterruptMode,
    /// Alternate register A
    pub a_alt: Wrapping<u8>,
    /// Alternate register F
    pub f_alt: u8,
    /// Alternate register B
    pub b_alt: Wrapping<u8>,
    /// Alternate register C
    pub c_alt: Wrapping<u8>,
    /// Alternate register D
    pub d_alt: Wrapping<u8>,
    /// Alternate register E
    pub e_alt: Wrapping<u8>,
    /// Alternate register H
    pub h_alt: Wrapping<u8>,
    /// Alternate register L
    pub l_alt: Wrapping<u8>,
    /// Interrupt page address register
    pub i: u8,
    /// Memory refresh register
    pub r: Wrapping<u8>,
    /// Carry flag
    pub cf: bool,
    /// Parity / Overflow flag
    pub pf: bool,
    /// Zero flag
    pub zf: bool,
    /// Sign flag
    pub sf: bool,
    /// Auxiliary carry flag
    pub hf: bool,
    /// Subtraction flag
    pub nf: bool,
    /// Undocumented flag X
    pub xf: bool,
    /// Undocumented flag Y
    pub yf: bool,
    /// Undocumented register `MEMPTR`
    pub mem_ptr: Wrapping<u16>,
    /// Opcode of the `in` instruction
    pub(crate) in_opcode: u8,
}

/// How the processor handles interruptions
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum InterruptMode {
    /// Handle interrupts like the Intel 8080
    #[default]
    I8080,
    /// This mode handles interruptions by jumping to the address `0038h`
    Rst38h,
    /// This mode handles interrupts using the interrupt vector in the `I` register.
    ///
    /// When in this mode the processor performs a jump to an address formed by the value of the `I`
    /// vector as the most significant byte and the data sent by the device as the least significant
    /// byte.
    Vectored,
}

impl State {
    /// Carry flag bit
    pub const C_FLAG_BIT: u32 = i8080::State::C_FLAG_BIT;

    /// Parity flag bit
    pub const P_FLAG_BIT: u32 = i8080::State::P_FLAG_BIT;

    /// Zero flag bit
    pub const Z_FLAG_BIT: u32 = i8080::State::Z_FLAG_BIT;

    /// Sign flag bit
    pub const S_FLAG_BIT: u32 = i8080::State::S_FLAG_BIT;

    /// Auxiliary carry flag bit
    pub const H_FLAG_BIT: u32 = i8080::State::A_FLAG_BIT;

    /// Subtraction flag bit
    pub const N_FLAG_BIT: u8 = 1;

    /// Undocumented flag X bit
    pub const X_FLAG_BIT: u8 = 3;

    /// Undocumented flag Y bit
    pub const Y_FLAG_BIT: u8 = 5;

    /// Create a new Z80 state
    pub fn new() -> Self {
        Self {
            a: Wrapping(0xff),
            b: Default::default(),
            c: Default::default(),
            d: Default::default(),
            e: Default::default(),
            h: Default::default(),
            l: Default::default(),
            sp: Wrapping(0xffff),
            pc: Default::default(),
            ix: Default::default(),
            iy: Default::default(),
            iff1: false,
            iff2: false,
            interrupt_mode: Default::default(),
            a_alt: Default::default(),
            f_alt: Default::default(),
            b_alt: Default::default(),
            c_alt: Default::default(),
            d_alt: Default::default(),
            e_alt: Default::default(),
            h_alt: Default::default(),
            l_alt: Default::default(),
            i: 0,
            r: Default::default(),
            cf: true,
            pf: true,
            zf: true,
            sf: true,
            hf: true,
            nf: true,
            xf: true,
            yf: true,
            mem_ptr: Default::default(),
            in_opcode: 0,
        }
    }

    /// Set the X and Y flags from a value
    pub fn xy_from_value(&mut self, value: u8) {
        self.xf = value & (1 << Self::X_FLAG_BIT) != 0;
        self.yf = value & (1 << Self::Y_FLAG_BIT) != 0;
    }

    /// Set the X and Y flags from the accumulator
    pub fn xy_from_accumulator(&mut self) {
        self.xy_from_value(self.a.0)
    }

    /// Increment the R register
    pub fn inc_r(&mut self) {
        const MASK: u8 = 1 << 7;
        let msb = self.r.0 & MASK;
        self.r += 1;
        self.r &= !MASK;
        self.r |= msb;
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
        self.xf = value & (1 << Self::X_FLAG_BIT) != 0;
        self.yf = value & (1 << Self::Y_FLAG_BIT) != 0;
    }

    #[inline]
    fn overflow_flag(&mut self, overflow: bool) {
        self.pf = overflow
    }

    #[inline]
    fn parity_flag(&mut self, value: u8) {
        self.pf = value.count_ones() & 1 == 0;
    }

    /// Turn the flags into the F register
    fn serialize_flags(&self) -> u8 {
        let mut val = 0;
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
        if self.hf {
            val |= 1 << Self::H_FLAG_BIT
        }
        if self.nf {
            val |= 1 << Self::N_FLAG_BIT
        }
        if self.xf {
            val |= 1 << Self::X_FLAG_BIT
        }
        if self.yf {
            val |= 1 << Self::Y_FLAG_BIT
        }

        val
    }

    /// Load the flags from the bit flags
    fn deserialize_flags(&mut self, flags: u8) {
        self.cf = flags & (1 << Self::C_FLAG_BIT) != 0;
        self.hf = flags & (1 << Self::H_FLAG_BIT) != 0;
        self.sf = flags & (1 << Self::S_FLAG_BIT) != 0;
        self.zf = flags & (1 << Self::Z_FLAG_BIT) != 0;
        self.pf = flags & (1 << Self::P_FLAG_BIT) != 0;
        self.nf = flags & (1 << Self::N_FLAG_BIT) != 0;
        self.xf = flags & (1 << Self::X_FLAG_BIT) != 0;
        self.yf = flags & (1 << Self::Y_FLAG_BIT) != 0;
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
        self.hf
    }

    #[inline]
    fn set_hf(&mut self, flag: bool) {
        self.hf = flag
    }

    #[inline]
    fn set_nf(&mut self, flag: bool) {
        self.nf = flag
    }

    fn dump(&self, opcode: u8) {
        print!("pc={:04x}h", self.pc);
        print!(",sp={:04x}h", self.sp);
        print!(",op={:02x}h", opcode);
        print!(",af={:02x}{:02x}h", self.a, self.serialize_flags());
        print!(",bc={:04x}h", self.get_bc());
        print!(",de={:04x}h", self.get_de());
        print!(",hl={:04x}h", self.get_hl());
        print!(",ix={:04x}h", self.ix);
        print!(",iy={:04x}h", self.iy);
        print!(",i={:02x}h", self.i);
        print!(",r={:02x}h", self.r);
        print!(",af'={:02x}{:02x}h", self.a_alt, self.f_alt);
        print!(",bc'={:02x}{:02x}h", self.b_alt, self.c_alt);
        print!(",de'={:02x}{:02x}h", self.d_alt, self.e_alt);
        print!(",hl'={:02x}{:02x}h", self.h_alt, self.l_alt);
        print!(",c={}", self.cf as u8);
        print!(",po={}", self.pf as u8);
        print!(",hc={}", self.hf as u8);
        print!(",n={}", self.nf as u8);
        print!(",z={}", self.zf as u8);
        print!(",s={}", self.sf as u8);
        print!(",memptr={:04x}h", self.mem_ptr);

        println!();
    }

    fn set_memptr(&mut self, address: u16) {
        self.mem_ptr.0 = address
    }
}
