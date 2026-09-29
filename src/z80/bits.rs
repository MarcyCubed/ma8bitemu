//! Bit instructions (prefix CB)

use crate::i8080::I8080FamilyEmulator;
use crate::memory::Memory;
use crate::z80::Emulator;

/// Perform a rotate or shift and sets the proper flags.
///
/// `action` is the actual function that manipulate bits. This function take care of the common
/// flags.
fn bit_move_flags(emulator: &mut Emulator, value: u8, action: fn(&mut Emulator, u8) -> u8) -> u8 {
    let value = value.rotate_left(1);
    let value = action(emulator, value);
    emulator.flags_from_value(value);
    emulator.parity_flag(value);
    emulator.nf = false;
    emulator.hf = false;
    value
}

/// Rotate a value left and copy the original most significant bit to the carry flag, updating
/// the flags.
///
/// Return the rotated value
pub(super) fn rlc_flags(emulator: &mut Emulator, value: u8) -> u8 {
    bit_move_flags(emulator, value, |emulator, value| {
        let value = value.rotate_left(1);
        emulator.cf = value & 0x1 != 0;
        value
    })
}

/// Rotate a value right and copy the original least significant bit to the carry flag, updating
/// the flags.
///
/// Return the rotated value
pub(super) fn rrc_flags(emulator: &mut Emulator, value: u8) -> u8 {
    bit_move_flags(emulator, value, |emulator, value| {
        emulator.cf = value & 0x1 != 0;
        let value = value.rotate_left(1);
        value
    })
}

/// Rotate left the 9-bit virtual register composed by the carry flag and the specified value.
///
/// Return the rotated value
pub(super) fn rl_flags(emulator: &mut Emulator, value: u8) -> u8 {
    bit_move_flags(emulator, value, |emulator, value| {
        let new_carry = value >> 7 != 0;
        let value = (value << 1) | emulator.cf as u8;
        emulator.cf = new_carry;
        value
    })
}

/// Rotate right the 9-bit virtual register composed by the carry flag and the specified value.
///
/// Return the rotated value
pub(super) fn rr_flags(emulator: &mut Emulator, value: u8) -> u8 {
    bit_move_flags(emulator, value, |emulator, value| {
        let new_carry = value & 1 != 0;
        let value = (value >> 1) | ((emulator.cf as u8) << 7);
        emulator.cf = new_carry;
        value
    })
}

/// Arithmetic left shift
pub(super) fn sla_flags(emulator: &mut Emulator, value: u8) -> u8 {
    bit_move_flags(emulator, value, |emulator, value| {
        emulator.cf = value >> 7 != 0;
        value << 1
    })
}

/// Arithmetic right shift
pub(super) fn sra_flags(emulator: &mut Emulator, value: u8) -> u8 {
    bit_move_flags(emulator, value, |emulator, value| {
        emulator.cf = value & 1 != 0;
        (value as i8 >> 1) as u8
    })
}

/// Logical left shift
///
/// Undocumented instruction
pub(super) fn sll_flags(emulator: &mut Emulator, value: u8) -> u8 {
    bit_move_flags(emulator, value, |emulator, value| {
        emulator.cf = value >> 7 != 0;
        (value << 1) | 1
    })
}

/// Logical right shift
///
/// Undocumented instruction
pub(super) fn srl_flags(emulator: &mut Emulator, value: u8) -> u8 {
    bit_move_flags(emulator, value, |emulator, value| {
        emulator.cf = value & 1 != 0;
        value >> 1
    })
}

/// Check if a bit of the value is 0
///
/// If the bit is `0`, sets the `Z` flag
pub(super) fn bit_flags(emulator: &mut Emulator, value: u8, bit_number: u32) {
    let bit = value & (1 << bit_number);
    emulator.flags_from_value(value);
    emulator.pf = bit == 0;
    emulator.nf = false;
    emulator.hf = true;
}

/// Check if a bit of a register is 0
macro_rules! bit_r {
    ($bit:literal, $emulator:ident, $reg:ident) => {{
        let value = $emulator.$reg.0;
        bit_flags($emulator, value, $bit);
        8
    }};
}

/// Check if a bit of a memory location is 0
macro_rules! bit_mem {
    ($bit:literal, $emulator:ident, $memory:ident) => {{
        let value = $emulator.load_hl($memory);
        bit_flags($emulator, value, $bit);
        $emulator.xy_from_value(($emulator.mem_ptr.0 >> 8) as u8);
        12
    }};
}

/// Reset a bit of a register
macro_rules! res_r {
    ($bit:literal, $emulator:ident, $reg:ident) => {{
        let value = $emulator.$reg.0 & !(1 << $bit);
        $emulator.$reg.0 = value;
        8
    }};
}

/// Reset a bit of the value pointed by hl
macro_rules! res_mem {
    ($bit:literal, $emulator:ident, $memory:ident) => {{
        let value = $emulator.load_hl($memory) & !(1 << $bit);
        $emulator.store_hl($memory, value);
        15
    }};
}

/// Set a bit of a register
macro_rules! set_r {
    ($bit:literal, $emulator:ident, $reg:ident) => {{
        let value = $emulator.$reg.0 | (1 << $bit);
        $emulator.$reg.0 = value;
        8
    }};
}

/// Set a bit of the value pointed by hl
macro_rules! set_mem {
    ($bit:literal, $emulator:ident, $memory:ident) => {{
        let value = $emulator.load_hl($memory) | (1 << $bit);
        $emulator.store_hl($memory, value);
        15
    }};
}

/// Runs a rotate or shift instruction on a register
macro_rules! rot_shift_r {
    ($operation:ident, $emulator:ident, $reg:ident) => {{
        $emulator.$reg.0 = $operation($emulator, $emulator.$reg.0);
        8
    }};
}

/// Runs a rotate or shift instruction on memory
macro_rules! rot_shift_mem {
    ($operation:ident, $emulator:ident, $memory:ident) => {{
        let value = $emulator.load_hl($memory);
        let value = $operation($emulator, value);
        $emulator.store_hl($memory, value);
        15
    }};
}

/// Execute a bit instruction
///
/// Return the time it takes to execute the instruction in clock cycles
pub(super) fn run_opcode(emulator: &mut Emulator, opcode: u8, memory: &mut impl Memory) -> u8 {
    emulator.inc_r();
    match opcode {
        // RLC
        0x00 => rot_shift_r!(rlc_flags, emulator, b),
        0x01 => rot_shift_r!(rlc_flags, emulator, c),
        0x02 => rot_shift_r!(rlc_flags, emulator, d),
        0x03 => rot_shift_r!(rlc_flags, emulator, e),
        0x04 => rot_shift_r!(rlc_flags, emulator, h),
        0x05 => rot_shift_r!(rlc_flags, emulator, l),
        0x07 => rot_shift_r!(rlc_flags, emulator, a),
        0x06 => rot_shift_mem!(rlc_flags, emulator, memory),
        // RRC
        0x08 => rot_shift_r!(rrc_flags, emulator, b),
        0x09 => rot_shift_r!(rrc_flags, emulator, c),
        0x0a => rot_shift_r!(rrc_flags, emulator, d),
        0x0b => rot_shift_r!(rrc_flags, emulator, e),
        0x0c => rot_shift_r!(rrc_flags, emulator, h),
        0x0d => rot_shift_r!(rrc_flags, emulator, l),
        0x0f => rot_shift_r!(rrc_flags, emulator, a),
        0x0e => rot_shift_mem!(rrc_flags, emulator, memory),
        // RL
        0x10 => rot_shift_r!(rl_flags, emulator, b),
        0x11 => rot_shift_r!(rl_flags, emulator, c),
        0x12 => rot_shift_r!(rl_flags, emulator, d),
        0x13 => rot_shift_r!(rl_flags, emulator, e),
        0x14 => rot_shift_r!(rl_flags, emulator, h),
        0x15 => rot_shift_r!(rl_flags, emulator, l),
        0x17 => rot_shift_r!(rl_flags, emulator, a),
        0x16 => rot_shift_mem!(rl_flags, emulator, memory),
        // RR
        0x18 => rot_shift_r!(rr_flags, emulator, b),
        0x19 => rot_shift_r!(rr_flags, emulator, c),
        0x1a => rot_shift_r!(rr_flags, emulator, d),
        0x1b => rot_shift_r!(rr_flags, emulator, e),
        0x1c => rot_shift_r!(rr_flags, emulator, h),
        0x1d => rot_shift_r!(rr_flags, emulator, l),
        0x1f => rot_shift_r!(rr_flags, emulator, a),
        0x1e => rot_shift_mem!(rr_flags, emulator, memory),
        // SLA
        0x20 => rot_shift_r!(sla_flags, emulator, b),
        0x21 => rot_shift_r!(sla_flags, emulator, c),
        0x22 => rot_shift_r!(sla_flags, emulator, d),
        0x23 => rot_shift_r!(sla_flags, emulator, e),
        0x24 => rot_shift_r!(sla_flags, emulator, h),
        0x25 => rot_shift_r!(sla_flags, emulator, l),
        0x27 => rot_shift_r!(sla_flags, emulator, a),
        0x26 => rot_shift_mem!(sla_flags, emulator, memory),
        // SRA
        0x28 => rot_shift_r!(sra_flags, emulator, b),
        0x29 => rot_shift_r!(sra_flags, emulator, c),
        0x2a => rot_shift_r!(sra_flags, emulator, d),
        0x2b => rot_shift_r!(sra_flags, emulator, e),
        0x2c => rot_shift_r!(sra_flags, emulator, h),
        0x2d => rot_shift_r!(sra_flags, emulator, l),
        0x2f => rot_shift_r!(sra_flags, emulator, a),
        0x2e => rot_shift_mem!(sra_flags, emulator, memory),
        // SLL
        0x30 => rot_shift_r!(sll_flags, emulator, b),
        0x31 => rot_shift_r!(sll_flags, emulator, c),
        0x32 => rot_shift_r!(sll_flags, emulator, d),
        0x33 => rot_shift_r!(sll_flags, emulator, e),
        0x34 => rot_shift_r!(sll_flags, emulator, h),
        0x35 => rot_shift_r!(sll_flags, emulator, l),
        0x37 => rot_shift_r!(sll_flags, emulator, a),
        0x36 => rot_shift_mem!(sll_flags, emulator, memory),
        // SRL
        0x38 => rot_shift_r!(srl_flags, emulator, b),
        0x39 => rot_shift_r!(srl_flags, emulator, c),
        0x3a => rot_shift_r!(srl_flags, emulator, d),
        0x3b => rot_shift_r!(srl_flags, emulator, e),
        0x3c => rot_shift_r!(srl_flags, emulator, h),
        0x3d => rot_shift_r!(srl_flags, emulator, l),
        0x3f => rot_shift_r!(srl_flags, emulator, a),
        0x3e => rot_shift_mem!(srl_flags, emulator, memory),
        // bit 0, r
        0x40 => bit_r!(0, emulator, b),
        0x41 => bit_r!(0, emulator, c),
        0x42 => bit_r!(0, emulator, d),
        0x43 => bit_r!(0, emulator, e),
        0x44 => bit_r!(0, emulator, h),
        0x45 => bit_r!(0, emulator, l),
        0x47 => bit_r!(0, emulator, a),
        0x46 => bit_mem!(0, emulator, memory),
        // bit 1, r
        0x48 => bit_r!(1, emulator, b),
        0x49 => bit_r!(1, emulator, c),
        0x4a => bit_r!(1, emulator, d),
        0x4b => bit_r!(1, emulator, e),
        0x4c => bit_r!(1, emulator, h),
        0x4d => bit_r!(1, emulator, l),
        0x4f => bit_r!(1, emulator, a),
        0x4e => bit_mem!(1, emulator, memory),
        // bit 2, r
        0x50 => bit_r!(2, emulator, b),
        0x51 => bit_r!(2, emulator, c),
        0x52 => bit_r!(2, emulator, d),
        0x53 => bit_r!(2, emulator, e),
        0x54 => bit_r!(2, emulator, h),
        0x55 => bit_r!(2, emulator, l),
        0x57 => bit_r!(2, emulator, a),
        0x56 => bit_mem!(2, emulator, memory),
        // bit 3, r
        0x58 => bit_r!(3, emulator, b),
        0x59 => bit_r!(3, emulator, c),
        0x5a => bit_r!(3, emulator, d),
        0x5b => bit_r!(3, emulator, e),
        0x5c => bit_r!(3, emulator, h),
        0x5d => bit_r!(3, emulator, l),
        0x5f => bit_r!(3, emulator, a),
        0x5e => bit_mem!(3, emulator, memory),
        // bit 4, r
        0x60 => bit_r!(4, emulator, b),
        0x61 => bit_r!(4, emulator, c),
        0x62 => bit_r!(4, emulator, d),
        0x63 => bit_r!(4, emulator, e),
        0x64 => bit_r!(4, emulator, h),
        0x65 => bit_r!(4, emulator, l),
        0x67 => bit_r!(4, emulator, a),
        0x66 => bit_mem!(4, emulator, memory),
        // bit 5, r
        0x68 => bit_r!(5, emulator, b),
        0x69 => bit_r!(5, emulator, c),
        0x6a => bit_r!(5, emulator, d),
        0x6b => bit_r!(5, emulator, e),
        0x6c => bit_r!(5, emulator, h),
        0x6d => bit_r!(5, emulator, l),
        0x6f => bit_r!(5, emulator, a),
        0x6e => bit_mem!(5, emulator, memory),
        // bit 6, r
        0x70 => bit_r!(6, emulator, b),
        0x71 => bit_r!(6, emulator, c),
        0x72 => bit_r!(6, emulator, d),
        0x73 => bit_r!(6, emulator, e),
        0x74 => bit_r!(6, emulator, h),
        0x75 => bit_r!(6, emulator, l),
        0x77 => bit_r!(6, emulator, a),
        0x76 => bit_mem!(6, emulator, memory),
        // bit 7, r
        0x78 => bit_r!(7, emulator, b),
        0x79 => bit_r!(7, emulator, c),
        0x7a => bit_r!(7, emulator, d),
        0x7b => bit_r!(7, emulator, e),
        0x7c => bit_r!(7, emulator, h),
        0x7d => bit_r!(7, emulator, l),
        0x7f => bit_r!(7, emulator, a),
        0x7e => bit_mem!(7, emulator, memory),
        // res 0, r
        0x80 => res_r!(0, emulator, b),
        0x81 => res_r!(0, emulator, c),
        0x82 => res_r!(0, emulator, d),
        0x83 => res_r!(0, emulator, e),
        0x84 => res_r!(0, emulator, h),
        0x85 => res_r!(0, emulator, l),
        0x87 => res_r!(0, emulator, a),
        0x86 => res_mem!(0, emulator, memory),
        // res 1, r
        0x88 => res_r!(1, emulator, b),
        0x89 => res_r!(1, emulator, c),
        0x8a => res_r!(1, emulator, d),
        0x8b => res_r!(1, emulator, e),
        0x8c => res_r!(1, emulator, h),
        0x8d => res_r!(1, emulator, l),
        0x8f => res_r!(1, emulator, a),
        0x8e => res_mem!(1, emulator, memory),
        // res 2, r
        0x90 => res_r!(2, emulator, b),
        0x91 => res_r!(2, emulator, c),
        0x92 => res_r!(2, emulator, d),
        0x93 => res_r!(2, emulator, e),
        0x94 => res_r!(2, emulator, h),
        0x95 => res_r!(2, emulator, l),
        0x97 => res_r!(2, emulator, a),
        0x96 => res_mem!(2, emulator, memory),
        // res 3, r
        0x98 => res_r!(3, emulator, b),
        0x99 => res_r!(3, emulator, c),
        0x9a => res_r!(3, emulator, d),
        0x9b => res_r!(3, emulator, e),
        0x9c => res_r!(3, emulator, h),
        0x9d => res_r!(3, emulator, l),
        0x9f => res_r!(3, emulator, a),
        0x9e => res_mem!(3, emulator, memory),
        // res 4, r
        0xa0 => res_r!(4, emulator, b),
        0xa1 => res_r!(4, emulator, c),
        0xa2 => res_r!(4, emulator, d),
        0xa3 => res_r!(4, emulator, e),
        0xa4 => res_r!(4, emulator, h),
        0xa5 => res_r!(4, emulator, l),
        0xa7 => res_r!(4, emulator, a),
        0xa6 => bit_mem!(4, emulator, memory),
        // res 5, r
        0xa8 => res_r!(5, emulator, b),
        0xa9 => res_r!(5, emulator, c),
        0xaa => res_r!(5, emulator, d),
        0xab => res_r!(5, emulator, e),
        0xac => res_r!(5, emulator, h),
        0xad => res_r!(5, emulator, l),
        0xaf => res_r!(5, emulator, a),
        0xae => res_mem!(5, emulator, memory),
        // res 6, r
        0xb0 => res_r!(6, emulator, b),
        0xb1 => res_r!(6, emulator, c),
        0xb2 => res_r!(6, emulator, d),
        0xb3 => res_r!(6, emulator, e),
        0xb4 => res_r!(6, emulator, h),
        0xb5 => res_r!(6, emulator, l),
        0xb7 => res_r!(6, emulator, a),
        0xb6 => res_mem!(6, emulator, memory),
        // res 7, r
        0xb8 => res_r!(7, emulator, b),
        0xb9 => res_r!(7, emulator, c),
        0xba => res_r!(7, emulator, d),
        0xbb => res_r!(7, emulator, e),
        0xbc => res_r!(7, emulator, h),
        0xbd => res_r!(7, emulator, l),
        0xbf => res_r!(7, emulator, a),
        0xbe => res_mem!(7, emulator, memory),
        // set 0, r
        0xc0 => set_r!(0, emulator, b),
        0xc1 => set_r!(0, emulator, c),
        0xc2 => set_r!(0, emulator, d),
        0xc3 => set_r!(0, emulator, e),
        0xc4 => set_r!(0, emulator, h),
        0xc5 => set_r!(0, emulator, l),
        0xc7 => set_r!(0, emulator, a),
        0xc6 => set_mem!(0, emulator, memory),
        // set 1, r
        0xc8 => set_r!(1, emulator, b),
        0xc9 => set_r!(1, emulator, c),
        0xca => set_r!(1, emulator, d),
        0xcb => set_r!(1, emulator, e),
        0xcc => set_r!(1, emulator, h),
        0xcd => set_r!(1, emulator, l),
        0xcf => set_r!(1, emulator, a),
        0xce => set_mem!(1, emulator, memory),
        // set 2, r
        0xd0 => set_r!(2, emulator, b),
        0xd1 => set_r!(2, emulator, c),
        0xd2 => set_r!(2, emulator, d),
        0xd3 => set_r!(2, emulator, e),
        0xd4 => set_r!(2, emulator, h),
        0xd5 => set_r!(2, emulator, l),
        0xd7 => set_r!(2, emulator, a),
        0xd6 => set_mem!(2, emulator, memory),
        // set 3, r
        0xd8 => set_r!(3, emulator, b),
        0xd9 => set_r!(3, emulator, c),
        0xda => set_r!(3, emulator, d),
        0xdb => set_r!(3, emulator, e),
        0xdc => set_r!(3, emulator, h),
        0xdd => set_r!(3, emulator, l),
        0xdf => set_r!(3, emulator, a),
        0xde => set_mem!(3, emulator, memory),
        // set 4, r
        0xe0 => set_r!(4, emulator, b),
        0xe1 => set_r!(4, emulator, c),
        0xe2 => set_r!(4, emulator, d),
        0xe3 => set_r!(4, emulator, e),
        0xe4 => set_r!(4, emulator, h),
        0xe5 => set_r!(4, emulator, l),
        0xe7 => set_r!(4, emulator, a),
        0xe6 => set_mem!(4, emulator, memory),
        // set 5, r
        0xe8 => set_r!(5, emulator, b),
        0xe9 => set_r!(5, emulator, c),
        0xea => set_r!(5, emulator, d),
        0xeb => set_r!(5, emulator, e),
        0xec => set_r!(5, emulator, h),
        0xed => set_r!(5, emulator, l),
        0xef => set_r!(5, emulator, a),
        0xee => set_mem!(5, emulator, memory),
        // set 6, r
        0xf0 => set_r!(6, emulator, b),
        0xf1 => set_r!(6, emulator, c),
        0xf2 => set_r!(6, emulator, d),
        0xf3 => set_r!(6, emulator, e),
        0xf4 => set_r!(6, emulator, h),
        0xf5 => set_r!(6, emulator, l),
        0xf7 => set_r!(6, emulator, a),
        0xf6 => set_mem!(6, emulator, memory),
        // set 7, r
        0xf8 => set_r!(7, emulator, b),
        0xf9 => set_r!(7, emulator, c),
        0xfa => set_r!(7, emulator, d),
        0xfb => set_r!(7, emulator, e),
        0xfc => set_r!(7, emulator, h),
        0xfd => set_r!(7, emulator, l),
        0xff => set_r!(7, emulator, a),
        0xfe => set_mem!(7, emulator, memory),
    }
}
