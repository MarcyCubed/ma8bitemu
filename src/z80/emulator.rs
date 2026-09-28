//! The core of the Z80 emulator

use crate::i8080::I8080FamilyState;
use crate::memory::Memory;
use crate::z80::state::State;
use crate::z80::{bits, z8080};
use crate::{ExecEffect, Fetch, i8080};
use core::mem;

/// The Z80 emulator
#[derive(Debug, Clone)]
pub struct Emulator {
    /// Internal state of the processor
    pub state: State,
    /// The effect of the last instruction
    last_effect: ExecEffect,
    /// The interrupt vector if an interrupt was caused by external hardware
    interrupt_vector: Option<u8>,
    /// Is there a pending NMI?
    nmi_pending: bool,
}

impl Emulator {
    /// Create a Z80 emulator
    pub fn new() -> Self {
        Self {
            state: State::new(),
            last_effect: ExecEffect::Normal,
            interrupt_vector: None,
            nmi_pending: false,
        }
    }

    /// Relative jump instruction
    fn jr(&mut self, cond: fn(&State) -> bool, memory: &impl Memory) -> u8 {
        let d = self.fetch_byte(memory);
        self.state.mem_ptr = self.state.pc;
        self.state.mem_ptr += d as u16;

        if cond(&self.state) {
            self.state.pc = self.state.mem_ptr;
            12
        } else {
            7
        }
    }
}

/// Implement the instruction `sub r`
macro_rules! sub_r {
    ($emulator:ident, $register:ident) => {{
        let value = $emulator.state.$register.0;
        z8080::sub_value(&mut $emulator.state, value, false);
        4
    }};
}

/// Implement the instruction `sbc r`
macro_rules! sbc_r {
    ($emulator:ident, $register:ident) => {{
        let value = $emulator.state.$register.0;
        let carry = $emulator.state.cf;
        z8080::sub_value(&mut $emulator.state, value, carry);
        4
    }};
}

/// Implement the instruction `and r`
macro_rules! and_r {
    ($emulator:ident, $register:ident) => {{
        let value = $emulator.state.$register.0;
        z8080::and_value(&mut $emulator.state, value);
        4
    }};
}

/// Implement the instruction `cp r`
macro_rules! cp_r {
    ($emulator:ident, $register:ident) => {{
        let value = $emulator.state.$register.0;
        z8080::cp_value(&mut $emulator.state, value);
        4
    }};
}

impl Fetch for Emulator {
    fn fetch_byte(&mut self, memory: &impl Memory) -> u8 {
        self.state.fetch_byte(memory)
    }
}
impl crate::EmulatorCore for Emulator {
    fn run_opcode(&mut self, opcode: u8, memory: &mut impl Memory) -> (u8, ExecEffect) {
        // Increase the R register
        self.state.inc_r();
        // Execute the instruction
        let result = 'main: {
            let clock_cycles = match opcode {
                0x00 => 4, // nop
                // ld bc, nn
                0x01 => {
                    i8080::load::lxi(&mut self.state, memory, |state, value| state.set_bc(value))
                }
                // ld de, nn
                0x11 => {
                    i8080::load::lxi(&mut self.state, memory, |state, value| state.set_de(value))
                }
                // ld hl, nn
                0x21 => {
                    i8080::load::lxi(&mut self.state, memory, |state, value| state.set_hl(value))
                }
                // ld sp, nn
                0x31 => {
                    i8080::load::lxi(&mut self.state, memory, |state, value| state.set_sp(value))
                }
                0x02 => i8080::load::stax(&mut self.state, memory, |s| s.get_bc()), // ld (bc), a
                0x12 => i8080::load::stax(&mut self.state, memory, |s| s.get_de()), // ld (de), a
                0x22 => i8080::load::shld(&mut self.state, memory),                 // ld (nn), hl
                0x32 => i8080::load::sta(&mut self.state, memory),                  // ld (nn), a
                0x03 => i8080::math::inx(&mut self.state, State::get_bc, State::set_bc), // inc bc
                0x13 => i8080::math::inx(&mut self.state, State::get_de, State::set_de), // inc de
                0x23 => i8080::math::inx(&mut self.state, State::get_hl, State::set_hl), // inc HL
                0x33 => i8080::math::inx(&mut self.state, State::get_sp_u16, State::set_sp), // inc sp
                0x04 => i8080::math::inc(&mut self.state, |s| &mut s.b), // inc b
                0x14 => i8080::math::inc(&mut self.state, |s| &mut s.d), // inc d
                0x24 => i8080::math::inc(&mut self.state, |s| &mut s.h), // inc h
                0x0c => i8080::math::inc(&mut self.state, |s| &mut s.c), // inc c
                0x1c => i8080::math::inc(&mut self.state, |s| &mut s.e), // inc e
                0x2c => i8080::math::inc(&mut self.state, |s| &mut s.l), // inc l
                0x3c => i8080::math::inc(&mut self.state, |s| &mut s.a), // inc a
                0x34 => i8080::math::inc_mem(&mut self.state, memory),   // inc (HL)
                0x05 => z8080::dec_r(&mut self.state, |s| &mut s.b),     // dec b
                0x15 => z8080::dec_r(&mut self.state, |s| &mut s.d),     // dec d
                0x25 => z8080::dec_r(&mut self.state, |s| &mut s.h),     // dec h
                0x0d => z8080::dec_r(&mut self.state, |s| &mut s.c),     // dec c
                0x1d => z8080::dec_r(&mut self.state, |s| &mut s.e),     // dec e
                0x2d => z8080::dec_r(&mut self.state, |s| &mut s.l),     // dec l
                0x3d => z8080::dec_r(&mut self.state, |s| &mut s.a),     // dec a
                // dec (hl)
                0x35 => {
                    let address = self.state.get_hl();
                    z8080::dec_mem(&mut self.state, memory, address, 11)
                }
                0x06 => i8080::load::mvi(&mut self.state, memory, State::get_b_mut), // ld b, n
                0x16 => i8080::load::mvi(&mut self.state, memory, State::get_d_mut), // ld d, n
                0x26 => i8080::load::mvi(&mut self.state, memory, State::get_h_mut), // ld h, n
                0x0e => i8080::load::mvi(&mut self.state, memory, State::get_c_mut), // ld c, n
                0x1e => i8080::load::mvi(&mut self.state, memory, State::get_e_mut), // ld e, n
                0x2e => i8080::load::mvi(&mut self.state, memory, State::get_l_mut), // ld l, n
                0x3e => i8080::load::mvi(&mut self.state, memory, State::get_a_mut), // ld a, n
                0x36 => i8080::load::mvi_mem(&mut self.state, memory),               // ld (hl), n
                0x07 => z8080::rlca(&mut self.state),                                // rlca
                0x17 => i8080::math::ral(&mut self.state),                           // rla
                0x27 => z8080::daa(&mut self.state),                                 // daa
                0x37 => z8080::scf(&mut self.state),                                 // sfc
                // add hl, bc
                0x09 => {
                    let value = self.state.get_bc();
                    z8080::add_hl_value(&mut self.state, value);
                    7
                }
                // add hl, de
                0x19 => {
                    let value = self.state.get_de();
                    z8080::add_hl_value(&mut self.state, value);
                    7
                }
                // add hl, hl
                0x29 => {
                    let value = self.state.get_hl();
                    z8080::add_hl_value(&mut self.state, value);
                    7
                }
                // add hl, sp
                0x39 => {
                    let value = self.state.sp.0;
                    z8080::add_hl_value(&mut self.state, value);
                    7
                }
                0x0a => i8080::load::ldax(&mut self.state, memory, State::get_bc), // ld a, (bc)
                0x1a => i8080::load::ldax(&mut self.state, memory, State::get_de), // ld a, (de)
                0x2a => i8080::load::lhld(&mut self.state, memory),                // ld hl, (nn)
                0x3a => i8080::load::lda(&mut self.state, memory),                 // ld a, (nn)
                0x0b => i8080::math::dcx(&mut self.state, State::get_bc, State::set_bc), // dec bc
                0x1b => i8080::math::dcx(&mut self.state, State::get_de, State::set_de), // dec de
                0x2b => i8080::math::dcx(&mut self.state, State::get_hl, State::set_hl), // dec hl
                0x3b => i8080::math::dcx(&mut self.state, State::get_sp_u16, State::set_sp), // dec sp
                0x0f => z8080::rrca(&mut self.state),                                        // rrca
                0x1F => i8080::math::rar(&mut self.state),                                   // rra
                0x2f => z8080::cpl(&mut self.state),                                         // cpl
                0x3f => z8080::ccf(&mut self.state),                                         // ccf
                // ld x, y
                0x40 => 4, // B, B
                0x41 => i8080::load::mov(&mut self.state, State::get_b_mut, State::get_c, 4),
                0x42 => i8080::load::mov(&mut self.state, State::get_b_mut, State::get_d, 4),
                0x43 => i8080::load::mov(&mut self.state, State::get_b_mut, State::get_e, 4),
                0x44 => i8080::load::mov(&mut self.state, State::get_b_mut, State::get_h, 4),
                0x45 => i8080::load::mov(&mut self.state, State::get_b_mut, State::get_l, 4),
                0x46 => i8080::load::mov_r_mem(&mut self.state, State::get_b_mut, memory),
                0x47 => i8080::load::mov(&mut self.state, State::get_b_mut, State::get_a, 4),
                0x48 => i8080::load::mov(&mut self.state, State::get_c_mut, State::get_b, 4),
                0x49 => 4, // C, C
                0x4a => i8080::load::mov(&mut self.state, State::get_c_mut, State::get_d, 4),
                0x4b => i8080::load::mov(&mut self.state, State::get_c_mut, State::get_e, 4),
                0x4c => i8080::load::mov(&mut self.state, State::get_c_mut, State::get_h, 4),
                0x4d => i8080::load::mov(&mut self.state, State::get_c_mut, State::get_l, 4),
                0x4e => i8080::load::mov_r_mem(&mut self.state, State::get_c_mut, memory),
                0x4f => i8080::load::mov(&mut self.state, State::get_c_mut, State::get_a, 4),
                0x50 => i8080::load::mov(&mut self.state, State::get_d_mut, State::get_b, 4),
                0x51 => i8080::load::mov(&mut self.state, State::get_d_mut, State::get_c, 4),
                0x52 => 4, // D, D
                0x53 => i8080::load::mov(&mut self.state, State::get_d_mut, State::get_e, 4),
                0x54 => i8080::load::mov(&mut self.state, State::get_d_mut, State::get_h, 4),
                0x55 => i8080::load::mov(&mut self.state, State::get_d_mut, State::get_l, 4),
                0x56 => i8080::load::mov_r_mem(&mut self.state, State::get_d_mut, memory),
                0x57 => i8080::load::mov(&mut self.state, State::get_d_mut, State::get_a, 4),
                0x58 => i8080::load::mov(&mut self.state, State::get_e_mut, State::get_b, 4),
                0x59 => i8080::load::mov(&mut self.state, State::get_e_mut, State::get_c, 4),
                0x5a => i8080::load::mov(&mut self.state, State::get_e_mut, State::get_d, 4),
                0x5b => 4, // E, E
                0x5c => i8080::load::mov(&mut self.state, State::get_e_mut, State::get_h, 4),
                0x5d => i8080::load::mov(&mut self.state, State::get_e_mut, State::get_l, 4),
                0x5e => i8080::load::mov_r_mem(&mut self.state, State::get_e_mut, memory),
                0x5f => i8080::load::mov(&mut self.state, State::get_e_mut, State::get_a, 4),
                0x60 => i8080::load::mov(&mut self.state, State::get_h_mut, State::get_b, 4),
                0x61 => i8080::load::mov(&mut self.state, State::get_h_mut, State::get_c, 4),
                0x62 => i8080::load::mov(&mut self.state, State::get_h_mut, State::get_d, 4),
                0x63 => i8080::load::mov(&mut self.state, State::get_h_mut, State::get_e, 4),
                0x64 => 4, // H, H
                0x65 => i8080::load::mov(&mut self.state, State::get_h_mut, State::get_l, 4),
                0x66 => i8080::load::mov_r_mem(&mut self.state, State::get_h_mut, memory),
                0x67 => i8080::load::mov(&mut self.state, State::get_h_mut, State::get_a, 4),
                0x68 => i8080::load::mov(&mut self.state, State::get_l_mut, State::get_b, 4),
                0x69 => i8080::load::mov(&mut self.state, State::get_l_mut, State::get_c, 4),
                0x6a => i8080::load::mov(&mut self.state, State::get_l_mut, State::get_d, 4),
                0x6b => i8080::load::mov(&mut self.state, State::get_l_mut, State::get_e, 4),
                0x6c => i8080::load::mov(&mut self.state, State::get_l_mut, State::get_h, 4),
                0x6d => 4, // L, L
                0x6e => i8080::load::mov_r_mem(&mut self.state, State::get_l_mut, memory),
                0x6f => i8080::load::mov(&mut self.state, State::get_l_mut, State::get_a, 4),
                0x78 => i8080::load::mov(&mut self.state, State::get_a_mut, State::get_b, 4),
                0x79 => i8080::load::mov(&mut self.state, State::get_a_mut, State::get_c, 4),
                0x7a => i8080::load::mov(&mut self.state, State::get_a_mut, State::get_d, 4),
                0x7b => i8080::load::mov(&mut self.state, State::get_a_mut, State::get_e, 4),
                0x7c => i8080::load::mov(&mut self.state, State::get_a_mut, State::get_h, 4),
                0x7d => i8080::load::mov(&mut self.state, State::get_a_mut, State::get_l, 4),
                0x7e => i8080::load::mov_r_mem(&mut self.state, State::get_a_mut, memory),
                0x7f => 4, // A, A
                0x70 => i8080::load::mov_mem_r(&mut self.state, memory, State::get_b),
                0x71 => i8080::load::mov_mem_r(&mut self.state, memory, State::get_c),
                0x72 => i8080::load::mov_mem_r(&mut self.state, memory, State::get_d),
                0x73 => i8080::load::mov_mem_r(&mut self.state, memory, State::get_e),
                0x74 => i8080::load::mov_mem_r(&mut self.state, memory, State::get_h),
                0x75 => i8080::load::mov_mem_r(&mut self.state, memory, State::get_l),
                0x77 => i8080::load::mov_mem_r(&mut self.state, memory, State::get_a),
                // halt
                0x76 => {
                    self.state.pc -= 1;
                    break 'main (4, ExecEffect::Halt);
                }
                // ADD X
                0x80 => i8080::math::add_r(&mut self.state, State::get_b), // add a, b
                0x81 => i8080::math::add_r(&mut self.state, State::get_c), // add a, c
                0x82 => i8080::math::add_r(&mut self.state, State::get_d), // add a, d
                0x83 => i8080::math::add_r(&mut self.state, State::get_e), // add a, e
                0x84 => i8080::math::add_r(&mut self.state, State::get_h), // add a, h
                0x85 => i8080::math::add_r(&mut self.state, State::get_l), // add a, l
                0x86 => i8080::math::add_mem(&mut self.state, memory),     // add a, (hl)
                0x87 => i8080::math::add_r(&mut self.state, State::get_a), // add a, a
                // ADC
                0x88 => i8080::math::adc_r(&mut self.state, State::get_b),
                0x89 => i8080::math::adc_r(&mut self.state, State::get_c),
                0x8a => i8080::math::adc_r(&mut self.state, State::get_d),
                0x8b => i8080::math::adc_r(&mut self.state, State::get_e),
                0x8c => i8080::math::adc_r(&mut self.state, State::get_h),
                0x8d => i8080::math::adc_r(&mut self.state, State::get_l),
                0x8e => i8080::math::adc_mem(&mut self.state, memory),
                0x8f => i8080::math::adc_r(&mut self.state, State::get_a),
                0x90 => sub_r!(self, b), // sub b
                0x91 => sub_r!(self, c), // sub c
                0x92 => sub_r!(self, d), // sub d
                0x93 => sub_r!(self, e), // sub e
                0x94 => sub_r!(self, h), // sub h
                0x95 => sub_r!(self, l), // sub l
                // sub (hl)
                0x96 => {
                    let value = self.state.load_hl(memory);
                    z8080::sub_value(&mut self.state, value, false);
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
                    let value = self.state.load_hl(memory);
                    let carry = self.state.cf;
                    z8080::sub_value(&mut self.state, value, carry);
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
                    let value = self.state.load_hl(memory);
                    z8080::and_value(&mut self.state, value);
                    7
                }
                0xa7 => and_r!(self, a), // and a
                // XRA
                0xa8 => i8080::math::xra_r(&mut self.state, State::get_b),
                0xa9 => i8080::math::xra_r(&mut self.state, State::get_c),
                0xaa => i8080::math::xra_r(&mut self.state, State::get_d),
                0xab => i8080::math::xra_r(&mut self.state, State::get_e),
                0xac => i8080::math::xra_r(&mut self.state, State::get_h),
                0xad => i8080::math::xra_r(&mut self.state, State::get_l),
                0xae => i8080::math::xra_mem(&mut self.state, memory),
                0xaf => i8080::math::xra_r(&mut self.state, State::get_a),
                // ORA
                0xb0 => i8080::math::ora_r(&mut self.state, State::get_b),
                0xb1 => i8080::math::ora_r(&mut self.state, State::get_c),
                0xb2 => i8080::math::ora_r(&mut self.state, State::get_d),
                0xb3 => i8080::math::ora_r(&mut self.state, State::get_e),
                0xb4 => i8080::math::ora_r(&mut self.state, State::get_h),
                0xb5 => i8080::math::ora_r(&mut self.state, State::get_l),
                0xb6 => i8080::math::ora_mem(&mut self.state, memory),
                0xb7 => i8080::math::ora_r(&mut self.state, State::get_a),
                0xb8 => cp_r!(self, b), // cp b
                0xb9 => cp_r!(self, c), // cp c
                0xba => cp_r!(self, d), // cp d
                0xbb => cp_r!(self, e), // cp e
                0xbc => cp_r!(self, h), // cp h
                0xbd => cp_r!(self, l), // cp l
                // cp (hl)
                0xbe => {
                    let value = self.state.load_hl(memory);
                    z8080::cp_value(&mut self.state, value);
                    7
                }
                0xbf => cp_r!(self, a), // cp a
                0xc0 => i8080::jump::ret_cond(&mut self.state, memory, |s| !s.zf), // RNZ
                0xc8 => i8080::jump::ret_cond(&mut self.state, memory, |s| s.zf), // RZ
                0xd0 => i8080::jump::ret_cond(&mut self.state, memory, |s| !s.cf), // RNC
                0xd8 => i8080::jump::ret_cond(&mut self.state, memory, |s| s.cf), // RC
                0xe0 => i8080::jump::ret_cond(&mut self.state, memory, |s| !s.pf), // RPO
                0xe8 => i8080::jump::ret_cond(&mut self.state, memory, |s| s.pf), // RPE
                0xf0 => i8080::jump::ret_cond(&mut self.state, memory, |s| !s.sf), // RP
                0xf8 => i8080::jump::ret_cond(&mut self.state, memory, |s| s.sf), // RM
                0xc9 => i8080::jump::ret(&mut self.state, memory), // RET
                0xc1 => i8080::jump::pop(&mut self.state, memory, State::set_bc), // POP B
                0xd1 => i8080::jump::pop(&mut self.state, memory, State::set_de), // POP D
                0xe1 => i8080::jump::pop(&mut self.state, memory, State::set_hl), // POP E
                0xf1 => i8080::jump::pop(&mut self.state, memory, State::set_af), // POP PSW
                0xc2 => i8080::jump::jp_cond_nn(&mut self.state, memory, |s| !s.zf), // JNZ
                0xca => i8080::jump::jp_cond_nn(&mut self.state, memory, |s| s.zf), // JZ
                0xd2 => i8080::jump::jp_cond_nn(&mut self.state, memory, |s| !s.cf), // JNC
                0xda => i8080::jump::jp_cond_nn(&mut self.state, memory, |s| s.cf), // JC
                0xe2 => i8080::jump::jp_cond_nn(&mut self.state, memory, |s| !s.pf), // JPO
                0xea => i8080::jump::jp_cond_nn(&mut self.state, memory, |s| s.pf), // JPE
                0xf2 => i8080::jump::jp_cond_nn(&mut self.state, memory, |s| !s.sf), // JP
                0xfa => i8080::jump::jp_cond_nn(&mut self.state, memory, |s| s.sf), // JM
                0xc3 => i8080::jump::jp_cond_nn(&mut self.state, memory, |_| true), // JMP
                // OUT d8
                0xd3 => {
                    let port = self.fetch_byte(memory);
                    break 'main (
                        11,
                        ExecEffect::Out {
                            port,
                            data: self.state.a.0,
                        },
                    );
                }
                // XTHL
                0xe3 => i8080::load::xthl(&mut self.state, memory, 19),
                // DI
                0xf3 => {
                    self.state.iff1 = false;
                    self.state.iff2 = false;
                    4
                }
                // CNZ, CZ, CNC, CC, CPO, CPE, CP, CM
                0xc4 => i8080::jump::call_cond_nn(&mut self.state, memory, |s| !s.zf, 17, 10), // CNZ
                0xcc => i8080::jump::call_cond_nn(&mut self.state, memory, |s| s.zf, 17, 10),  // CZ
                0xd4 => i8080::jump::call_cond_nn(&mut self.state, memory, |s| !s.cf, 17, 10), // CNC
                0xdc => i8080::jump::call_cond_nn(&mut self.state, memory, |s| s.cf, 17, 10),  // CC
                0xe4 => i8080::jump::call_cond_nn(&mut self.state, memory, |s| !s.pf, 17, 10), // CPO
                0xec => i8080::jump::call_cond_nn(&mut self.state, memory, |s| s.pf, 17, 10), // CPE
                0xf4 => i8080::jump::call_cond_nn(&mut self.state, memory, |s| !s.sf, 17, 10), // CP
                0xfc => i8080::jump::call_cond_nn(&mut self.state, memory, |s| s.sf, 17, 10), // CM
                // CALL
                0xcd => i8080::jump::call_cond_nn(&mut self.state, memory, |_| true, 17, 10),
                0xc5 => i8080::jump::push(&mut self.state, memory, State::get_bc), // PUSH B
                0xd5 => i8080::jump::push(&mut self.state, memory, State::get_de), // PUSH D
                0xe5 => i8080::jump::push(&mut self.state, memory, State::get_hl), // PUSH H
                0xf5 => i8080::jump::push(&mut self.state, memory, State::get_af), // PUSH PSW
                // add a, n
                0xc6 => i8080::math::alu_imm(
                    &mut self.state,
                    |s, v| i8080::math::add_value(s, v, false),
                    memory,
                ),
                // adc a, n
                0xce => i8080::math::alu_imm(
                    &mut self.state,
                    |s, v| i8080::math::add_value(s, v, s.cf),
                    memory,
                ),
                // sub n
                0xd6 => {
                    let value = self.fetch_byte(memory);
                    z8080::sub_value(&mut self.state, value, false);
                    7
                }
                // sbc a, n
                0xde => {
                    let value = self.fetch_byte(memory);
                    let carry = self.state.cf;
                    z8080::sub_value(&mut self.state, value, carry);
                    7
                }
                // and n
                0xe6 => {
                    let value = self.fetch_byte(memory);
                    z8080::and_value(&mut self.state, value);
                    7
                }
                0xee => i8080::math::alu_imm(&mut self.state, i8080::math::xor_value, memory), // xor n
                0xf6 => i8080::math::alu_imm(&mut self.state, i8080::math::or_value, memory), // or n
                // cp n
                0xfe => {
                    let value = self.fetch_byte(memory);
                    z8080::cp_value(&mut self.state, value);
                    7
                }
                // RST
                0xc7 => i8080::jump::rst(&mut self.state, memory, 0x00),
                0xcf => i8080::jump::rst(&mut self.state, memory, 0x08),
                0xd7 => i8080::jump::rst(&mut self.state, memory, 0x10),
                0xdf => i8080::jump::rst(&mut self.state, memory, 0x18),
                0xe7 => i8080::jump::rst(&mut self.state, memory, 0x20),
                0xef => i8080::jump::rst(&mut self.state, memory, 0x28),
                0xf7 => i8080::jump::rst(&mut self.state, memory, 0x30),
                0xff => i8080::jump::rst(&mut self.state, memory, 0x38),
                0xe9 => i8080::jump::jp_hl(&mut self.state, 4), // PCHL
                0xf9 => i8080::load::sphl(&mut self.state, 6),  // spHL
                // IN d8
                0xdb => {
                    let port = self.fetch_byte(memory);
                    break 'main (11, ExecEffect::In { port });
                }
                0xeb => i8080::load::xchg(&mut self.state), // XCHG
                // EI
                0xfb => {
                    self.state.iff1 = true;
                    self.state.iff2 = true;
                    break 'main (4, ExecEffect::InterruptDelay);
                }
                // New Z80 instructions

                // ex af, af'
                0x08 => {
                    mem::swap(&mut self.state.a, &mut self.state.a_alt);
                    let flags = self.state.serialize_flags();
                    self.state.deserialize_flags(self.state.f_alt);
                    self.state.f_alt = flags;
                    4
                }
                // djnz d
                0x10 => {
                    let d = self.fetch_byte(memory);
                    self.state.b -= 1;
                    if self.state.b.0 == 0 {
                        8
                    } else {
                        self.state.pc += d as u16;
                        11
                    }
                }
                0x18 => self.jr(|_| true, memory),  // jr d
                0x20 => self.jr(|s| !s.zf, memory), // jr nz, d
                0x28 => self.jr(|s| s.zf, memory),  // jr z, d
                0x30 => self.jr(|s| !s.cf, memory), // jr nc, d
                0x38 => self.jr(|s| s.cf, memory),  // jr c, d
                // exx
                0xd9 => {
                    mem::swap(&mut self.state.b, &mut self.state.b_alt);
                    mem::swap(&mut self.state.c, &mut self.state.c_alt);
                    mem::swap(&mut self.state.d, &mut self.state.d_alt);
                    mem::swap(&mut self.state.e, &mut self.state.e_alt);
                    mem::swap(&mut self.state.h, &mut self.state.h_alt);
                    mem::swap(&mut self.state.l, &mut self.state.l_alt);
                    4
                }
                0xcb => {
                    let opcode = self.fetch_byte(memory);
                    bits::run_opcode(&mut self.state, opcode, memory)
                }
                0xdd => todo!("IX instructions"),
                0xed => todo!("Misc instructions"),
                0xfd => todo!("IY instructions"),
            };
            (clock_cycles, ExecEffect::Normal)
        };
        self.last_effect = result.1;
        result
    }
}
