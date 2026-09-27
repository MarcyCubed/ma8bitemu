//! 8080 instructions that work differently in the Z80

use crate::i8080::I8080FamilyState;
use crate::i8080::math::{add_value, check_carry};
use crate::memory::Memory;
use crate::z80::State;
use core::num::Wrapping;

/// Do `a - b - borrow`, update the flags in the state and return the result
#[inline]
fn sub_flags(state: &mut State, a: u8, b: u8, borrow: bool) -> u8 {
    let a = a as u16;
    let b = b as u16;
    let mut c = Wrapping(a as u16);
    c -= b;
    c -= borrow as u16;
    state.cf = c.0 & 0x100 != 0;
    state.hf = check_carry(4, a, b, c.0);
    state.nf = true;
    state.pf = check_carry(7, a, b, c.0) != state.cf;
    let value = c.0 as u8;
    state.flags_from_value(value);
    value
}

/// Subtract a value and a borrow from A, updating the flags
pub(crate) fn sub_value(state: &mut State, value: u8, borrow: bool) {
    state.a.0 = sub_flags(state, state.a.0, value, borrow);
}

/// Decrement a value and set the appropriate flags on the state.
///
/// Return the decremented value.
#[inline]
pub(crate) fn dec_flags(state: &mut State, value: u8) -> u8 {
    let new_value = value.wrapping_sub(1);
    state.flags_from_value(new_value);
    state.pf = (1 << 7) & (value ^ new_value) != 0;
    state.hf = new_value & 0xf != 0xf;
    state.nf = true;
    new_value
}

/// Decrement a register and set the appropriate flags
///
/// Return the number of clock cycles it takes to execute the instruction
#[inline]
pub(crate) fn dec_r(
    state: &mut State,
    get_register: impl Fn(&mut State) -> &mut Wrapping<u8>,
) -> u8 {
    let old_value = *get_register(state);
    let new_value = dec_flags(state, old_value.0);
    get_register(state).0 = new_value;
    4
}

/// Decrement a value in memory and set the appropriate flags
///
/// Return the number of clock cycles it takes to execute the instruction
pub(crate) fn dec_mem(state: &mut State, memory: &mut impl Memory, address: u16, cycles: u8) -> u8 {
    let value = dec_flags(state, memory.load(address));
    memory.store(address, value);
    cycles
}

/// Adjust a BCD value after a math operation
///
/// Return the number of clock cycles it takes to execute the instruction
pub(crate) fn daa(state: &mut State) -> u8 {
    let a = state.a.0;

    let mut diff = 0;
    if state.hf || a & 0x0f > 9 {
        diff = 0x06;
    }
    if state.cf || a > 0x99 {
        diff += 0x60;
    }

    if state.nf {
        sub_value(state, diff, false);
    } else {
        add_value(state, diff, false);
    };

    state.parity_from_accumulator();
    4
}

/// Set the carry flag
///
/// Return the number of clock cycles it takes to execute the instruction
pub(crate) fn scf(state: &mut State) -> u8 {
    state.xy_from_accumulator();
    state.cf = true;
    state.nf = false;
    state.hf = false;
    4
}

/// Add a 16-bit value to HL and set the flags.
pub(crate) fn add_hl_value(state: &mut State, value: u16) {
    state.mem_ptr = (state.a.0 as u16).wrapping_add(1);
    let (result, carry) = state.get_hl().overflowing_add(value);
    state.nf = false;
    state.cf = carry;
    let [result_low, result_high] = result.to_le_bytes();
    state.hf = check_carry(
        State::H_FLAG_BIT,
        state.h.0 as u16,
        value,
        result_high as u16,
    );
    state.xy_from_value(result_high);
    state.h.0 = result_high;
    state.l.0 = result_low;
}

/// Complement of the accumulator
///
/// Return the number of clock cycles it took to execute the instruction
pub(crate) fn cpl(state: &mut State) -> u8 {
    state.a.0 = !state.a.0;
    state.nf = true;
    state.hf = true;
    state.xy_from_accumulator();
    4
}

/// Invert the carry flag
///
/// Return the number of clock cycles it took to execute the instruction
pub(crate) fn ccf(state: &mut State) -> u8 {
    state.hf = state.cf;
    state.cf = !state.cf;
    state.nf = false;
    state.xy_from_accumulator();

    4
}

/// Perform a logical AND between the value and the accumulator, updating the flags
///
/// Return the number of clock cycles it took to execute the instruction
pub(crate) fn and_value(state: &mut State, value: u8) -> u8 {
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
pub(crate) fn cp_value(state: &mut State, value: u8) -> u8 {
    sub_flags(state, state.a.0, value, false);
    4
}
