//! IX and IY instructions

use crate::i8080::I8080FamilyEmulator;
use crate::i8080::jump::{pop_stack, push_stack};
use crate::i8080::math::{add_value, inc_value, or_value, xor_value};
use crate::memory::Memory;
use crate::z80::z8080::{and_value, cp_value, dec_flags, sub_value};
use crate::z80::{Emulator, double_prefix, z8080};
use crate::{ExecEffect, Fetch};
use std::num::Wrapping;

/// A trait to abstract away IX and IY registers.
pub(super) trait IndexRegister {
    /// Get the value of the IX or IY register
    fn get(emulator: &Emulator) -> Wrapping<u16>;

    /// Get a mutable reference to the IX or YY register
    fn get_mut(emulator: &mut Emulator) -> &mut Wrapping<u16>;

    /// Calculate the address pointed by the index register and the displacement.
    ///
    /// Update `MEMPTR`
    fn displace(emulator: &mut Emulator, displacement: u8) -> u16 {
        emulator.mem_ptr = Self::get(emulator);
        emulator.mem_ptr += displacement as i8 as u16;
        emulator.mem_ptr.0
    }

    /// Get the high byte of the register
    fn high(emulator: &Emulator) -> u8 {
        let [_, h] = Self::get(emulator).0.to_le_bytes();
        h
    }

    /// Get the high byte of the register
    fn low(emulator: &Emulator) -> u8 {
        Self::get(emulator).0 as u8
    }
}

/// Selector for the IX register
pub(super) struct IX;

impl IndexRegister for IX {
    fn get(emulator: &Emulator) -> Wrapping<u16> {
        emulator.ix
    }

    fn get_mut(emulator: &mut Emulator) -> &mut Wrapping<u16> {
        &mut emulator.ix
    }
}

/// Selector for the IY register
pub(super) struct IY;

impl IndexRegister for IY {
    fn get(emulator: &Emulator) -> Wrapping<u16> {
        emulator.iy
    }

    fn get_mut(emulator: &mut Emulator) -> &mut Wrapping<u16> {
        &mut emulator.iy
    }
}

/// Copy the value of one of the high or low bytes of the index register into another register
macro_rules! ld_r_izr {
    ($emulator:ident , $register:ident, $izr:ident) => {{
        $emulator.$register.0 = I::$izr($emulator);
        8
    }};
}

/// Load a value from memory addressed by an indexed displacement into a register
macro_rules! ld_r_iz_d {
    ($emulator:ident , $register:ident, $memory:ident) => {{
        let d = $emulator.fetch_byte($memory);
        $emulator.$register.0 = $memory.load(I::displace($emulator, d));
        19
    }};
}

/// Copy a register to the high byte of the index register
macro_rules! ld_izh_r {
    ($emulator:ident , $register:ident) => {{
        let [low, _] = I::get($emulator).0.to_le_bytes();
        let high = $emulator.$register.0;
        I::get_mut($emulator).0 = u16::from_le_bytes([low, high]);
        8
    }};
}

/// Copy a register to the low byte of the index register
macro_rules! ld_izl_r {
    ($emulator:ident , $register:ident) => {{
        let [_, high] = I::get($emulator).0.to_le_bytes();
        let low = $emulator.$register.0;
        I::get_mut($emulator).0 = u16::from_le_bytes([low, high]);
        8
    }};
}

/// Store the value in the register in the memory pointed by iz+d
macro_rules! ld_iz_d_r {
    ($emulator:ident , $register:ident, $memory:ident) => {{
        let d = $emulator.fetch_byte($memory);
        let address = I::displace($emulator, d);
        $memory.store(address, $emulator.$register.0);
        19
    }};
}

/// Arithmetic instruction with the low or high byte of the index register
macro_rules! arith_izr {
    ($operation: expr, $izr:ident, $carry: expr, $emulator:ident) => {{
        $operation($emulator, I::$izr($emulator), $carry);
        8
    }};
}

/// Arithmetic instruction with an indexed value
macro_rules! arith_iz_d {
    ($operation: expr, $carry: expr, $emulator:ident, $memory:ident) => {{
        let d = $emulator.fetch_byte($memory);
        let value = $memory.load(I::displace($emulator, d));
        $operation($emulator, value, $carry);
        19
    }};
}

/// Logic instruction with the low or high byte of the index register
macro_rules! logic_izr {
    ($operation: expr, $izr:ident, $emulator:ident) => {{
        $operation($emulator, I::$izr($emulator));
        8
    }};
}

/// Logic instruction with an indexed value
macro_rules! logic_iz_d {
    ($operation: expr, $emulator:ident, $memory:ident) => {{
        let d = $emulator.fetch_byte($memory);
        let value = $memory.load(I::displace($emulator, d));
        $operation($emulator, value);
        19
    }};
}

/// Run an opcode prefixed by DD or FD
pub(super) fn run_opcode<I: IndexRegister>(
    emulator: &mut Emulator,
    opcode: u8,
    memory: &mut impl Memory,
) -> (u8, ExecEffect) {
    // Save R in case we backtrack
    let saved_r = emulator.r;
    emulator.inc_r();
    let clock_cycles = match opcode {
        // add iz, bc
        0x09 => {
            let value = emulator.get_bc();
            I::get_mut(emulator).0 =
                z8080::double_add_flags(emulator, I::get(emulator).0, value, false);
            15
        }
        // add iz, de
        0x19 => {
            let value = emulator.get_de();
            I::get_mut(emulator).0 =
                z8080::double_add_flags(emulator, I::get(emulator).0, value, false);
            15
        }
        // add iz, iz
        0x29 => {
            let value = I::get(emulator).0;
            I::get_mut(emulator).0 =
                z8080::double_add_flags(emulator, I::get(emulator).0, value, false);
            15
        }
        // add iz, sp
        0x39 => {
            let value = emulator.sp.0;
            I::get_mut(emulator).0 =
                z8080::double_add_flags(emulator, I::get(emulator).0, value, false);
            15
        }
        // ld iz, nn
        0x21 => {
            I::get_mut(emulator).0 = emulator.fetch_word(memory);
            14
        }
        // ld (nn), iz
        0x22 => {
            let address = emulator.fetch_word(memory);
            memory.store_16(address, I::get(emulator).0);
            20
        }
        // inc iz
        0x23 => {
            *I::get_mut(emulator) += 1;
            10
        }
        // inc izh
        0x24 => {
            let [izl, izh] = I::get(emulator).0.to_le_bytes();
            let izh = inc_value(emulator, Wrapping(izh)).0;
            I::get_mut(emulator).0 = u16::from_le_bytes([izl, izh]);
            8
        }
        // dec izh
        0x25 => {
            let [izl, izh] = I::get(emulator).0.to_le_bytes();
            let izh = dec_flags(emulator, izh);
            I::get_mut(emulator).0 = u16::from_le_bytes([izl, izh]);
            8
        }
        // inc izl
        0x2c => {
            let [izl, izh] = I::get(emulator).0.to_le_bytes();
            let izl = inc_value(emulator, Wrapping(izl)).0;
            I::get_mut(emulator).0 = u16::from_le_bytes([izl, izh]);
            8
        }
        // dec izl
        0x2d => {
            let [izl, izh] = I::get(emulator).0.to_le_bytes();
            let izl = dec_flags(emulator, izl);
            I::get_mut(emulator).0 = u16::from_le_bytes([izl, izh]);
            8
        }
        // ld izh, n
        0x26 => {
            let [izl, _] = I::get(emulator).0.to_le_bytes();
            let izh = emulator.fetch_byte(memory);
            I::get_mut(emulator).0 = u16::from_le_bytes([izl, izh]);
            11
        }
        // ld izl, n
        0x2e => {
            let [_, izh] = I::get(emulator).0.to_le_bytes();
            let izl = emulator.fetch_byte(memory);
            I::get_mut(emulator).0 = u16::from_le_bytes([izl, izh]);
            11
        }
        // ld iz, (nn)
        0x2a => {
            I::get_mut(emulator).0 = memory.load_16(emulator.fetch_word(memory));
            20
        }
        // dec iz
        0x2b => {
            *I::get_mut(emulator) -= 1;
            10
        }
        // inc (iz+d)
        0x34 => {
            let d = emulator.fetch_byte(memory);
            let address = I::displace(emulator, d);
            let value = memory.load(address);
            memory.store(address, inc_value(emulator, Wrapping(value)).0);
            23
        }
        // dec (iz+d)
        0x35 => {
            let d = emulator.fetch_byte(memory);
            let address = I::displace(emulator, d);
            let value = memory.load(address);
            memory.store(address, dec_flags(emulator, value));
            23
        }

        // ld (iz+d), n
        0x36 => {
            let d = emulator.fetch_byte(memory);
            let n = emulator.fetch_byte(memory);
            let address = I::displace(emulator, d);
            memory.store(address, n);
            19
        }
        0x44 => ld_r_izr!(emulator, b, high),    // ld b, izh
        0x45 => ld_r_izr!(emulator, b, low),     // ld b, izl
        0x46 => ld_r_iz_d!(emulator, b, memory), // ld b, (iz+d)
        0x4c => ld_r_izr!(emulator, c, high),    // ld c, izh
        0x4d => ld_r_izr!(emulator, c, low),     // ld c, izl
        0x4e => ld_r_iz_d!(emulator, c, memory), // ld c, (iz+d)
        0x54 => ld_r_izr!(emulator, d, high),    // ld d, izh
        0x55 => ld_r_izr!(emulator, d, low),     // ld d, izl
        0x56 => ld_r_iz_d!(emulator, d, memory), // ld d, (iz+d)
        0x5c => ld_r_izr!(emulator, e, high),    // ld e, izh
        0x5d => ld_r_izr!(emulator, e, low),     // ld e, izl
        0x5e => ld_r_iz_d!(emulator, e, memory), // ld e, (iz+d)
        0x60 => ld_izh_r!(emulator, b),          // ld izh, b
        0x61 => ld_izh_r!(emulator, c),          // ld izh, c
        0x62 => ld_izh_r!(emulator, d),          // ld izh, d
        0x63 => ld_izh_r!(emulator, e),          // ld izh, e
        0x67 => ld_izh_r!(emulator, a),          // ld izh, a
        0x64 => 8,                               // ld izh, izh
        // ld izh, izl
        0x65 => {
            let low = I::low(emulator);
            I::get_mut(emulator).0 = u16::from_le_bytes([low, low]);
            8
        }
        0x66 => ld_r_iz_d!(emulator, h, memory), // ld h, (iz+d)
        0x68 => ld_izl_r!(emulator, b),          // ld izl, b
        0x69 => ld_izl_r!(emulator, c),          // ld izl, c
        0x6a => ld_izl_r!(emulator, d),          // ld izl, d
        0x6b => ld_izl_r!(emulator, e),          // ld izl, e
        0x6f => ld_izl_r!(emulator, a),          // ld izl, a
        // ld izl, izh
        0x6c => {
            let high = I::high(emulator);
            I::get_mut(emulator).0 = u16::from_le_bytes([high, high]);
            8
        }
        0x6d => 8,                               // ld izl, izl
        0x6e => ld_r_iz_d!(emulator, l, memory), // ld h, (iz+d)
        0x70 => ld_iz_d_r!(emulator, b, memory), // ld (iz+d), b
        0x71 => ld_iz_d_r!(emulator, c, memory), // ld (iz+d), c
        0x72 => ld_iz_d_r!(emulator, d, memory), // ld (iz+d), d
        0x73 => ld_iz_d_r!(emulator, e, memory), // ld (iz+d), e
        0x74 => ld_iz_d_r!(emulator, h, memory), // ld (iz+d), h
        0x75 => ld_iz_d_r!(emulator, l, memory), // ld (iz+d), l
        0x77 => ld_iz_d_r!(emulator, a, memory), // ld (iz+d), a
        0x7c => ld_r_izr!(emulator, a, high),    // ld a, izh
        0x7d => ld_r_izr!(emulator, a, low),     // ld a, izl
        0x7e => ld_r_iz_d!(emulator, a, memory), // ld a, (iz+d)
        // Arithmetic and logic operations
        0x84 => arith_izr!(add_value, high, false, emulator), // add a, izh
        0x85 => arith_izr!(add_value, low, false, emulator),  // add a, izl
        0x86 => arith_iz_d!(add_value, false, emulator, memory), // add a, (iz+d)
        0x8c => arith_izr!(add_value, high, emulator.cf, emulator), // adc a, izh
        0x8d => arith_izr!(add_value, low, emulator.cf, emulator), // adc a, izl
        0x8e => arith_iz_d!(add_value, emulator.cf, emulator, memory), // adc a, (iz+d)
        0x94 => arith_izr!(sub_value, high, false, emulator), // sub a, izh
        0x95 => arith_izr!(sub_value, low, false, emulator),  // sub a, izl
        0x96 => arith_iz_d!(sub_value, false, emulator, memory), // sub a, (iz+d)
        0x9c => arith_izr!(sub_value, high, emulator.cf, emulator), // sbc a, izh
        0x9d => arith_izr!(sub_value, low, emulator.cf, emulator), // sbc a, izl
        0x9e => arith_iz_d!(sub_value, emulator.cf, emulator, memory), // sbc a, (iz+d)
        0xa4 => logic_izr!(and_value, high, emulator),        // and a, izh
        0xa5 => logic_izr!(and_value, low, emulator),         // and a, izl
        0xa6 => logic_iz_d!(and_value, emulator, memory),     // and a, (iz+d)
        0xac => logic_izr!(xor_value, high, emulator),        // xor a, izh
        0xad => logic_izr!(xor_value, low, emulator),         // xor a, izl
        0xae => logic_iz_d!(xor_value, emulator, memory),     // xor a, (iz+d)
        0xb4 => logic_izr!(or_value, high, emulator),         // or a, izh
        0xb5 => logic_izr!(or_value, low, emulator),          // or a, izl
        0xb6 => logic_iz_d!(or_value, emulator, memory),      // or a, (iz+d)
        0xbc => logic_izr!(cp_value, high, emulator),         // cp a, izh
        0xbd => logic_izr!(cp_value, low, emulator),          // cp a, izl
        0xbe => logic_iz_d!(cp_value, emulator, memory),      // cp a, (iz+d)
        // pop iz
        0xe1 => {
            I::get_mut(emulator).0 = pop_stack(emulator, memory);
            14
        }
        // push iz
        0xe5 => {
            push_stack(emulator, memory, I::get(emulator).0);
            15
        }
        // ex (sp), iz
        0xe3 => {
            let iz = I::get(emulator).0;
            I::get_mut(emulator).0 = memory.load_16(emulator.sp.0);
            memory.store_16(emulator.sp.0, iz);
            23
        }
        // jp (iz)
        0xe9 => {
            emulator.pc = I::get(emulator);
            emulator.mem_ptr = emulator.pc;
            8
        }
        // ld sp, ix
        0xf9 => {
            emulator.sp = I::get(emulator);
            10
        }
        // Instruction has two prefixes
        0xcb => {
            let d = emulator.fetch_byte(memory);
            let opcode = emulator.fetch_byte(memory);
            let address = I::displace(emulator, d);
            double_prefix::run_opcode(emulator, address, opcode, memory)
        }
        // Prefix on a normal instruction behaves as a NOP
        _ => {
            // Backtrack, so we'll interpret this opcode as a regular instruction next time
            emulator.r = saved_r;
            emulator.pc -= 1;
            4 // Just a regular NOP. Nothing to see
        }
    };
    (clock_cycles, ExecEffect::Normal)
}
