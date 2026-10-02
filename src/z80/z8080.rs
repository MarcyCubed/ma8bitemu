//! 8080 instructions that work differently in the Z80

use crate::i8080;
use crate::i8080::I8080FamilyEmulator;
use crate::i8080::math::check_carry;
use crate::memory::Memory;
use crate::z80::Emulator;
use core::num::Wrapping;

/// Do `a - b - borrow`, update the flags and return the result
#[inline]
pub(super) fn sub_flags(emulator: &mut Emulator, a: u8, b: u8, borrow: bool) -> u8 {
    let a = a as u16;
    let b = b as u16;
    let mut c = Wrapping(a as u16);
    c -= b;
    c -= borrow as u16;
    emulator.cf = c.0 & 0x100 != 0;
    emulator.hf = check_carry(4, a, b, c.0);
    emulator.nf = true;
    emulator.pf = check_carry(7, a, b, c.0) != emulator.cf;
    let value = c.0 as u8;
    emulator.flags_from_value(value);
    value
}

/// Subtract a value and a borrow from A, updating the flags
pub(super) fn sub_value(emulator: &mut Emulator, value: u8, borrow: bool) {
    emulator.a.0 = sub_flags(emulator, emulator.a.0, value, borrow);
}

/// Decrement a value and set the appropriate flags.
///
/// Return the decremented value.
#[inline]
pub(super) fn dec_flags(emulator: &mut Emulator, value: u8) -> u8 {
    let new_value = value.wrapping_sub(1);
    emulator.flags_from_value(new_value);
    emulator.pf = new_value == 0x7f;
    emulator.hf = (new_value ^ value) & (1 << Emulator::H_FLAG_BIT) != 0;
    emulator.nf = true;
    new_value
}

/// Decrement a register and set the appropriate flags
///
/// Return the number of clock cycles it takes to execute the instruction
#[inline]
pub(super) fn dec_r(
    emulator: &mut Emulator,
    get_register: impl Fn(&mut Emulator) -> &mut Wrapping<u8>,
) -> u8 {
    let old_value = *get_register(emulator);
    let new_value = dec_flags(emulator, old_value.0);
    get_register(emulator).0 = new_value;
    4
}

/// Decrement a value in memory and set the appropriate flags
///
/// Return the number of clock cycles it takes to execute the instruction
pub(super) fn dec_mem(
    state: &mut Emulator,
    memory: &mut impl Memory,
    address: u16,
    cycles: u8,
) -> u8 {
    let value = dec_flags(state, memory.load(address));
    memory.store(address, value);
    cycles
}

/// Adjust a BCD value after a math operation
///
/// Return the number of clock cycles it takes to execute the instruction
pub(super) fn daa(emulator: &mut Emulator) -> u8 {
    let a = emulator.a.0;

    let mut diff = 0;
    if emulator.hf || a & 0x0f > 9 {
        diff = 0x06;
    }
    if emulator.cf || a > 0x99 {
        diff += 0x60;
        emulator.cf = true;
    }

    if emulator.nf {
        emulator.hf = emulator.hf && (a & 0x0F) < 0x06;
        emulator.a -= diff;
    } else {
        emulator.hf = (a & 0x0F) > 0x09;
        emulator.a += diff;
    };

    emulator.flags_from_accumulator();
    emulator.parity_from_accumulator();
    4
}

/// Set the carry flag
///
/// Return the number of clock cycles it takes to execute the instruction
pub(super) fn scf(state: &mut Emulator) -> u8 {
    state.xy_from_accumulator();
    state.cf = true;
    state.nf = false;
    state.hf = false;
    4
}

/// Adds two 16-bit values and a carry together and set the flags.
///
/// Return the sum
pub(super) fn double_add_flags(state: &mut Emulator, a: u16, b: u16, carry: bool) -> u16 {
    state.mem_ptr.0 = a;
    state.mem_ptr += 1;
    let (result, carry_0) = a.overflowing_add(b);
    let (result, carry_1) = result.overflowing_add(carry as u16);
    state.nf = false;
    state.cf = carry_0 || carry_1;
    let result_high = result >> 8;
    state.hf = check_carry(Emulator::H_FLAG_BIT + 8, a, b, result);
    state.xy_from_value(result_high as u8);
    result
}

/// Complement of the accumulator
///
/// Return the number of clock cycles it took to execute the instruction
pub(super) fn cpl(state: &mut Emulator) -> u8 {
    state.a.0 = !state.a.0;
    state.nf = true;
    state.hf = true;
    state.xy_from_accumulator();
    4
}

/// Invert the carry flag
///
/// Return the number of clock cycles it took to execute the instruction
pub(super) fn ccf(state: &mut Emulator) -> u8 {
    state.hf = state.cf;
    state.cf = !state.cf;
    state.nf = false;
    state.xy_from_accumulator();

    4
}

/// Perform a logical AND between the value and the accumulator, updating the flags
///
/// Return the number of clock cycles it took to execute the instruction
pub(super) fn and_value(state: &mut Emulator, value: u8) -> u8 {
    state.a.0 &= value;
    state.flags_from_accumulator();
    state.parity_from_accumulator();
    state.cf = false;
    state.nf = false;
    state.hf = true;
    4
}

/// Compare the accumulator to the value, updating the flags
///
/// Return the number of clock cycles it took to execute the instruction
pub(super) fn cp_value(state: &mut Emulator, value: u8) -> u8 {
    sub_flags(state, state.a.0, value, false);
    state.xy_from_value(value);
    4
}

/// Rotate the accumulator left and copy the bit that wrapped around to the carry flag
pub(super) fn rlca(state: &mut Emulator) -> u8 {
    state.a.0 = state.a.0.rotate_left(1);
    state.cf = state.a.0 & 0x1 != 0;
    state.nf = false;
    state.hf = false;
    state.xy_from_accumulator();
    4
}

/// Rotate the accumulator right and copy the bit that wrapped around to the carry flag
pub(crate) fn rrca(state: &mut Emulator) -> u8 {
    state.cf = state.a.0 & 0x1 != 0;
    state.a.0 = state.a.0.rotate_right(1);
    state.nf = false;
    state.hf = false;
    state.xy_from_accumulator();
    4
}

/// Rotate the 9-bit value composed by the C flag and the accumulator to the left
pub(super) fn rla(emulator: &mut Emulator) -> u8 {
    i8080::math::ral(emulator);
    emulator.nf = false;
    emulator.hf = false;
    emulator.xy_from_accumulator();
    4
}

/// Rotate the 9-bit value composed by the C flag and the accumulator to the right
pub(super) fn rra(emulator: &mut Emulator) -> u8 {
    i8080::math::rar(emulator);
    emulator.nf = false;
    emulator.hf = false;
    emulator.xy_from_accumulator();
    4
}
