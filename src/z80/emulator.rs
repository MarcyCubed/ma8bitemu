//! The core of the Z80 emulator

use crate::i8080::I8080FamilyEmulator;
use crate::memory::Memory;
use crate::z80::{bits, indexed, misc, z8080};
use crate::{EmulatorCore, ExecEffect, Fetch, i8080};
use core::mem;
use std::num::Wrapping;

/// The Z80 emulator
#[derive(Debug, Clone, Copy)]
pub struct Emulator {
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
    /// The effect of the last instruction
    last_effect: ExecEffect,
    /// The interrupt vector if an interrupt was caused by external hardware
    interrupt_vector: Option<u8>,
    /// Is there a pending NMI?
    nmi_pending: bool,
    /// The input continuation
    pub(super) input_continuation: Option<InputContinuation>,
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

/// Function that handles receiving input and finishing the IN instruction
pub(super) type InputContinuation = fn(&mut Emulator, u8);

impl Emulator {
    /// Create a Z80 emulator
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
            last_effect: ExecEffect::Normal,
            interrupt_vector: None,
            nmi_pending: false,
            input_continuation: None,
        }
    }

    /// Carry flag bit
    pub const C_FLAG_BIT: u32 = i8080::Emulator::C_FLAG_BIT;

    /// Parity flag bit
    pub const P_FLAG_BIT: u32 = i8080::Emulator::P_FLAG_BIT;

    /// Zero flag bit
    pub const Z_FLAG_BIT: u32 = i8080::Emulator::Z_FLAG_BIT;

    /// Sign flag bit
    pub const S_FLAG_BIT: u32 = i8080::Emulator::S_FLAG_BIT;

    /// Auxiliary carry flag bit
    pub const H_FLAG_BIT: u32 = i8080::Emulator::A_FLAG_BIT;

    /// Subtraction flag bit
    pub const N_FLAG_BIT: u8 = 1;

    /// Undocumented flag X bit
    pub const X_FLAG_BIT: u8 = 3;

    /// Undocumented flag Y bit
    pub const Y_FLAG_BIT: u8 = 5;

    /// Relative jump instruction
    fn jr(&mut self, cond: fn(&Emulator) -> bool, memory: &impl Memory) -> u8 {
        let d = self.fetch_byte(memory);
        self.mem_ptr = self.pc;
        self.mem_ptr += d as u16;

        if cond(&self) {
            self.pc = self.mem_ptr;
            12
        } else {
            7
        }
    }

    /// Set the S and Z flags from a value
    pub fn sz_from_value(&mut self, value: u8) {
        self.zf = value == 0;
        self.sf = value & (1 << 7) != 0;
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

/// Implement the instruction `sub r`
macro_rules! sub_r {
    ($emulator:ident, $register:ident) => {{
        let value = $emulator.$register.0;
        z8080::sub_value($emulator, value, false);
        4
    }};
}

/// Implement the instruction `sbc r`
macro_rules! sbc_r {
    ($emulator:ident, $register:ident) => {{
        let value = $emulator.$register.0;
        let carry = $emulator.cf;
        z8080::sub_value($emulator, value, carry);
        4
    }};
}

/// Implement the instruction `and r`
macro_rules! and_r {
    ($emulator:ident, $register:ident) => {{
        let value = $emulator.$register.0;
        z8080::and_value($emulator, value);
        4
    }};
}

/// Implement the instruction `cp r`
macro_rules! cp_r {
    ($emulator:ident, $register:ident) => {{
        let value = $emulator.$register.0;
        z8080::cp_value($emulator, value);
        4
    }};
}

impl EmulatorCore for Emulator {
    fn run_opcode(&mut self, opcode: u8, memory: &mut impl Memory) -> (u8, ExecEffect) {
        // Increase the R register
        self.inc_r();
        self.input_continuation = None;
        // Execute the instruction
        let result = 'main: {
            let clock_cycles = match opcode {
                0x00 => 4, // nop
                // ld bc, nn
                0x01 => i8080::load::lxi(self, memory, |emulator, value| emulator.set_bc(value)),
                // ld de, nn
                0x11 => i8080::load::lxi(self, memory, |emulator, value| emulator.set_de(value)),
                // ld hl, nn
                0x21 => i8080::load::lxi(self, memory, |emulator, value| emulator.set_hl(value)),
                // ld sp, nn
                0x31 => i8080::load::lxi(self, memory, |emulator, value| emulator.set_sp(value)),
                0x02 => i8080::load::stax(self, memory, |e| e.get_bc()), // ld (bc), a
                0x12 => i8080::load::stax(self, memory, |e| e.get_de()), // ld (de), a
                0x22 => i8080::load::shld_rr(self, memory, Self::get_hl), // ld (nn), hl
                0x32 => i8080::load::sta(self, memory),                  // ld (nn), a
                0x03 => i8080::math::inx(self, Self::get_bc, Self::set_bc), // inc bc
                0x13 => i8080::math::inx(self, Self::get_de, Self::set_de), // inc de
                0x23 => i8080::math::inx(self, Self::get_hl, Self::set_hl), // inc HL
                0x33 => i8080::math::inx(self, Self::get_sp_u16, Self::set_sp), // inc sp
                0x04 => i8080::math::inc(self, |e| &mut e.b),            // inc b
                0x14 => i8080::math::inc(self, |e| &mut e.d),            // inc d
                0x24 => i8080::math::inc(self, |e| &mut e.h),            // inc h
                0x0c => i8080::math::inc(self, |e| &mut e.c),            // inc c
                0x1c => i8080::math::inc(self, |e| &mut e.e),            // inc e
                0x2c => i8080::math::inc(self, |e| &mut e.l),            // inc l
                0x3c => i8080::math::inc(self, |e| &mut e.a),            // inc a
                0x34 => i8080::math::inc_mem(self, memory),              // inc (HL)
                0x05 => z8080::dec_r(self, |e| &mut e.b),                // dec b
                0x15 => z8080::dec_r(self, |e| &mut e.d),                // dec d
                0x25 => z8080::dec_r(self, |e| &mut e.h),                // dec h
                0x0d => z8080::dec_r(self, |e| &mut e.c),                // dec c
                0x1d => z8080::dec_r(self, |e| &mut e.e),                // dec e
                0x2d => z8080::dec_r(self, |e| &mut e.l),                // dec l
                0x3d => z8080::dec_r(self, |e| &mut e.a),                // dec a
                // dec (hl)
                0x35 => {
                    let address = self.get_hl();
                    z8080::dec_mem(self, memory, address, 11)
                }
                0x06 => i8080::load::mvi(self, memory, Self::get_b_mut), // ld b, n
                0x16 => i8080::load::mvi(self, memory, Self::get_d_mut), // ld d, n
                0x26 => i8080::load::mvi(self, memory, Self::get_h_mut), // ld h, n
                0x0e => i8080::load::mvi(self, memory, Self::get_c_mut), // ld c, n
                0x1e => i8080::load::mvi(self, memory, Self::get_e_mut), // ld e, n
                0x2e => i8080::load::mvi(self, memory, Self::get_l_mut), // ld l, n
                0x3e => i8080::load::mvi(self, memory, Self::get_a_mut), // ld a, n
                0x36 => i8080::load::mvi_mem(self, memory),              // ld (hl), n
                0x07 => z8080::rlca(self),                               // rlca
                0x17 => z8080::rla(self),                                // rla
                0x27 => z8080::daa(self),                                // daa
                0x37 => z8080::scf(self),                                // sfc
                // add hl, bc
                0x09 => {
                    let value = self.get_bc();
                    let hl = z8080::double_add_flags(self, self.get_hl(), value, false);
                    self.set_hl(hl);
                    11
                }
                // add hl, de
                0x19 => {
                    let value = self.get_de();
                    let hl = z8080::double_add_flags(self, self.get_hl(), value, false);
                    self.set_hl(hl);
                    11
                }
                // add hl, hl
                0x29 => {
                    let value = self.get_hl();
                    let hl = z8080::double_add_flags(self, self.get_hl(), value, false);
                    self.set_hl(hl);
                    11
                }
                // add hl, sp
                0x39 => {
                    let value = self.sp.0;
                    let hl = z8080::double_add_flags(self, self.get_hl(), value, false);
                    self.set_hl(hl);
                    11
                }
                0x0a => i8080::load::ldax(self, memory, Self::get_bc), // ld a, (bc)
                0x1a => i8080::load::ldax(self, memory, Self::get_de), // ld a, (de)
                0x2a => i8080::load::lhld_rr(self, memory, Self::set_hl), // ld hl, (nn)
                0x3a => i8080::load::lda(self, memory),                // ld a, (nn)
                0x0b => i8080::math::dcx(self, Self::get_bc, Self::set_bc), // dec bc
                0x1b => i8080::math::dcx(self, Self::get_de, Self::set_de), // dec de
                0x2b => i8080::math::dcx(self, Self::get_hl, Self::set_hl), // dec hl
                0x3b => i8080::math::dcx(self, Self::get_sp_u16, Self::set_sp), // dec sp
                0x0f => z8080::rrca(self),                             // rrca
                0x1F => i8080::math::rar(self),                        // rra
                0x2f => z8080::cpl(self),                              // cpl
                0x3f => z8080::ccf(self),                              // ccf
                // ld x, y
                0x40 => 4, // B, B
                0x41 => i8080::load::mov(self, Self::get_b_mut, Self::get_c, 4),
                0x42 => i8080::load::mov(self, Self::get_b_mut, Self::get_d, 4),
                0x43 => i8080::load::mov(self, Self::get_b_mut, Self::get_e, 4),
                0x44 => i8080::load::mov(self, Self::get_b_mut, Self::get_h, 4),
                0x45 => i8080::load::mov(self, Self::get_b_mut, Self::get_l, 4),
                0x46 => i8080::load::mov_r_mem(self, Self::get_b_mut, memory),
                0x47 => i8080::load::mov(self, Self::get_b_mut, Self::get_a, 4),
                0x48 => i8080::load::mov(self, Self::get_c_mut, Self::get_b, 4),
                0x49 => 4, // C, C
                0x4a => i8080::load::mov(self, Self::get_c_mut, Self::get_d, 4),
                0x4b => i8080::load::mov(self, Self::get_c_mut, Self::get_e, 4),
                0x4c => i8080::load::mov(self, Self::get_c_mut, Self::get_h, 4),
                0x4d => i8080::load::mov(self, Self::get_c_mut, Self::get_l, 4),
                0x4e => i8080::load::mov_r_mem(self, Self::get_c_mut, memory),
                0x4f => i8080::load::mov(self, Self::get_c_mut, Self::get_a, 4),
                0x50 => i8080::load::mov(self, Self::get_d_mut, Self::get_b, 4),
                0x51 => i8080::load::mov(self, Self::get_d_mut, Self::get_c, 4),
                0x52 => 4, // D, D
                0x53 => i8080::load::mov(self, Self::get_d_mut, Self::get_e, 4),
                0x54 => i8080::load::mov(self, Self::get_d_mut, Self::get_h, 4),
                0x55 => i8080::load::mov(self, Self::get_d_mut, Self::get_l, 4),
                0x56 => i8080::load::mov_r_mem(self, Self::get_d_mut, memory),
                0x57 => i8080::load::mov(self, Self::get_d_mut, Self::get_a, 4),
                0x58 => i8080::load::mov(self, Self::get_e_mut, Self::get_b, 4),
                0x59 => i8080::load::mov(self, Self::get_e_mut, Self::get_c, 4),
                0x5a => i8080::load::mov(self, Self::get_e_mut, Self::get_d, 4),
                0x5b => 4, // E, E
                0x5c => i8080::load::mov(self, Self::get_e_mut, Self::get_h, 4),
                0x5d => i8080::load::mov(self, Self::get_e_mut, Self::get_l, 4),
                0x5e => i8080::load::mov_r_mem(self, Self::get_e_mut, memory),
                0x5f => i8080::load::mov(self, Self::get_e_mut, Self::get_a, 4),
                0x60 => i8080::load::mov(self, Self::get_h_mut, Self::get_b, 4),
                0x61 => i8080::load::mov(self, Self::get_h_mut, Self::get_c, 4),
                0x62 => i8080::load::mov(self, Self::get_h_mut, Self::get_d, 4),
                0x63 => i8080::load::mov(self, Self::get_h_mut, Self::get_e, 4),
                0x64 => 4, // H, H
                0x65 => i8080::load::mov(self, Self::get_h_mut, Self::get_l, 4),
                0x66 => i8080::load::mov_r_mem(self, Self::get_h_mut, memory),
                0x67 => i8080::load::mov(self, Self::get_h_mut, Self::get_a, 4),
                0x68 => i8080::load::mov(self, Self::get_l_mut, Self::get_b, 4),
                0x69 => i8080::load::mov(self, Self::get_l_mut, Self::get_c, 4),
                0x6a => i8080::load::mov(self, Self::get_l_mut, Self::get_d, 4),
                0x6b => i8080::load::mov(self, Self::get_l_mut, Self::get_e, 4),
                0x6c => i8080::load::mov(self, Self::get_l_mut, Self::get_h, 4),
                0x6d => 4, // L, L
                0x6e => i8080::load::mov_r_mem(self, Self::get_l_mut, memory),
                0x6f => i8080::load::mov(self, Self::get_l_mut, Self::get_a, 4),
                0x78 => i8080::load::mov(self, Self::get_a_mut, Self::get_b, 4),
                0x79 => i8080::load::mov(self, Self::get_a_mut, Self::get_c, 4),
                0x7a => i8080::load::mov(self, Self::get_a_mut, Self::get_d, 4),
                0x7b => i8080::load::mov(self, Self::get_a_mut, Self::get_e, 4),
                0x7c => i8080::load::mov(self, Self::get_a_mut, Self::get_h, 4),
                0x7d => i8080::load::mov(self, Self::get_a_mut, Self::get_l, 4),
                0x7e => i8080::load::mov_r_mem(self, Self::get_a_mut, memory),
                0x7f => 4, // A, A
                0x70 => i8080::load::mov_mem_r(self, memory, Self::get_b),
                0x71 => i8080::load::mov_mem_r(self, memory, Self::get_c),
                0x72 => i8080::load::mov_mem_r(self, memory, Self::get_d),
                0x73 => i8080::load::mov_mem_r(self, memory, Self::get_e),
                0x74 => i8080::load::mov_mem_r(self, memory, Self::get_h),
                0x75 => i8080::load::mov_mem_r(self, memory, Self::get_l),
                0x77 => i8080::load::mov_mem_r(self, memory, Self::get_a),
                // halt
                0x76 => {
                    self.pc -= 1;
                    break 'main (4, ExecEffect::Halt);
                }
                // ADD X
                0x80 => i8080::math::add_r(self, Self::get_b), // add a, b
                0x81 => i8080::math::add_r(self, Self::get_c), // add a, c
                0x82 => i8080::math::add_r(self, Self::get_d), // add a, d
                0x83 => i8080::math::add_r(self, Self::get_e), // add a, e
                0x84 => i8080::math::add_r(self, Self::get_h), // add a, h
                0x85 => i8080::math::add_r(self, Self::get_l), // add a, l
                0x86 => i8080::math::add_mem(self, memory),    // add a, (hl)
                0x87 => i8080::math::add_r(self, Self::get_a), // add a, a
                // ADC
                0x88 => i8080::math::adc_r(self, Self::get_b),
                0x89 => i8080::math::adc_r(self, Self::get_c),
                0x8a => i8080::math::adc_r(self, Self::get_d),
                0x8b => i8080::math::adc_r(self, Self::get_e),
                0x8c => i8080::math::adc_r(self, Self::get_h),
                0x8d => i8080::math::adc_r(self, Self::get_l),
                0x8e => i8080::math::adc_mem(self, memory),
                0x8f => i8080::math::adc_r(self, Self::get_a),
                0x90 => sub_r!(self, b), // sub b
                0x91 => sub_r!(self, c), // sub c
                0x92 => sub_r!(self, d), // sub d
                0x93 => sub_r!(self, e), // sub e
                0x94 => sub_r!(self, h), // sub h
                0x95 => sub_r!(self, l), // sub l
                // sub (hl)
                0x96 => {
                    let value = self.load_hl(memory);
                    z8080::sub_value(self, value, false);
                    7
                }
                0x97 => sub_r!(self, a), // sub a
                0x98 => sbc_r!(self, b), // sbc b
                0x99 => sbc_r!(self, c), // sbc c
                0x9a => sbc_r!(self, d), // sbc d
                0x9b => sbc_r!(self, e), // sbc e
                0x9c => sbc_r!(self, h), // sbc h
                0x9d => sbc_r!(self, l), // sbc l
                // sbc (hl)
                0x9e => {
                    let value = self.load_hl(memory);
                    let carry = self.cf;
                    z8080::sub_value(self, value, carry);
                    7
                }
                0x9f => sbc_r!(self, a), // sbc a
                0xa0 => and_r!(self, b), // and b
                0xa1 => and_r!(self, c), // and c
                0xa2 => and_r!(self, d), // and d
                0xa3 => and_r!(self, e), // and e
                0xa4 => and_r!(self, h), // and h
                0xa5 => and_r!(self, l), // and l
                // and (hl)
                0xa6 => {
                    let value = self.load_hl(memory);
                    z8080::and_value(self, value);
                    7
                }
                0xa7 => and_r!(self, a), // and a
                // XRA
                0xa8 => i8080::math::xra_r(self, Self::get_b),
                0xa9 => i8080::math::xra_r(self, Self::get_c),
                0xaa => i8080::math::xra_r(self, Self::get_d),
                0xab => i8080::math::xra_r(self, Self::get_e),
                0xac => i8080::math::xra_r(self, Self::get_h),
                0xad => i8080::math::xra_r(self, Self::get_l),
                0xae => i8080::math::xra_mem(self, memory),
                0xaf => i8080::math::xra_r(self, Self::get_a),
                // ORA
                0xb0 => i8080::math::ora_r(self, Self::get_b),
                0xb1 => i8080::math::ora_r(self, Self::get_c),
                0xb2 => i8080::math::ora_r(self, Self::get_d),
                0xb3 => i8080::math::ora_r(self, Self::get_e),
                0xb4 => i8080::math::ora_r(self, Self::get_h),
                0xb5 => i8080::math::ora_r(self, Self::get_l),
                0xb6 => i8080::math::ora_mem(self, memory),
                0xb7 => i8080::math::ora_r(self, Self::get_a),
                0xb8 => cp_r!(self, b), // cp b
                0xb9 => cp_r!(self, c), // cp c
                0xba => cp_r!(self, d), // cp d
                0xbb => cp_r!(self, e), // cp e
                0xbc => cp_r!(self, h), // cp h
                0xbd => cp_r!(self, l), // cp l
                // cp (hl)
                0xbe => {
                    let value = self.load_hl(memory);
                    z8080::cp_value(self, value);
                    7
                }
                0xbf => cp_r!(self, a), // cp a
                0xc0 => i8080::jump::ret_cond(self, memory, |e| !e.zf), // RNZ
                0xc8 => i8080::jump::ret_cond(self, memory, |e| e.zf), // RZ
                0xd0 => i8080::jump::ret_cond(self, memory, |e| !e.cf), // RNC
                0xd8 => i8080::jump::ret_cond(self, memory, |e| e.cf), // RC
                0xe0 => i8080::jump::ret_cond(self, memory, |e| !e.pf), // RPO
                0xe8 => i8080::jump::ret_cond(self, memory, |e| e.pf), // RPE
                0xf0 => i8080::jump::ret_cond(self, memory, |e| !e.sf), // RP
                0xf8 => i8080::jump::ret_cond(self, memory, |e| e.sf), // RM
                0xc9 => i8080::jump::ret(self, memory), // RET
                0xc1 => i8080::jump::pop(self, memory, Self::set_bc), // POP B
                0xd1 => i8080::jump::pop(self, memory, Self::set_de), // POP D
                0xe1 => i8080::jump::pop(self, memory, Self::set_hl), // POP E
                0xf1 => i8080::jump::pop(self, memory, Self::set_af), // POP PSW
                0xc2 => i8080::jump::jp_cond_nn(self, memory, |e| !e.zf), // JNZ
                0xca => i8080::jump::jp_cond_nn(self, memory, |e| e.zf), // JZ
                0xd2 => i8080::jump::jp_cond_nn(self, memory, |e| !e.cf), // JNC
                0xda => i8080::jump::jp_cond_nn(self, memory, |e| e.cf), // JC
                0xe2 => i8080::jump::jp_cond_nn(self, memory, |e| !e.pf), // JPO
                0xea => i8080::jump::jp_cond_nn(self, memory, |e| e.pf), // JPE
                0xf2 => i8080::jump::jp_cond_nn(self, memory, |e| !e.sf), // JP
                0xfa => i8080::jump::jp_cond_nn(self, memory, |e| e.sf), // JM
                0xc3 => i8080::jump::jp_cond_nn(self, memory, |_| true), // JMP
                // OUT d8
                0xd3 => {
                    let port = self.fetch_byte(memory);
                    self.mem_ptr.0 = u16::from_le_bytes([port.wrapping_add(1), self.a.0]);
                    break 'main (
                        11,
                        ExecEffect::Out {
                            port: u16::from_le_bytes([port, self.a.0]),
                            data: self.a.0,
                        },
                    );
                }
                // XTHL
                0xe3 => i8080::load::xthl(self, memory, 19),
                // DI
                0xf3 => {
                    self.iff1 = false;
                    self.iff2 = false;
                    4
                }
                // CNZ, CZ, CNC, CC, CPO, CPE, CP, CM
                0xc4 => i8080::jump::call_cond_nn(self, memory, |e| !e.zf, 17, 10), // CNZ
                0xcc => i8080::jump::call_cond_nn(self, memory, |e| e.zf, 17, 10),  // CZ
                0xd4 => i8080::jump::call_cond_nn(self, memory, |e| !e.cf, 17, 10), // CNC
                0xdc => i8080::jump::call_cond_nn(self, memory, |e| e.cf, 17, 10),  // CC
                0xe4 => i8080::jump::call_cond_nn(self, memory, |e| !e.pf, 17, 10), // CPO
                0xec => i8080::jump::call_cond_nn(self, memory, |e| e.pf, 17, 10),  // CPE
                0xf4 => i8080::jump::call_cond_nn(self, memory, |e| !e.sf, 17, 10), // CP
                0xfc => i8080::jump::call_cond_nn(self, memory, |e| e.sf, 17, 10),  // CM
                // CALL
                0xcd => i8080::jump::call_cond_nn(self, memory, |_| true, 17, 10),
                0xc5 => i8080::jump::push(self, memory, Self::get_bc), // PUSH B
                0xd5 => i8080::jump::push(self, memory, Self::get_de), // PUSH D
                0xe5 => i8080::jump::push(self, memory, Self::get_hl), // PUSH H
                0xf5 => i8080::jump::push(self, memory, Self::get_af), // PUSH PSW
                // add a, n
                0xc6 => {
                    i8080::math::alu_imm(self, |e, v| i8080::math::add_value(e, v, false), memory)
                }
                // adc a, n
                0xce => {
                    i8080::math::alu_imm(self, |e, v| i8080::math::add_value(e, v, e.cf), memory)
                }
                // sub n
                0xd6 => {
                    let value = self.fetch_byte(memory);
                    z8080::sub_value(self, value, false);
                    7
                }
                // sbc a, n
                0xde => {
                    let value = self.fetch_byte(memory);
                    let carry = self.cf;
                    z8080::sub_value(self, value, carry);
                    7
                }
                // and n
                0xe6 => {
                    let value = self.fetch_byte(memory);
                    z8080::and_value(self, value);
                    7
                }
                0xee => i8080::math::alu_imm(self, i8080::math::xor_value, memory), // xor n
                0xf6 => i8080::math::alu_imm(self, i8080::math::or_value, memory),  // or n
                // cp n
                0xfe => {
                    let value = self.fetch_byte(memory);
                    z8080::cp_value(self, value);
                    7
                }
                // RST
                0xc7 => i8080::jump::rst(self, memory, 0x00),
                0xcf => i8080::jump::rst(self, memory, 0x08),
                0xd7 => i8080::jump::rst(self, memory, 0x10),
                0xdf => i8080::jump::rst(self, memory, 0x18),
                0xe7 => i8080::jump::rst(self, memory, 0x20),
                0xef => i8080::jump::rst(self, memory, 0x28),
                0xf7 => i8080::jump::rst(self, memory, 0x30),
                0xff => i8080::jump::rst(self, memory, 0x38),
                0xe9 => i8080::jump::jp_hl(self, 4), // PCHL
                0xf9 => i8080::load::sphl(self, 6),  // spHL
                // in a (n)
                0xdb => {
                    let port = self.fetch_byte(memory) as u16;
                    self.mem_ptr.0 = self.a.0 as u16;
                    self.mem_ptr += port;
                    self.mem_ptr += 1;
                    self.input_continuation = Some(|emu, input| emu.a.0 = input);
                    break 'main (11, ExecEffect::In { port });
                }
                0xeb => i8080::load::xchg(self), // XCHG
                // EI
                0xfb => {
                    self.iff1 = true;
                    self.iff2 = true;
                    break 'main (4, ExecEffect::InterruptDelay);
                }
                // New Z80 instructions

                // ex af, af'
                0x08 => {
                    mem::swap(&mut self.a, &mut self.a_alt);
                    let flags = self.serialize_flags();
                    self.deserialize_flags(self.f_alt);
                    self.f_alt = flags;
                    4
                }
                // djnz d
                0x10 => {
                    let d = self.fetch_byte(memory);
                    self.b -= 1;
                    if self.b.0 == 0 {
                        8
                    } else {
                        self.pc += d as u16;
                        11
                    }
                }
                0x18 => self.jr(|_| true, memory),  // jr d
                0x20 => self.jr(|e| !e.zf, memory), // jr nz, d
                0x28 => self.jr(|e| e.zf, memory),  // jr z, d
                0x30 => self.jr(|e| !e.cf, memory), // jr nc, d
                0x38 => self.jr(|e| e.cf, memory),  // jr c, d
                // exx
                0xd9 => {
                    mem::swap(&mut self.b, &mut self.b_alt);
                    mem::swap(&mut self.c, &mut self.c_alt);
                    mem::swap(&mut self.d, &mut self.d_alt);
                    mem::swap(&mut self.e, &mut self.e_alt);
                    mem::swap(&mut self.h, &mut self.h_alt);
                    mem::swap(&mut self.l, &mut self.l_alt);
                    4
                }
                // CB prefix Instructions
                0xcb => {
                    let opcode = self.fetch_byte(memory);
                    bits::run_opcode(self, opcode, memory)
                }
                // DD prefix instructions
                0xdd => {
                    let opcode = self.fetch_byte(memory);
                    break 'main indexed::run_opcode::<indexed::IX>(self, opcode, memory);
                }
                // ED prefix Instructions
                0xed => {
                    let opcode = self.fetch_byte(memory);
                    break 'main misc::run_opcode(self, opcode, memory);
                }
                // FD prefix instructions
                0xfd => {
                    let opcode = self.fetch_byte(memory);
                    break 'main indexed::run_opcode::<indexed::IY>(self, opcode, memory);
                }
            };
            (clock_cycles, ExecEffect::Normal)
        };
        self.last_effect = result.1;
        result
    }
}

impl I8080FamilyEmulator for Emulator {
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
        self.sz_from_value(value);
        self.xy_from_value(value);
    }

    #[inline]
    fn overflow_flag(&mut self, overflow: bool) {
        self.pf = overflow
    }

    #[inline]
    fn parity_from_value(&mut self, value: u8) {
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

    #[cfg(feature = "std")]
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

    #[cfg(feature = "std")]
    fn dump_memory(&self, memory: &impl Memory) {
        print!("pc={:04x}h:[{:02x}h", self.pc, memory.load(self.pc.0));
        for i in 1u16..4 {
            print!(",{:02x}h", memory.load(self.pc.0.wrapping_add(i)));
        }
        print!("],sp={:04x}h:[{:02x}h]", self.sp, memory.load(self.sp.0));
        print!(",af={:02x}{:02x}h", self.a, self.serialize_flags());
        print!(
            ",bc={:04x}h:[{:02x}h]",
            self.get_bc(),
            memory.load(self.get_bc())
        );
        print!(
            ",de={:04x}h:[{:02x}h]",
            self.get_de(),
            memory.load(self.get_de())
        );
        print!(
            ",hl={:04x}h:[{:02x}h]",
            self.get_hl(),
            memory.load(self.get_hl())
        );
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

    fn input(&mut self, value: u8) {
        if let Some(cont) = self.input_continuation {
            cont(self, value);
            self.input_continuation = None;
        }
    }

    fn interrupt(&mut self, _vector: u8) {
        todo!()
    }
}
