//! The core of the emulator

use crate::i8080::state::State;
use crate::i8080::{I8080FamilyState, jump, load, math};
use crate::memory::Memory;
use crate::{ExecEffect, Fetch};

/// The 8080 emulator
#[derive(Debug, Clone, Copy)]
pub struct Emulator {
    /// Internal state of the processor
    pub state: State,
    /// The effect of the last instruction
    last_effect: ExecEffect,
    /// The interrupt vector if an interrupt was caused by external hardware
    interrupt_vector: Option<u8>,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            state: State::new(),
            last_effect: ExecEffect::Normal,
            interrupt_vector: None,
        }
    }
}

impl Fetch for Emulator {
    fn fetch_byte(&mut self, memory: &impl Memory) -> u8 {
        self.state.fetch_byte(memory)
    }
}

impl crate::EmulatorCore for Emulator {
    /// Get the next instruction
    fn next_instruction(&mut self, memory: &impl Memory) -> u8 {
        // If we can and should trigger an interrupt...
        if let Some(vector) = self.interrupt_vector
            && self.state.inte
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
                0x01 => load::lxi(&mut self.state, memory, |state, value| state.set_bc(value)),
                // LXI D, d16
                0x11 => load::lxi(&mut self.state, memory, |state, value| state.set_de(value)),
                // LXI H, d16
                0x21 => load::lxi(&mut self.state, memory, |state, value| state.set_hl(value)),
                // LXI SP, d16
                0x31 => load::lxi(&mut self.state, memory, |state, value| state.set_sp(value)),
                // STAX B
                0x02 => load::stax(&self.state, memory, |s| s.get_bc()),
                // STAX D
                0x12 => load::stax(&self.state, memory, |s| s.get_de()),
                // SHLD a16
                0x22 => load::shld(&mut self.state, memory),
                // STA a16
                0x32 => load::sta(&mut self.state, memory),
                // INX B
                0x03 => math::inx(&mut self.state, State::get_bc, State::set_bc),
                // INX D
                0x13 => math::inx(&mut self.state, State::get_de, State::set_de),
                // INX H
                0x23 => math::inx(&mut self.state, State::get_hl, State::set_hl),
                // INX SP
                0x33 => math::inx(&mut self.state, State::get_sp_u16, State::set_sp),
                // INR B
                0x04 => math::inc(&mut self.state, |s| &mut s.b),
                // INR D
                0x14 => math::inc(&mut self.state, |s| &mut s.d),
                // INR H
                0x24 => math::inc(&mut self.state, |s| &mut s.h),
                // INR C
                0x0c => math::inc(&mut self.state, |s| &mut s.c),
                // INR E
                0x1c => math::inc(&mut self.state, |s| &mut s.e),
                // INR L
                0x2c => math::inc(&mut self.state, |s| &mut s.l),
                // INR A
                0x3c => math::inc(&mut self.state, |s| &mut s.a),
                // INR M
                0x34 => math::inc_mem(&mut self.state, memory),
                // DCR B
                0x05 => math::dec(&mut self.state, |s| &mut s.b),
                // DCR D
                0x15 => math::dec(&mut self.state, |s| &mut s.d),
                // DCR H
                0x25 => math::dec(&mut self.state, |s| &mut s.h),
                // DCR C
                0x0d => math::dec(&mut self.state, |s| &mut s.c),
                // DCR E
                0x1d => math::dec(&mut self.state, |s| &mut s.e),
                // DCR L
                0x2d => math::dec(&mut self.state, |s| &mut s.l),
                // DCR A
                0x3d => math::dec(&mut self.state, |s| &mut s.a),
                // DCR M
                0x35 => math::dec_mem(&mut self.state, memory),
                // MVI B, d8
                0x06 => load::mvi(&mut self.state, memory, State::get_b_mut),
                // MVI D, d8
                0x16 => load::mvi(&mut self.state, memory, State::get_d_mut),
                // MVI H, d8
                0x26 => load::mvi(&mut self.state, memory, State::get_h_mut),
                // MVI C, d8
                0x0e => load::mvi(&mut self.state, memory, State::get_c_mut),
                // MVI E, d8
                0x1e => load::mvi(&mut self.state, memory, State::get_e_mut),
                // MVI L, d8
                0x2e => load::mvi(&mut self.state, memory, State::get_l_mut),
                // MVI A, d8
                0x3e => load::mvi(&mut self.state, memory, State::get_a_mut),
                // MVI M
                0x36 => load::mvi_mem(&mut self.state, memory),
                // RLC
                0x07 => math::rlc(&mut self.state),
                // RAL
                0x17 => math::ral(&mut self.state),
                // DAA
                0x27 => math::daa(&mut self.state),
                // STC
                0x37 => math::stc(&mut self.state),
                // DAD B
                0x09 => math::dad(&mut self.state, State::get_bc),
                // DAD D
                0x19 => math::dad(&mut self.state, State::get_de),
                // DAD H
                0x29 => math::dad(&mut self.state, State::get_hl),
                // DAD SP
                0x39 => math::dad(&mut self.state, State::get_sp_u16),
                // LDAX B
                0x0a => load::ldax(&mut self.state, memory, State::get_bc),
                // LDAX D
                0x1a => load::ldax(&mut self.state, memory, State::get_de),
                // LHLD a16
                0x2a => load::lhld(&mut self.state, memory),
                // LDA a16
                0x3a => load::lda(&mut self.state, memory),
                // DCX B
                0x0b => math::dcx(&mut self.state, State::get_bc, State::set_bc),
                // DCX D
                0x1b => math::dcx(&mut self.state, State::get_de, State::set_de),
                // DCX H
                0x2b => math::dcx(&mut self.state, State::get_hl, State::set_hl),
                // DCX SP
                0x3b => math::dcx(&mut self.state, State::get_sp_u16, State::set_sp),
                // RRC
                0x0f => math::rrc(&mut self.state),
                // RAR
                0x1F => math::rar(&mut self.state),
                // CMA
                0x2f => math::cma(&mut self.state),
                // CMC
                0x3f => math::cmc(&mut self.state),
                // MOV X, X
                0x40 => 5, // B, B
                0x41 => load::mov(&mut self.state, State::get_b_mut, State::get_c, 5),
                0x42 => load::mov(&mut self.state, State::get_b_mut, State::get_d, 5),
                0x43 => load::mov(&mut self.state, State::get_b_mut, State::get_e, 5),
                0x44 => load::mov(&mut self.state, State::get_b_mut, State::get_h, 5),
                0x45 => load::mov(&mut self.state, State::get_b_mut, State::get_l, 5),
                0x46 => load::mov_r_mem(&mut self.state, State::get_b_mut, memory),
                0x47 => load::mov(&mut self.state, State::get_b_mut, State::get_a, 5),

                0x48 => load::mov(&mut self.state, State::get_c_mut, State::get_b, 5),
                0x49 => 5, // C, C
                0x4a => load::mov(&mut self.state, State::get_c_mut, State::get_d, 5),
                0x4b => load::mov(&mut self.state, State::get_c_mut, State::get_e, 5),
                0x4c => load::mov(&mut self.state, State::get_c_mut, State::get_h, 5),
                0x4d => load::mov(&mut self.state, State::get_c_mut, State::get_l, 5),
                0x4e => load::mov_r_mem(&mut self.state, State::get_c_mut, memory),
                0x4f => load::mov(&mut self.state, State::get_c_mut, State::get_a, 5),

                0x50 => load::mov(&mut self.state, State::get_d_mut, State::get_b, 5),
                0x51 => load::mov(&mut self.state, State::get_d_mut, State::get_c, 5),
                0x52 => 5, // D, D
                0x53 => load::mov(&mut self.state, State::get_d_mut, State::get_e, 5),
                0x54 => load::mov(&mut self.state, State::get_d_mut, State::get_h, 5),
                0x55 => load::mov(&mut self.state, State::get_d_mut, State::get_l, 5),
                0x56 => load::mov_r_mem(&mut self.state, State::get_d_mut, memory),
                0x57 => load::mov(&mut self.state, State::get_d_mut, State::get_a, 5),

                0x58 => load::mov(&mut self.state, State::get_e_mut, State::get_b, 5),
                0x59 => load::mov(&mut self.state, State::get_e_mut, State::get_c, 5),
                0x5a => load::mov(&mut self.state, State::get_e_mut, State::get_d, 5),
                0x5b => 5, // E, E
                0x5c => load::mov(&mut self.state, State::get_e_mut, State::get_h, 5),
                0x5d => load::mov(&mut self.state, State::get_e_mut, State::get_l, 5),
                0x5e => load::mov_r_mem(&mut self.state, State::get_e_mut, memory),
                0x5f => load::mov(&mut self.state, State::get_e_mut, State::get_a, 5),

                0x60 => load::mov(&mut self.state, State::get_h_mut, State::get_b, 5),
                0x61 => load::mov(&mut self.state, State::get_h_mut, State::get_c, 5),
                0x62 => load::mov(&mut self.state, State::get_h_mut, State::get_d, 5),
                0x63 => load::mov(&mut self.state, State::get_h_mut, State::get_e, 5),
                0x64 => 5, // H, H
                0x65 => load::mov(&mut self.state, State::get_h_mut, State::get_l, 5),
                0x66 => load::mov_r_mem(&mut self.state, State::get_h_mut, memory),
                0x67 => load::mov(&mut self.state, State::get_h_mut, State::get_a, 5),

                0x68 => load::mov(&mut self.state, State::get_l_mut, State::get_b, 5),
                0x69 => load::mov(&mut self.state, State::get_l_mut, State::get_c, 5),
                0x6a => load::mov(&mut self.state, State::get_l_mut, State::get_d, 5),
                0x6b => load::mov(&mut self.state, State::get_l_mut, State::get_e, 5),
                0x6c => load::mov(&mut self.state, State::get_l_mut, State::get_h, 5),
                0x6d => 5, // L, L
                0x6e => load::mov_r_mem(&mut self.state, State::get_l_mut, memory),
                0x6f => load::mov(&mut self.state, State::get_l_mut, State::get_a, 5),

                0x78 => load::mov(&mut self.state, State::get_a_mut, State::get_b, 5),
                0x79 => load::mov(&mut self.state, State::get_a_mut, State::get_c, 5),
                0x7a => load::mov(&mut self.state, State::get_a_mut, State::get_d, 5),
                0x7b => load::mov(&mut self.state, State::get_a_mut, State::get_e, 5),
                0x7c => load::mov(&mut self.state, State::get_a_mut, State::get_h, 5),
                0x7d => load::mov(&mut self.state, State::get_a_mut, State::get_l, 5),
                0x7e => load::mov_r_mem(&mut self.state, State::get_a_mut, memory),
                0x7f => 5, // A, A

                0x70 => load::mov_mem_r(&mut self.state, memory, State::get_b),
                0x71 => load::mov_mem_r(&mut self.state, memory, State::get_c),
                0x72 => load::mov_mem_r(&mut self.state, memory, State::get_d),
                0x73 => load::mov_mem_r(&mut self.state, memory, State::get_e),
                0x74 => load::mov_mem_r(&mut self.state, memory, State::get_h),
                0x75 => load::mov_mem_r(&mut self.state, memory, State::get_l),
                0x77 => load::mov_mem_r(&mut self.state, memory, State::get_a),
                // HLT
                0x76 => {
                    self.state.pc -= 1;
                    break 'main (7, ExecEffect::Halt);
                }
                // ADD X
                0x80 => math::add_r(&mut self.state, State::get_b),
                0x81 => math::add_r(&mut self.state, State::get_c),
                0x82 => math::add_r(&mut self.state, State::get_d),
                0x83 => math::add_r(&mut self.state, State::get_e),
                0x84 => math::add_r(&mut self.state, State::get_h),
                0x85 => math::add_r(&mut self.state, State::get_l),
                0x86 => math::add_mem(&mut self.state, memory),
                0x87 => math::add_r(&mut self.state, State::get_a),
                // ADC
                0x88 => math::adc_r(&mut self.state, State::get_b),
                0x89 => math::adc_r(&mut self.state, State::get_c),
                0x8a => math::adc_r(&mut self.state, State::get_d),
                0x8b => math::adc_r(&mut self.state, State::get_e),
                0x8c => math::adc_r(&mut self.state, State::get_h),
                0x8d => math::adc_r(&mut self.state, State::get_l),
                0x8e => math::adc_mem(&mut self.state, memory),
                0x8f => math::adc_r(&mut self.state, State::get_a),
                // SUB
                0x90 => math::sub_r(&mut self.state, State::get_b),
                0x91 => math::sub_r(&mut self.state, State::get_c),
                0x92 => math::sub_r(&mut self.state, State::get_d),
                0x93 => math::sub_r(&mut self.state, State::get_e),
                0x94 => math::sub_r(&mut self.state, State::get_h),
                0x95 => math::sub_r(&mut self.state, State::get_l),
                0x96 => math::sub_mem(&mut self.state, memory),
                0x97 => math::sub_r(&mut self.state, State::get_a),
                // SBB
                0x98 => math::sbb_r(&mut self.state, State::get_b),
                0x99 => math::sbb_r(&mut self.state, State::get_c),
                0x9a => math::sbb_r(&mut self.state, State::get_d),
                0x9b => math::sbb_r(&mut self.state, State::get_e),
                0x9c => math::sbb_r(&mut self.state, State::get_h),
                0x9d => math::sbb_r(&mut self.state, State::get_l),
                0x9e => math::sbb_mem(&mut self.state, memory),
                0x9f => math::sbb_r(&mut self.state, State::get_a),
                // ANA
                0xa0 => math::ana_r(&mut self.state, State::get_b),
                0xa1 => math::ana_r(&mut self.state, State::get_c),
                0xa2 => math::ana_r(&mut self.state, State::get_d),
                0xa3 => math::ana_r(&mut self.state, State::get_e),
                0xa4 => math::ana_r(&mut self.state, State::get_h),
                0xa5 => math::ana_r(&mut self.state, State::get_l),
                0xa6 => math::ana_mem(&mut self.state, memory),
                0xa7 => math::ana_r(&mut self.state, State::get_a),
                // XRA
                0xa8 => math::xra_r(&mut self.state, State::get_b),
                0xa9 => math::xra_r(&mut self.state, State::get_c),
                0xaa => math::xra_r(&mut self.state, State::get_d),
                0xab => math::xra_r(&mut self.state, State::get_e),
                0xac => math::xra_r(&mut self.state, State::get_h),
                0xad => math::xra_r(&mut self.state, State::get_l),
                0xae => math::xra_mem(&mut self.state, memory),
                0xaf => math::xra_r(&mut self.state, State::get_a),
                // ORA
                0xb0 => math::ora_r(&mut self.state, State::get_b),
                0xb1 => math::ora_r(&mut self.state, State::get_c),
                0xb2 => math::ora_r(&mut self.state, State::get_d),
                0xb3 => math::ora_r(&mut self.state, State::get_e),
                0xb4 => math::ora_r(&mut self.state, State::get_h),
                0xb5 => math::ora_r(&mut self.state, State::get_l),
                0xb6 => math::ora_mem(&mut self.state, memory),
                0xb7 => math::ora_r(&mut self.state, State::get_a),
                // CMP
                0xb8 => math::cmp_r(&mut self.state, State::get_b),
                0xb9 => math::cmp_r(&mut self.state, State::get_c),
                0xba => math::cmp_r(&mut self.state, State::get_d),
                0xbb => math::cmp_r(&mut self.state, State::get_e),
                0xbc => math::cmp_r(&mut self.state, State::get_h),
                0xbd => math::cmp_r(&mut self.state, State::get_l),
                0xbe => math::cmp_mem(&mut self.state, memory),
                0xbf => math::cmp_r(&mut self.state, State::get_a),
                0xc0 => jump::ret_cond(&mut self.state, memory, |s| !s.zf), // RNZ
                0xc8 => jump::ret_cond(&mut self.state, memory, |s| s.zf),  // RZ
                0xd0 => jump::ret_cond(&mut self.state, memory, |s| !s.cf), // RNC
                0xd8 => jump::ret_cond(&mut self.state, memory, |s| s.cf),  // RC
                0xe0 => jump::ret_cond(&mut self.state, memory, |s| !s.pf), // RPO
                0xe8 => jump::ret_cond(&mut self.state, memory, |s| s.pf),  // RPE
                0xf0 => jump::ret_cond(&mut self.state, memory, |s| !s.sf), // RP
                0xf8 => jump::ret_cond(&mut self.state, memory, |s| s.sf),  // RM
                0xc9 | 0xd9 => jump::ret(&mut self.state, memory),          // RET
                0xc1 => jump::pop(&mut self.state, memory, State::set_bc),  // POP B
                0xd1 => jump::pop(&mut self.state, memory, State::set_de),  // POP D
                0xe1 => jump::pop(&mut self.state, memory, State::set_hl),  // POP E
                0xf1 => jump::pop(&mut self.state, memory, State::set_af),  // POP PSW
                0xc2 => jump::jp_cond_nn(&mut self.state, memory, |s| !s.zf), // JNZ
                0xca => jump::jp_cond_nn(&mut self.state, memory, |s| s.zf), // JZ
                0xd2 => jump::jp_cond_nn(&mut self.state, memory, |s| !s.cf), // JNC
                0xda => jump::jp_cond_nn(&mut self.state, memory, |s| s.cf), // JC
                0xe2 => jump::jp_cond_nn(&mut self.state, memory, |s| !s.pf), // JPO
                0xea => jump::jp_cond_nn(&mut self.state, memory, |s| s.pf), // JPE
                0xf2 => jump::jp_cond_nn(&mut self.state, memory, |s| !s.sf), // JP
                0xfa => jump::jp_cond_nn(&mut self.state, memory, |s| s.sf), // JM
                0xc3 | 0xcb => jump::jp_cond_nn(&mut self.state, memory, |_| true), // JMP
                // OUT d8
                0xd3 => {
                    let port = self.fetch_byte(memory);
                    break 'main (
                        10,
                        ExecEffect::Out {
                            port,
                            data: self.state.a.0,
                        },
                    );
                }
                // XTHL
                0xe3 => load::xthl(&mut self.state, memory, 18),
                // DI
                0xf3 => {
                    self.state.inte = false;
                    4
                }
                // CNZ, CZ, CNC, CC, CPO, CPE, CP, CM
                0xc4 => jump::call_cond_nn(&mut self.state, memory, |s| !s.zf, 17, 11), // CNZ
                0xcc => jump::call_cond_nn(&mut self.state, memory, |s| s.zf, 17, 11),  // CZ
                0xd4 => jump::call_cond_nn(&mut self.state, memory, |s| !s.cf, 17, 11), // CNC
                0xdc => jump::call_cond_nn(&mut self.state, memory, |s| s.cf, 17, 11),  // CC
                0xe4 => jump::call_cond_nn(&mut self.state, memory, |s| !s.pf, 17, 11), // CPO
                0xec => jump::call_cond_nn(&mut self.state, memory, |s| s.pf, 17, 11),  // CPE
                0xf4 => jump::call_cond_nn(&mut self.state, memory, |s| !s.sf, 17, 11), // CP
                0xfc => jump::call_cond_nn(&mut self.state, memory, |s| s.sf, 17, 11),  // CM
                // CALL
                0xcd | 0xdd | 0xed | 0xfd => {
                    jump::call_cond_nn(&mut self.state, memory, |_| true, 17, 11)
                }
                0xc5 => jump::push(&mut self.state, memory, State::get_bc), // PUSH B
                0xd5 => jump::push(&mut self.state, memory, State::get_de), // PUSH D
                0xe5 => jump::push(&mut self.state, memory, State::get_hl), // PUSH H
                0xf5 => jump::push(&mut self.state, memory, State::get_af), // PUSH PSW
                0xc6 => math::alu_imm(&mut self.state, |s, v| math::add_value(s, v, false), memory), // ADI
                0xce => math::alu_imm(&mut self.state, |s, v| math::add_value(s, v, s.cf), memory), // ACI
                0xd6 => math::alu_imm(&mut self.state, |s, v| math::sub_value(s, v, false), memory), // SUI
                0xde => math::alu_imm(&mut self.state, |s, v| math::sub_value(s, v, s.cf), memory), // SBI
                0xe6 => math::alu_imm(&mut self.state, math::and_value, memory), // ANI
                0xee => math::alu_imm(&mut self.state, math::xor_value, memory), // XRI
                0xf6 => math::alu_imm(&mut self.state, math::or_value, memory),  // ORI
                0xfe => math::alu_imm(&mut self.state, math::cmp_value, memory), // CPI
                // RST
                0xc7 => jump::rst(&mut self.state, memory, 0x00),
                0xcf => jump::rst(&mut self.state, memory, 0x08),
                0xd7 => jump::rst(&mut self.state, memory, 0x10),
                0xdf => jump::rst(&mut self.state, memory, 0x18),
                0xe7 => jump::rst(&mut self.state, memory, 0x20),
                0xef => jump::rst(&mut self.state, memory, 0x28),
                0xf7 => jump::rst(&mut self.state, memory, 0x30),
                0xff => jump::rst(&mut self.state, memory, 0x38),
                0xe9 => jump::jp_hl(&mut self.state, 5), // PCHL
                0xf9 => load::sphl(&mut self.state, 5),  // SPHL
                // IN d8
                0xdb => {
                    let port = self.fetch_byte(memory);
                    break 'main (10, ExecEffect::In { port });
                }
                0xeb => load::xchg(&mut self.state), // XCHG
                // EI
                0xfb => {
                    self.state.inte = true;
                    break 'main (4, ExecEffect::InterruptDelay);
                }
            };
            (clock_cycles, ExecEffect::Normal)
        };
        self.last_effect = result.1;
        result
    }
}

impl Emulator {
    /// Give the emulator an input requested by the IN instruction
    pub fn input(&mut self, value: u8) {
        self.state.a.0 = value
    }

    /// Cause an interrupt
    pub fn interrupt(&mut self, vector: u8) {
        self.interrupt_vector = Some(vector);
        self.state.inte = false;
    }
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
        assert_eq!(
            emulator.state.pc.0, 0x8,
            "interruption didn't return properly"
        );
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
        assert_eq!(
            emulator.state.pc.0, 0x8,
            "interruption didn't return properly"
        );
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
