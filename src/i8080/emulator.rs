//! The core of the emulator

use crate::ExecEffect;
use crate::i8080::{Fetch, I8080FamilyEmulator, jump, load, math};
use crate::memory::Memory;
use std::num::Wrapping;

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
    fn next_instruction(&mut self, memory: &impl Memory) -> u8 {
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
                0x03 => math::inx(self, Self::get_bc, Self::set_bc),
                // INX D
                0x13 => math::inx(self, Self::get_de, Self::set_de),
                // INX H
                0x23 => math::inx(self, Self::get_hl, Self::set_hl),
                // INX SP
                0x33 => math::inx(self, Self::get_sp_u16, Self::set_sp),
                // INR B
                0x04 => math::inc(self, |s| &mut s.b),
                // INR D
                0x14 => math::inc(self, |s| &mut s.d),
                // INR H
                0x24 => math::inc(self, |s| &mut s.h),
                // INR C
                0x0c => math::inc(self, |s| &mut s.c),
                // INR E
                0x1c => math::inc(self, |s| &mut s.e),
                // INR L
                0x2c => math::inc(self, |s| &mut s.l),
                // INR A
                0x3c => math::inc(self, |s| &mut s.a),
                // INR M
                0x34 => math::inc_mem(self, memory),
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
                0x0b => math::dcx(self, Self::get_bc, Self::set_bc),
                // DCX D
                0x1b => math::dcx(self, Self::get_de, Self::set_de),
                // DCX H
                0x2b => math::dcx(self, Self::get_hl, Self::set_hl),
                // DCX SP
                0x3b => math::dcx(self, Self::get_sp_u16, Self::set_sp),
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

    fn input(&mut self, value: u8) {
        self.a.0 = value
    }

    fn interrupt(&mut self, vector: u8) {
        self.interrupt_vector = Some(vector);
        self.inte = false;
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
    use crate::EmulatorCore;
    use core::assert_matches;

    /// Make a program to test interrupts
    ///
    ///  We can track where we are in the program
    fn make_program() -> [u8; 256] {
        [
            // This originally was written for a Z80 emulator, that's why it has Z80 assembly
            0xd3, 0x01, // 00h: out (1), a
            // Put SP within the memory
            0x31, 0xff, 0x00, // 02h: ld sp, 0xff
            0xfb, // 05h: ei
            0xd3, 0x02, // 06h: out (2), a
            0xd3, 0x03, // 08h: out (3), a
            // Block with interrupts disabled
            0xf3, // 0ah: di
            0xd3, 0x04, // 0bh: out (4), a
            0xfb, // 0dh: ei
            // An out immediately after ei
            0xd3, 0x05, // 0eh: out (5), a
            0xd3, 0x06, // 10h: out (6), a
            0x76, // 12h: halt
            0x00, 0x00, 0x00, 0x00, 0x00, // 13h: nop *  5
            // an interruption handler at 18h
            0xf3, // 18h: di
            0xd3, 0x07, // 19h: out (7), a
            0xfb, // 1bh: ei
            0xc9, // 1ch: ret
            0xd3, 0x06, // 1dh: out (6), a
            0x00, //nop
            // an interruption handler with 2 outs at 20h
            0xf3, // 20h: di
            0xd3, 0x20, // 21h: out (20h), a
            0xd3, 0x21, // 23h: out (21h), a
            0xfb, // 25h: ei
            0xc9, // 26h: ret
            0x00, // nop
            // A different interruption handler at 28h
            0xf3, // 28h: di
            0x00, // 29h: nop
            0xd3, 0x28, // 2ah: out (0x28), a
            0x00, // 2ch: nop
            0xfb, // 2dh: ei
            0xc9, // 2eh: ret
            0x00, // nop
            // Filler
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 30h: nop *  8
            // rst 38h
            0xf3, // 38h: di
            0x00, // 39h: nop
            0xd3, 0x38, // 3ah: out (38h), a
            0x00, // 3ch nop
            0xfb, // 3dh: ei
            0xc9, // 3eh: ret
            0x00, // nop
            // Filler
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 40h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 48h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 50h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 58h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 60h: nop * 6
            // NMI handler
            0xd3, 0x66, // 66h: out (66h), a
            0xed, 0x45, // 68h: retn
            // Should be unreachable
            0xd3, 0xbb, // 6ah: out (bbh), a
            0x76, // 6ch: halt
            0x00, 0x00, 0x00, // 6dh: nop
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 70h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 78h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 80h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // a0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // a8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // b0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // b8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // c0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // c8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // d0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // d8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // e0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // e8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f8h: nop *  8
        ]
    }

    macro_rules! assert_port {
        ($number:literal, $result:expr, $text: literal) => {
            assert_matches!($result, ExecEffect::Out { .. });
            if let ExecEffect::Out { port, .. } = $result {
                assert_eq!($number, port & 0xff, $text);
            }
        };
    }

    /// Run until we execute an Out or reach the limit
    fn run_until_out(emulator: &mut Emulator, program: &mut impl Memory, limit: u32) -> ExecEffect {
        let mut limit = limit;
        loop {
            let effect = emulator.run(program).1;
            if let ExecEffect::Out { .. } = effect {
                break effect;
            } else if limit == 0 {
                break ExecEffect::Normal;
            } else {
                limit -= 1;
            }
        }
    }

    #[test]
    fn check_if_interrupts_trigger_when_they_should() {
        let mut emulator = Emulator::new();
        let mut program = make_program();
        // Request an interrupt that runs rst 18h
        emulator.interrupt(0xdf);
        // Should run into the first out before the ei
        let (_, effect) = emulator.run(&mut program);
        assert_port!(1, effect, "Interrupt handled while interrupts are disabled");
        // Set up the stack
        emulator.step(&mut program);
        // Run ei. The next instruction should be still not interrupted
        let (_, effect) = emulator.step(&mut program);
        assert_matches!(effect, ExecEffect::InterruptDelay);
        // Should run into the out after the ei
        let (_, effect) = emulator.run(&mut program);
        assert_port!(
            2,
            effect,
            "ei didn't block interruptions for the next instruction"
        );
        // Should do a rst 18h to the interrupt handler and hit the OUT there
        let (_, effect) = emulator.run(&mut program);
        assert_port!(7, effect, "Didn't start handling interruption");
        // New interrupt with rst 20h
        emulator.interrupt(0xe7);
        // Should hit EI at the end of the handler
        let (_, effect) = emulator.step(&mut program);
        assert_matches!(effect, ExecEffect::InterruptDelay);
        // Should run the return and jump to where we were before
        let _ = emulator.step(&mut program);
        assert_eq!(emulator.pc.0, 0x8, "interruption didn't return properly");
        // Now it should handle the interrupt requested in the middle of the other one.
        let effect = run_until_out(&mut emulator, &mut program, 2000);
        assert_port!(0x20, effect, "Didn't start handling second interruption");
        let effect = run_until_out(&mut emulator, &mut program, 2000);
        assert_port!(
            0x21,
            effect,
            "Stopped in the middle of second interruption somehow"
        );
        // EI
        let (_, effect) = emulator.step(&mut program);
        assert_matches!(effect, ExecEffect::InterruptDelay);
        // RET
        let (_, effect) = emulator.step(&mut program);
        assert_matches!(effect, ExecEffect::Normal);
        // Did we really return?
        assert_eq!(emulator.pc.0, 0x8, "interruption didn't return properly");
        let (_, effect) = emulator.step(&mut program);
        assert_port!(3, effect, "interruption didn't return properly");
    }

    #[test]
    fn check_running_limit() {
        let mut program = [0xcb, 0x00, 0x00]; // jmp 0x00 : Infinite loop
        let mut emulator = Emulator::new();
        const LIMIT: usize = 10000;
        let (clock_cycles, _) = emulator.run_limited_clock(&mut program, LIMIT);
        assert!(clock_cycles > LIMIT, "Stopped before reaching the limit.");
    }
}
