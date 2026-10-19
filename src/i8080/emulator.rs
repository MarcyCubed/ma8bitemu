//! The core of the emulator

use crate::ExecEffect;
use crate::i8080::{Fetch, I8080FamilyEmulator, jump, load, math};
use crate::memory::Memory;
use core::num::Wrapping;

/// The 8080 emulator
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Emulator {
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
    /// The effect of the last instruction
    last_effect: ExecEffect,
    /// The interrupt vector if an interrupt was caused by external hardware
    interrupt_vector: Option<u8>,
}

impl Emulator {
    /// Create an 8080 emulator
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
            last_effect: ExecEffect::Normal,
            interrupt_vector: None,
        }
    }
}

impl crate::EmulatorCore for Emulator {
    /// Get the next instruction
    fn next_instruction(&mut self, memory: &mut impl Memory) -> u8 {
        // If we can and should trigger an interrupt...
        if let Some(vector) = self.interrupt_vector
            && self.inte
            && self.last_effect != ExecEffect::InterruptDelay
        {
            // Run the instruction
            self.interrupt_vector = None;
            vector
        } else {
            self.fetch_byte(memory)
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    /// Execute a single opcode.
    ///
    /// Return the number of clock cycles it took to execute the instruction and the result of the execution
    fn run_opcode(&mut self, opcode: u8, memory: &mut impl Memory) -> (u8, ExecEffect) {
        let result = 'main: {
            let clock_cycles = match opcode {
                // NOP
                0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => 4,
                // LXI B, d16
                0x01 => load::lxi(self, memory, |state, value| state.set_bc(value)),
                // LXI D, d16
                0x11 => load::lxi(self, memory, |state, value| state.set_de(value)),
                // LXI H, d16
                0x21 => load::lxi(self, memory, |state, value| state.set_hl(value)),
                // LXI SP, d16
                0x31 => load::lxi(self, memory, |state, value| state.set_sp(value)),
                // STAX B
                0x02 => load::stax(self, memory, |s| s.get_bc()),
                // STAX D
                0x12 => load::stax(self, memory, |s| s.get_de()),
                // SHLD a16
                0x22 => load::shld_rr(self, memory, Self::get_hl),
                // STA a16
                0x32 => load::sta(self, memory),
                // INX B
                0x03 => math::inx(self, Self::get_bc, Self::set_bc, 5),
                // INX D
                0x13 => math::inx(self, Self::get_de, Self::set_de, 5),
                // INX H
                0x23 => math::inx(self, Self::get_hl, Self::set_hl, 5),
                // INX SP
                0x33 => math::inx(self, Self::get_sp_u16, Self::set_sp, 5),
                // INR B
                0x04 => math::inc(self, |s| &mut s.b, 5),
                // INR D
                0x14 => math::inc(self, |s| &mut s.d, 5),
                // INR H
                0x24 => math::inc(self, |s| &mut s.h, 5),
                // INR C
                0x0c => math::inc(self, |s| &mut s.c, 5),
                // INR E
                0x1c => math::inc(self, |s| &mut s.e, 5),
                // INR L
                0x2c => math::inc(self, |s| &mut s.l, 5),
                // INR A
                0x3c => math::inc(self, |s| &mut s.a, 5),
                // INR M
                0x34 => math::inc_mem(self, memory, 10),
                // DCR B
                0x05 => math::dec(self, |s| &mut s.b),
                // DCR D
                0x15 => math::dec(self, |s| &mut s.d),
                // DCR H
                0x25 => math::dec(self, |s| &mut s.h),
                // DCR C
                0x0d => math::dec(self, |s| &mut s.c),
                // DCR E
                0x1d => math::dec(self, |s| &mut s.e),
                // DCR L
                0x2d => math::dec(self, |s| &mut s.l),
                // DCR A
                0x3d => math::dec(self, |s| &mut s.a),
                // DCR M
                0x35 => math::dec_mem(self, memory),
                // MVI B, d8
                0x06 => load::mvi(self, memory, Self::get_b_mut),
                // MVI D, d8
                0x16 => load::mvi(self, memory, Self::get_d_mut),
                // MVI H, d8
                0x26 => load::mvi(self, memory, Self::get_h_mut),
                // MVI C, d8
                0x0e => load::mvi(self, memory, Self::get_c_mut),
                // MVI E, d8
                0x1e => load::mvi(self, memory, Self::get_e_mut),
                // MVI L, d8
                0x2e => load::mvi(self, memory, Self::get_l_mut),
                // MVI A, d8
                0x3e => load::mvi(self, memory, Self::get_a_mut),
                // MVI M
                0x36 => load::mvi_mem(self, memory),
                // RLC
                0x07 => math::rlc(self),
                // RAL
                0x17 => math::ral(self),
                // DAA
                0x27 => math::daa(self),
                // STC
                0x37 => math::stc(self),
                // DAD B
                0x09 => math::dad(self, Self::get_bc),
                // DAD D
                0x19 => math::dad(self, Self::get_de),
                // DAD H
                0x29 => math::dad(self, Self::get_hl),
                // DAD SP
                0x39 => math::dad(self, Self::get_sp_u16),
                // LDAX B
                0x0a => load::ldax(self, memory, Self::get_bc),
                // LDAX D
                0x1a => load::ldax(self, memory, Self::get_de),
                // LHLD a16
                0x2a => load::lhld_rr(self, memory, Self::set_hl),
                // LDA a16
                0x3a => load::lda(self, memory),
                // DCX B
                0x0b => math::dcx(self, Self::get_bc, Self::set_bc, 5),
                // DCX D
                0x1b => math::dcx(self, Self::get_de, Self::set_de, 5),
                // DCX H
                0x2b => math::dcx(self, Self::get_hl, Self::set_hl, 5),
                // DCX SP
                0x3b => math::dcx(self, Self::get_sp_u16, Self::set_sp, 5),
                // RRC
                0x0f => math::rrc(self),
                // RAR
                0x1F => math::rar(self),
                // CMA
                0x2f => math::cma(self),
                // CMC
                0x3f => math::cmc(self),
                // MOV X, X
                0x40 => 5, // B, B
                0x41 => load::mov(self, Self::get_b_mut, Self::get_c, 5),
                0x42 => load::mov(self, Self::get_b_mut, Self::get_d, 5),
                0x43 => load::mov(self, Self::get_b_mut, Self::get_e, 5),
                0x44 => load::mov(self, Self::get_b_mut, Self::get_h, 5),
                0x45 => load::mov(self, Self::get_b_mut, Self::get_l, 5),
                0x46 => load::mov_r_mem(self, Self::get_b_mut, memory),
                0x47 => load::mov(self, Self::get_b_mut, Self::get_a, 5),

                0x48 => load::mov(self, Self::get_c_mut, Self::get_b, 5),
                0x49 => 5, // C, C
                0x4a => load::mov(self, Self::get_c_mut, Self::get_d, 5),
                0x4b => load::mov(self, Self::get_c_mut, Self::get_e, 5),
                0x4c => load::mov(self, Self::get_c_mut, Self::get_h, 5),
                0x4d => load::mov(self, Self::get_c_mut, Self::get_l, 5),
                0x4e => load::mov_r_mem(self, Self::get_c_mut, memory),
                0x4f => load::mov(self, Self::get_c_mut, Self::get_a, 5),

                0x50 => load::mov(self, Self::get_d_mut, Self::get_b, 5),
                0x51 => load::mov(self, Self::get_d_mut, Self::get_c, 5),
                0x52 => 5, // D, D
                0x53 => load::mov(self, Self::get_d_mut, Self::get_e, 5),
                0x54 => load::mov(self, Self::get_d_mut, Self::get_h, 5),
                0x55 => load::mov(self, Self::get_d_mut, Self::get_l, 5),
                0x56 => load::mov_r_mem(self, Self::get_d_mut, memory),
                0x57 => load::mov(self, Self::get_d_mut, Self::get_a, 5),

                0x58 => load::mov(self, Self::get_e_mut, Self::get_b, 5),
                0x59 => load::mov(self, Self::get_e_mut, Self::get_c, 5),
                0x5a => load::mov(self, Self::get_e_mut, Self::get_d, 5),
                0x5b => 5, // E, E
                0x5c => load::mov(self, Self::get_e_mut, Self::get_h, 5),
                0x5d => load::mov(self, Self::get_e_mut, Self::get_l, 5),
                0x5e => load::mov_r_mem(self, Self::get_e_mut, memory),
                0x5f => load::mov(self, Self::get_e_mut, Self::get_a, 5),

                0x60 => load::mov(self, Self::get_h_mut, Self::get_b, 5),
                0x61 => load::mov(self, Self::get_h_mut, Self::get_c, 5),
                0x62 => load::mov(self, Self::get_h_mut, Self::get_d, 5),
                0x63 => load::mov(self, Self::get_h_mut, Self::get_e, 5),
                0x64 => 5, // H, H
                0x65 => load::mov(self, Self::get_h_mut, Self::get_l, 5),
                0x66 => load::mov_r_mem(self, Self::get_h_mut, memory),
                0x67 => load::mov(self, Self::get_h_mut, Self::get_a, 5),

                0x68 => load::mov(self, Self::get_l_mut, Self::get_b, 5),
                0x69 => load::mov(self, Self::get_l_mut, Self::get_c, 5),
                0x6a => load::mov(self, Self::get_l_mut, Self::get_d, 5),
                0x6b => load::mov(self, Self::get_l_mut, Self::get_e, 5),
                0x6c => load::mov(self, Self::get_l_mut, Self::get_h, 5),
                0x6d => 5, // L, L
                0x6e => load::mov_r_mem(self, Self::get_l_mut, memory),
                0x6f => load::mov(self, Self::get_l_mut, Self::get_a, 5),

                0x78 => load::mov(self, Self::get_a_mut, Self::get_b, 5),
                0x79 => load::mov(self, Self::get_a_mut, Self::get_c, 5),
                0x7a => load::mov(self, Self::get_a_mut, Self::get_d, 5),
                0x7b => load::mov(self, Self::get_a_mut, Self::get_e, 5),
                0x7c => load::mov(self, Self::get_a_mut, Self::get_h, 5),
                0x7d => load::mov(self, Self::get_a_mut, Self::get_l, 5),
                0x7e => load::mov_r_mem(self, Self::get_a_mut, memory),
                0x7f => 5, // A, A

                0x70 => load::mov_mem_r(self, memory, Self::get_b),
                0x71 => load::mov_mem_r(self, memory, Self::get_c),
                0x72 => load::mov_mem_r(self, memory, Self::get_d),
                0x73 => load::mov_mem_r(self, memory, Self::get_e),
                0x74 => load::mov_mem_r(self, memory, Self::get_h),
                0x75 => load::mov_mem_r(self, memory, Self::get_l),
                0x77 => load::mov_mem_r(self, memory, Self::get_a),
                // HLT
                0x76 => {
                    self.pc -= 1;
                    break 'main (7, ExecEffect::Halt);
                }
                // ADD X
                0x80 => math::add_r(self, Self::get_b),
                0x81 => math::add_r(self, Self::get_c),
                0x82 => math::add_r(self, Self::get_d),
                0x83 => math::add_r(self, Self::get_e),
                0x84 => math::add_r(self, Self::get_h),
                0x85 => math::add_r(self, Self::get_l),
                0x86 => math::add_mem(self, memory),
                0x87 => math::add_r(self, Self::get_a),
                // ADC
                0x88 => math::adc_r(self, Self::get_b),
                0x89 => math::adc_r(self, Self::get_c),
                0x8a => math::adc_r(self, Self::get_d),
                0x8b => math::adc_r(self, Self::get_e),
                0x8c => math::adc_r(self, Self::get_h),
                0x8d => math::adc_r(self, Self::get_l),
                0x8e => math::adc_mem(self, memory),
                0x8f => math::adc_r(self, Self::get_a),
                // SUB
                0x90 => math::sub_r(self, Self::get_b),
                0x91 => math::sub_r(self, Self::get_c),
                0x92 => math::sub_r(self, Self::get_d),
                0x93 => math::sub_r(self, Self::get_e),
                0x94 => math::sub_r(self, Self::get_h),
                0x95 => math::sub_r(self, Self::get_l),
                0x96 => math::sub_mem(self, memory),
                0x97 => math::sub_r(self, Self::get_a),
                // SBB
                0x98 => math::sbb_r(self, Self::get_b),
                0x99 => math::sbb_r(self, Self::get_c),
                0x9a => math::sbb_r(self, Self::get_d),
                0x9b => math::sbb_r(self, Self::get_e),
                0x9c => math::sbb_r(self, Self::get_h),
                0x9d => math::sbb_r(self, Self::get_l),
                0x9e => math::sbb_mem(self, memory),
                0x9f => math::sbb_r(self, Self::get_a),
                // ANA
                0xa0 => math::ana_r(self, Self::get_b),
                0xa1 => math::ana_r(self, Self::get_c),
                0xa2 => math::ana_r(self, Self::get_d),
                0xa3 => math::ana_r(self, Self::get_e),
                0xa4 => math::ana_r(self, Self::get_h),
                0xa5 => math::ana_r(self, Self::get_l),
                0xa6 => math::ana_mem(self, memory),
                0xa7 => math::ana_r(self, Self::get_a),
                // XRA
                0xa8 => math::xra_r(self, Self::get_b),
                0xa9 => math::xra_r(self, Self::get_c),
                0xaa => math::xra_r(self, Self::get_d),
                0xab => math::xra_r(self, Self::get_e),
                0xac => math::xra_r(self, Self::get_h),
                0xad => math::xra_r(self, Self::get_l),
                0xae => math::xra_mem(self, memory),
                0xaf => math::xra_r(self, Self::get_a),
                // ORA
                0xb0 => math::ora_r(self, Self::get_b),
                0xb1 => math::ora_r(self, Self::get_c),
                0xb2 => math::ora_r(self, Self::get_d),
                0xb3 => math::ora_r(self, Self::get_e),
                0xb4 => math::ora_r(self, Self::get_h),
                0xb5 => math::ora_r(self, Self::get_l),
                0xb6 => math::ora_mem(self, memory),
                0xb7 => math::ora_r(self, Self::get_a),
                // CMP
                0xb8 => math::cmp_r(self, Self::get_b),
                0xb9 => math::cmp_r(self, Self::get_c),
                0xba => math::cmp_r(self, Self::get_d),
                0xbb => math::cmp_r(self, Self::get_e),
                0xbc => math::cmp_r(self, Self::get_h),
                0xbd => math::cmp_r(self, Self::get_l),
                0xbe => math::cmp_mem(self, memory),
                0xbf => math::cmp_r(self, Self::get_a),
                0xc0 => jump::ret_cond(self, memory, |s| !s.zf), // RNZ
                0xc8 => jump::ret_cond(self, memory, |s| s.zf),  // RZ
                0xd0 => jump::ret_cond(self, memory, |s| !s.cf), // RNC
                0xd8 => jump::ret_cond(self, memory, |s| s.cf),  // RC
                0xe0 => jump::ret_cond(self, memory, |s| !s.pf), // RPO
                0xe8 => jump::ret_cond(self, memory, |s| s.pf),  // RPE
                0xf0 => jump::ret_cond(self, memory, |s| !s.sf), // RP
                0xf8 => jump::ret_cond(self, memory, |s| s.sf),  // RM
                0xc9 | 0xd9 => jump::ret(self, memory),          // RET
                0xc1 => jump::pop(self, memory, Self::set_bc),   // POP B
                0xd1 => jump::pop(self, memory, Self::set_de),   // POP D
                0xe1 => jump::pop(self, memory, Self::set_hl),   // POP E
                0xf1 => jump::pop(self, memory, Self::set_af),   // POP PSW
                0xc2 => jump::jp_cond_nn(self, memory, |s| !s.zf), // JNZ
                0xca => jump::jp_cond_nn(self, memory, |s| s.zf), // JZ
                0xd2 => jump::jp_cond_nn(self, memory, |s| !s.cf), // JNC
                0xda => jump::jp_cond_nn(self, memory, |s| s.cf), // JC
                0xe2 => jump::jp_cond_nn(self, memory, |s| !s.pf), // JPO
                0xea => jump::jp_cond_nn(self, memory, |s| s.pf), // JPE
                0xf2 => jump::jp_cond_nn(self, memory, |s| !s.sf), // JP
                0xfa => jump::jp_cond_nn(self, memory, |s| s.sf), // JM
                0xc3 | 0xcb => jump::jp_cond_nn(self, memory, |_| true), // JMP
                // OUT d8
                0xd3 => {
                    let port = self.fetch_byte(memory) as u16;
                    break 'main (
                        10,
                        ExecEffect::Out {
                            port,
                            data: self.a.0,
                        },
                    );
                }
                // XTHL
                0xe3 => load::xthl(self, memory, 18),
                // DI
                0xf3 => {
                    self.inte = false;
                    4
                }
                // CNZ, CZ, CNC, CC, CPO, CPE, CP, CM
                0xc4 => jump::call_cond_nn(self, memory, |s| !s.zf, 17, 11), // CNZ
                0xcc => jump::call_cond_nn(self, memory, |s| s.zf, 17, 11),  // CZ
                0xd4 => jump::call_cond_nn(self, memory, |s| !s.cf, 17, 11), // CNC
                0xdc => jump::call_cond_nn(self, memory, |s| s.cf, 17, 11),  // CC
                0xe4 => jump::call_cond_nn(self, memory, |s| !s.pf, 17, 11), // CPO
                0xec => jump::call_cond_nn(self, memory, |s| s.pf, 17, 11),  // CPE
                0xf4 => jump::call_cond_nn(self, memory, |s| !s.sf, 17, 11), // CP
                0xfc => jump::call_cond_nn(self, memory, |s| s.sf, 17, 11),  // CM
                // CALL
                0xcd | 0xdd | 0xed | 0xfd => jump::call_cond_nn(self, memory, |_| true, 17, 11),
                0xc5 => jump::push(self, memory, Self::get_bc), // PUSH B
                0xd5 => jump::push(self, memory, Self::get_de), // PUSH D
                0xe5 => jump::push(self, memory, Self::get_hl), // PUSH H
                0xf5 => jump::push(self, memory, Self::get_af), // PUSH PSW
                0xc6 => math::alu_imm(self, |s, v| math::add_value(s, v, false), memory), // ADI
                0xce => math::alu_imm(self, |s, v| math::add_value(s, v, s.cf), memory), // ACI
                0xd6 => math::alu_imm(self, |s, v| math::sub_value(s, v, false), memory), // SUI
                0xde => math::alu_imm(self, |s, v| math::sub_value(s, v, s.cf), memory), // SBI
                0xe6 => math::alu_imm(self, math::and_value, memory), // ANI
                0xee => math::alu_imm(self, math::xor_value, memory), // XRI
                0xf6 => math::alu_imm(self, math::or_value, memory), // ORI
                0xfe => math::alu_imm(self, math::cmp_value, memory), // CPI
                // RST
                0xc7 => jump::rst(self, memory, 0x00),
                0xcf => jump::rst(self, memory, 0x08),
                0xd7 => jump::rst(self, memory, 0x10),
                0xdf => jump::rst(self, memory, 0x18),
                0xe7 => jump::rst(self, memory, 0x20),
                0xef => jump::rst(self, memory, 0x28),
                0xf7 => jump::rst(self, memory, 0x30),
                0xff => jump::rst(self, memory, 0x38),
                0xe9 => jump::jp_hl(self, 5), // PCHL
                0xf9 => load::sphl(self, 5),  // SPHL
                // IN d8
                0xdb => {
                    let port = self.fetch_byte(memory) as u16;
                    break 'main (10, ExecEffect::In { port });
                }
                0xeb => load::xchg(self), // XCHG
                // EI
                0xfb => {
                    self.inte = true;
                    break 'main (4, ExecEffect::InterruptDelay);
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
        self.zf = value == 0;
        self.sf = value & (1 << 7) != 0;
        self.pf = value.count_ones() & 1 == 0;
    }

    #[inline]
    fn overflow_flag(&mut self, _overflow: bool) {
        // No overflow in 8080
    }

    #[inline]
    fn parity_from_value(&mut self, _value: u8) {
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

    /// Write the state to the screen.
    ///
    /// Also shows the opcode if it's known.
    #[cfg(feature = "std")]
    fn dump(&self, opcode: u8) {
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

    #[cfg(feature = "std")]
    fn dump_memory(&self, memory: &impl Memory) {
        print!("pc={:04x}h:[{:02x}h", self.pc, memory.load(self.pc.0));
        for i in 1u16..4 {
            print!(",{:02x}h", memory.load(self.pc.0.wrapping_add(i)));
        }
        print!("],sp={:04x}h:[{:02x}h]", self.sp, memory.load(self.sp.0));
        print!(",af={:02x}{:02x}h", self.a, self.serialize_flags());
        print!(",bc={:04x}h", self.get_bc());
        print!(",de={:04x}h", self.get_de());
        print!(
            ",hl={:04x}h:[{:02x}h]",
            self.get_hl(),
            memory.load(self.get_hl())
        );
        print!(",cf={}", self.cf as u8);
        print!(",pf={}", self.pf as u8);
        print!(",af={}", self.af as u8);
        print!(",zf={}", self.zf as u8);
        print!(",sf={}", self.sf as u8);
        print!(",iff={}", self.inte as u8);

        println!();
    }

    fn set_memptr(&mut self, _address: u16) {
        // Nothing to do
    }

    fn input(&mut self, _memory: &mut impl Memory, value: u8) {
        self.a.0 = value
    }

    fn interrupt(&mut self, vector: u8) -> u8 {
        self.interrupt_vector = Some(vector);
        self.inte = false;
        11 // I couldn't find the actual time in the 8080 documentation, so I used Z80's
    }
}

impl Emulator {
    /// Carry flag bit
    pub const C_FLAG_BIT: u32 = 0;

    /// Parity flag bit
    pub const P_FLAG_BIT: u32 = 2;

    /// Zero flag bit
    pub const Z_FLAG_BIT: u32 = 6;

    /// Sign flag bit
    pub const S_FLAG_BIT: u32 = 7;

    /// Auxiliary carry flag bit
    pub const A_FLAG_BIT: u32 = 4;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i8080;

    #[test]
    fn check_if_interrupts_trigger_when_they_should() {
        i8080::tests::test_interrupts(Emulator::new())
    }

    #[test]
    fn check_running_limit() {
        i8080::tests::test_run_limited_clock(Emulator::new());
    }

    #[test]
    fn check_run_n_instructions() {
        i8080::tests::test_run_n_instructions(Emulator::new())
    }

    /// Check if the IN instruction is working properly
    #[test]
    fn input() {
        i8080::tests::test_input(Emulator::new, 10);
    }

    /// Check if the OUT instruction is working properly
    #[test]
    fn output() {
        i8080::tests::test_output(Emulator::new, 10);
    }
}
