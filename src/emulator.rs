//! The core of the emulator

use crate::memory::Memory;
use crate::state::State;
use crate::{jump, math};
use core::num::Wrapping;

/// The 8080 emulator
#[derive(Debug, Clone, Copy)]
pub struct Emulator {
    /// Internal state of the processor
    pub state: State,
    /// State of the emulation itself
    emulator_state: EmulatorState,
    /// The interrupt vector if an interrupt was caused by external hardware
    interrupt_vector: Option<u8>,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            state: State::new(),
            emulator_state: EmulatorState::Ready,
            interrupt_vector: None,
        }
    }

    /// Load one byte from memory
    pub fn fetch_byte(&mut self, memory: &impl Memory) -> u8 {
        let address = self.state.pc.0;
        self.state.pc += 1;
        memory.load(address)
    }

    /// Load a 16-bit word from memory
    pub fn fetch_word(&mut self, memory: &impl Memory) -> u16 {
        let byte_0 = self.fetch_byte(memory);
        let byte_1 = self.fetch_byte(memory);
        u16::from_le_bytes([byte_0, byte_1])
    }

    /// Execute a single opcode.
    ///
    /// Return the number of clock cycles it took to execute the instruction and the result of the execution
    pub fn run_opcode(&mut self, opcode: u8, memory: &mut impl Memory) -> (u8, EmulatorState) {
        let clock_cycles = match opcode {
            // NOP
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => 4,
            // LXI B, d16
            0x01 => {
                let d = self.fetch_word(memory);
                self.state.set_bc(d);
                10
            }
            // LXI D, d16
            0x11 => {
                let d = self.fetch_word(memory);
                self.state.set_de(d);
                10
            }
            // LXI H, d16
            0x21 => {
                let d = self.fetch_word(memory);
                self.state.set_hl(d);
                10
            }
            // LXI SP, d16
            0x31 => {
                let d = self.fetch_word(memory);
                self.state.sp = Wrapping(d);
                10
            }
            // STAX B
            0x02 => {
                memory.store(self.state.bc(), self.state.a.0);
                7
            }
            // STAX D
            0x12 => {
                memory.store(self.state.de(), self.state.a.0);
                7
            }
            // SHLD a16
            0x22 => {
                let address_0 = self.fetch_word(memory);
                let address_1 = address_0.wrapping_add(1);
                memory.store(address_0, self.state.l.0);
                memory.store(address_1, self.state.h.0);
                16
            }
            // STA a16
            0x32 => {
                let address = self.fetch_word(memory);
                memory.store(address, self.state.a.0);
                13
            }
            // INX B
            0x03 => {
                let inc = self.state.bc().wrapping_add(1);
                self.state.set_bc(inc);
                5
            }
            // INX D
            0x13 => {
                let inc = self.state.de().wrapping_add(1);
                self.state.set_de(inc);
                5
            }
            // INX H
            0x23 => {
                let inc = self.state.hl().wrapping_add(1);
                self.state.set_hl(inc);
                5
            }
            // INX SP
            0x33 => {
                self.state.sp += 1;
                5
            }
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
            0x34 => {
                let address = self.state.hl();
                let value = math::inc_value(&mut self.state, Wrapping(memory.load(address)));
                memory.store(address, value.0);
                10
            }
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
            0x35 => {
                let address = self.state.hl();
                let value = math::dec_value(&mut self.state, Wrapping(memory.load(address)));
                memory.store(address, value.0);
                10
            }
            // MVI B, d8
            0x06 => {
                self.state.b.0 = self.fetch_byte(memory);
                7
            }
            // MVI D, d8
            0x16 => {
                self.state.d.0 = self.fetch_byte(memory);
                7
            }
            // MVI H, d8
            0x26 => {
                self.state.h.0 = self.fetch_byte(memory);
                7
            }
            // MVI C, d8
            0x0e => {
                self.state.c.0 = self.fetch_byte(memory);
                7
            }
            // MVI E, d8
            0x1e => {
                self.state.e.0 = self.fetch_byte(memory);
                7
            }
            // MVI L, d8
            0x2e => {
                self.state.l.0 = self.fetch_byte(memory);
                7
            }
            // MVI A, d8
            0x3e => {
                self.state.a.0 = self.fetch_byte(memory);
                7
            }
            // MVI M
            0x36 => {
                let address = self.state.hl();
                memory.store(address, self.fetch_byte(memory));
                10
            }
            // RLC
            0x07 => {
                self.state.a.0 = self.state.a.0.rotate_left(1);
                self.state.cf = self.state.a.0 & 0x1 != 0;
                4
            }
            // RAL
            0x17 => {
                let new_c_flag = self.state.a.0 & (1 << 7) != 0;
                let new_a = self.state.a.0 << 1 | self.state.cf as u8;
                self.state.a.0 = new_a;
                self.state.cf = new_c_flag;
                4
            }
            // DAA
            0x27 => {
                math::daa(&mut self.state);
                4
            }
            // STC
            0x37 => {
                self.state.cf = true;
                4
            }
            // DAD B
            0x09 => {
                let value = self.state.bc();
                math::dad_value(&mut self.state, value);
                10
            }
            // DAD D
            0x19 => {
                let value = self.state.de();
                math::dad_value(&mut self.state, value);
                10
            }
            // DAD H
            0x29 => {
                let value = self.state.hl();
                math::dad_value(&mut self.state, value);
                10
            }
            // DAD SP
            0x39 => {
                let value = self.state.sp.0;
                math::dad_value(&mut self.state, value);
                10
            }
            // LDAX B
            0x0a => {
                let value = memory.load(self.state.bc());
                self.state.a.0 = value;
                7
            }
            // LDAX D
            0x1a => {
                let value = memory.load(self.state.de());
                self.state.a.0 = value;
                7
            }
            // LHLD a16
            0x2a => {
                let address = self.fetch_word(memory);
                self.state.l.0 = memory.load(address);
                self.state.h.0 = memory.load(address.wrapping_add(1));
                16
            }
            // LDA a16
            0x3a => {
                let address = self.fetch_word(memory);
                self.state.a.0 = memory.load(address);
                13
            }
            // DCX B
            0x0b => {
                let inc = self.state.bc().wrapping_sub(1);
                self.state.set_bc(inc);
                5
            }
            // DCX D
            0x1b => {
                let inc = self.state.de().wrapping_sub(1);
                self.state.set_de(inc);
                5
            }
            // DCX H
            0x2b => {
                let inc = self.state.hl().wrapping_sub(1);
                self.state.set_hl(inc);
                5
            }
            // DCX SP
            0x3b => {
                self.state.sp -= 1;
                5
            }
            // RRC
            0x0f => {
                self.state.cf = self.state.a.0 & 0x1 != 0;
                self.state.a.0 = self.state.a.0.rotate_right(1);
                4
            }
            // RAR
            0x1F => {
                let new_c_flag = self.state.a.0 & 0x1 != 0;
                self.state.a.0 = (self.state.a.0 >> 1) | ((self.state.cf as u8) << 7);
                self.state.cf = new_c_flag;
                4
            }
            // CMA
            0x2f => {
                self.state.a = !self.state.a;
                4
            }
            // CMC
            0x3f => {
                self.state.cf = !self.state.cf;
                4
            }
            // MOV X, X
            0x40..=0x75 | 0x77..=0x7f => {
                let dst = (opcode >> 3) & 0b111;

                // Return if moving to the same register
                if (opcode ^ (opcode >> 3)) & 0b111 == 0 {
                    return (5, EmulatorState::Ready);
                }

                // Get the destination
                fn get_dst(state: &mut State, src: u8) -> Option<&mut u8> {
                    match src {
                        0b000 => Some(&mut state.b.0),
                        0b001 => Some(&mut state.c.0),
                        0b010 => Some(&mut state.d.0),
                        0b011 => Some(&mut state.e.0),
                        0b100 => Some(&mut state.h.0),
                        0b101 => Some(&mut state.l.0),
                        0b111 => Some(&mut state.a.0),
                        _ => None,
                    }
                }

                let mut clock_cycles = 5;

                let src_value = self.state.source_from_opcode(opcode).unwrap_or_else(|| {
                    clock_cycles = 7;
                    memory.load(self.state.hl())
                });

                match get_dst(&mut self.state, dst) {
                    None => {
                        clock_cycles = 7;
                        memory.store(self.state.hl(), src_value);
                    }
                    Some(dst) => {
                        *dst = src_value;
                    }
                }
                clock_cycles
            }
            // HLT
            0x76 => {
                self.state.pc -= 1;
                return (7, EmulatorState::Halted);
            }
            // ADD
            0x80..=0x87 => {
                let (value, clock_cycles) = match self.state.source_from_opcode(opcode) {
                    None => (memory.load(self.state.hl()), 7),
                    Some(n) => (n, 4),
                };
                math::add_value(&mut self.state, value, false);
                clock_cycles
            }
            // ADC
            0x88..=0x8f => {
                let (value, clock_cycles) = match self.state.source_from_opcode(opcode) {
                    None => (memory.load(self.state.hl()), 7),
                    Some(n) => (n, 4),
                };
                let carry = self.state.cf;
                math::add_value(&mut self.state, value, carry);
                clock_cycles
            }
            // SUB
            0x90..=0x97 => {
                let (value, clock_cycles) = match self.state.source_from_opcode(opcode) {
                    None => (memory.load(self.state.hl()), 7),
                    Some(n) => (n, 4),
                };
                math::sub_value(&mut self.state, value, false);
                clock_cycles
            }
            // SBB
            0x98..=0x9f => {
                let (value, clock_cycles) = match self.state.source_from_opcode(opcode) {
                    None => (memory.load(self.state.hl()), 7),
                    Some(n) => (n, 4),
                };
                let carry = self.state.cf;
                math::sub_value(&mut self.state, value, carry);
                clock_cycles
            }
            // ANA
            0xa0..=0xa7 => {
                let (value, clock_cycles) = match self.state.source_from_opcode(opcode) {
                    None => (memory.load(self.state.hl()), 7),
                    Some(n) => (n, 4),
                };
                math::and_value(&mut self.state, value);
                clock_cycles
            }
            // XRA
            0xa8..=0xaf => {
                let (value, clock_cycles) = match self.state.source_from_opcode(opcode) {
                    None => (memory.load(self.state.hl()), 7),
                    Some(n) => (n, 4),
                };
                math::xor_value(&mut self.state, value);
                clock_cycles
            }
            // ORA
            0xb0..=0xb7 => {
                let (value, clock_cycles) = match self.state.source_from_opcode(opcode) {
                    None => (memory.load(self.state.hl()), 7),
                    Some(n) => (n, 4),
                };
                math::or_value(&mut self.state, value);
                clock_cycles
            }
            // CMP
            0xb8..=0xbf => {
                let (value, clock_cycles) = match self.state.source_from_opcode(opcode) {
                    None => (memory.load(self.state.hl()), 7),
                    Some(n) => (n, 4),
                };
                math::cmp_value(&mut self.state, value);
                clock_cycles
            }
            // RNZ, RZ, RNC, RC, RPO, RPE, RP, RM
            0xc0 | 0xc8 | 0xd0 | 0xd8 | 0xe0 | 0xe8 | 0xf0 | 0xf8 => {
                let cond = self.state.opcode_to_condition(opcode);
                if jump::ret_if(&mut self.state, memory, cond) {
                    11
                } else {
                    5
                }
            }
            // RET
            0xc9 | 0xd9 => {
                jump::ret(&mut self.state, memory);
                10
            }
            // POP B
            0xc1 => {
                let bc = jump::pop(&mut self.state, memory);
                self.state.set_bc(bc);
                10
            }
            // POP D
            0xd1 => {
                let de = jump::pop(&mut self.state, memory);
                self.state.set_de(de);
                10
            }
            // POP H
            0xe1 => {
                let hl = jump::pop(&mut self.state, memory);
                self.state.set_hl(hl);
                10
            }
            // POP PSW
            0xf1 => {
                let af = jump::pop(&mut self.state, memory).to_le_bytes();
                self.state.deserialize_flags(af[0]);
                self.state.a.0 = af[1];
                10
            }
            // JNZ, JZ, JNC, JC, JPO, JPE, JP, JM
            0xc2 | 0xca | 0xd2 | 0xda | 0xe2 | 0xea | 0xf2 | 0xfa => {
                let address = self.fetch_word(memory);
                if self.state.opcode_to_condition(opcode) {
                    self.state.pc.0 = address;
                }
                10
            }
            // JMP a16
            0xc3 | 0xcb => {
                let address = self.fetch_word(memory);
                self.state.pc.0 = address;
                10
            }
            // OUT d8
            0xd3 => {
                let port = self.fetch_byte(memory);
                return (
                    10,
                    EmulatorState::Out {
                        port,
                        data: self.state.a.0,
                    },
                );
            }
            // XTHL
            0xe3 => {
                let popped = jump::pop(&mut self.state, memory);
                let hl = self.state.hl();
                jump::push(&mut self.state, memory, hl);
                self.state.set_hl(popped);
                18
            }
            // DI
            0xf3 => {
                self.state.inte = false;
                4
            }
            // CNZ, CZ, CNC, CC, CPO, CPE, CP, CM
            0xc4 | 0xcc | 0xd4 | 0xdc | 0xe4 | 0xec | 0xf4 | 0xfc => {
                let address = self.fetch_word(memory);
                let cond = self.state.opcode_to_condition(opcode);
                if jump::call_if(&mut self.state, memory, cond, address) {
                    17
                } else {
                    11
                }
            }
            // CALL a16
            0xcd | 0xdd | 0xed | 0xfd => {
                let address = self.fetch_word(memory);
                jump::call(&mut self.state, memory, address);
                17
            }
            // PUSH B
            0xc5 => {
                let value = self.state.bc();
                jump::push(&mut self.state, memory, value);
                11
            }
            // PUSH D
            0xd5 => {
                let value = self.state.de();
                jump::push(&mut self.state, memory, value);
                11
            }
            // PUSH H
            0xe5 => {
                let value = self.state.hl();
                jump::push(&mut self.state, memory, value);
                11
            }
            // PUSH PSW
            0xf5 => {
                let af = u16::from_le_bytes([self.state.serialize_flags(), self.state.a.0]);
                jump::push(&mut self.state, memory, af);
                11
            }
            // ADI d8
            0xc6 => {
                let d8 = self.fetch_byte(memory);
                math::add_value(&mut self.state, d8, false);
                7
            }
            // ACI d8
            0xce => {
                let d8 = self.fetch_byte(memory);
                let carry = self.state.cf;
                math::add_value(&mut self.state, d8, carry);
                7
            }
            // SUI d8
            0xd6 => {
                let d8 = self.fetch_byte(memory);
                math::sub_value(&mut self.state, d8, false);
                7
            }
            // SBI d8
            0xde => {
                let d8 = self.fetch_byte(memory);
                let carry = self.state.cf;
                math::sub_value(&mut self.state, d8, carry);
                7
            }
            // ANI d8
            0xe6 => {
                let d8 = self.fetch_byte(memory);
                math::and_value(&mut self.state, d8);
                7
            }
            // XRI d8
            0xee => {
                let d8 = self.fetch_byte(memory);
                math::xor_value(&mut self.state, d8);
                7
            }
            // ORI d8
            0xf6 => {
                let d8 = self.fetch_byte(memory);
                math::or_value(&mut self.state, d8);
                7
            }
            // CPI d8
            0xfe => {
                let d8 = self.fetch_byte(memory);
                math::cmp_value(&mut self.state, d8);
                7
            }
            // RST
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => {
                let address = (opcode & 0b111000) as u16;
                jump::call(&mut self.state, memory, address);
                11
            }
            // PCHL
            0xe9 => {
                self.state.pc.0 = self.state.hl();
                5
            }
            // SPHL
            0xf9 => {
                self.state.sp.0 = self.state.hl();
                5
            }
            // IN d8
            0xdb => {
                let port = self.fetch_byte(memory);
                return (10, EmulatorState::In { port });
            }
            // XCHG
            0xeb => {
                let de = self.state.de();
                let hl = self.state.hl();
                self.state.set_hl(de);
                self.state.set_de(hl);
                4
            }
            // EI
            0xfb => {
                self.state.inte = true;
                return (4, EmulatorState::InterruptDelay);
            }
        };
        (clock_cycles, EmulatorState::Ready)
    }

    /// Give the emulator an input requested by the IN instruction
    pub fn input(&mut self, value: u8) {
        self.state.a.0 = value
    }

    /// Cause an interrupt
    pub fn interrupt(&mut self, vector: u8) {
        self.interrupt_vector = Some(vector)
    }

    /// Run a program until it exceeds the number of clock cycles, halts or performs I/O
    ///
    /// Return the number of clock cycles it took to execute the program and the result of the execution
    pub fn run_limited(
        &mut self,
        memory: &mut impl Memory,
        limit: usize,
    ) -> (usize, EmulatorState) {
        let mut clock_cycles = 0;

        let result = loop {
            if clock_cycles > limit {
                break EmulatorState::Ready;
            }

            let (cycles, result) = self.step(memory);
            clock_cycles += cycles as usize;
            match result {
                EmulatorState::Ready => continue,
                result => break result,
            }
        };

        (clock_cycles, result)
    }

    /// Run a program until it halts or performs I/O
    ///
    /// Return the number of clock cycles it took to execute the program and the result of the execution.
    ///
    /// This may loop forever.
    pub fn run(&mut self, memory: &mut impl Memory) -> (usize, EmulatorState) {
        let mut clock_cycles = 0;

        loop {
            let (cycles, result) = self.step(memory);
            clock_cycles += cycles as usize;
            match result {
                EmulatorState::Ready => continue,
                result => return (clock_cycles, result),
            }
        }
    }

    /// Execute the next instruction
    ///
    /// Return the number of clock cycles it took to execute the instruction and the result of the execution
    pub fn step(&mut self, memory: &mut impl Memory) -> (u8, EmulatorState) {
        // If we can and should trigger an interrupt
        let opcode = if let Some(vector) = self.interrupt_vector
            && self.state.inte
            && self.emulator_state != EmulatorState::InterruptDelay
        {
            vector
        } else {
            self.fetch_byte(memory)
        };
        let result = self.run_opcode(opcode, memory);
        self.emulator_state = result.1;
        result
    }
}

/// The state of the emulator
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EmulatorState {
    /// The Emulator is ready to run the next instruction
    Ready,
    /// The emulator is halted
    Halted,
    /// The emulator can't process interrupts yet
    InterruptDelay,
    /// The emulator wants to read the given port
    In { port: u8 },
    /// The emulator wants to write data to the given port
    Out { port: u8, data: u8 },
}
