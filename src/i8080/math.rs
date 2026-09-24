//! Math instructions

use crate::i8080::I8080FamilyState;
use crate::i8080::state::State;
use crate::memory::Memory;
use core::num::Wrapping;

/// Increment a 16-bit register
pub(crate) fn inx<S: I8080FamilyState>(
    state: &mut S,
    getter: fn(&S) -> u16,
    setter: fn(&mut S, u16),
) -> u8 {
    let inc = getter(state).wrapping_add(1);
    setter(state, inc);
    5
}

/// Decrement a 16-bit register
pub(crate) fn dcx<S: I8080FamilyState>(
    state: &mut S,
    getter: fn(&S) -> u16,
    setter: fn(&mut S, u16),
) -> u8 {
    let inc = getter(state).wrapping_sub(1);
    setter(state, inc);
    5
}

/// Increment a register and set the appropriate flags
#[inline]
pub(crate) fn inc<S: I8080FamilyState>(
    state: &mut S,
    get_register: impl Fn(&mut S) -> &mut Wrapping<u8>,
) -> u8 {
    let old_value = *get_register(state);
    let new_value = inc_value(state, old_value);
    *get_register(state) = new_value;
    5
}

/// Increment a value and set the appropriate flags on the state.
///
/// Return the incremented value.
#[inline]
fn inc_value(state: &mut impl I8080FamilyState, value: Wrapping<u8>) -> Wrapping<u8> {
    let mut new_value = value;
    new_value += 1;
    state.flags_from_value(new_value.0);
    state.overflow_flag((1 << 7) & (value.0 ^ new_value.0) != 0);
    state.set_af((1 << State::A_FLAG_BIT) & (value.0 ^ new_value.0) != 0);
    state.set_nf(false);
    new_value
}

/// Increments the value in memory pointed by HL
pub(crate) fn inc_mem<S: I8080FamilyState>(state: &mut S, memory: &mut impl Memory) -> u8 {
    let address = state.get_hl();
    let value = inc_value(state, Wrapping(memory.load(address)));
    memory.store(address, value.0);
    10
}

/// Decrement a register and set the appropriate flags
#[inline]
pub(crate) fn dec<S: I8080FamilyState>(
    state: &mut S,
    get_register: impl Fn(&mut S) -> &mut Wrapping<u8>,
) -> u8 {
    let old_value = *get_register(state);
    let new_value = dec_value(state, old_value);
    *get_register(state) = new_value;
    5
}

/// Decrement a value and set the appropriate flags on the state.
///
/// Return the decremented value.
#[inline]
fn dec_value(state: &mut impl I8080FamilyState, value: Wrapping<u8>) -> Wrapping<u8> {
    let mut new_value = value;
    new_value -= 1;
    state.flags_from_value(new_value.0);
    state.overflow_flag((1 << 7) & (value.0 ^ new_value.0) != 0);
    state.set_af(new_value.0 & 0xf != 0xf);
    state.set_nf(true);
    new_value
}

/// Decrements the value in memory pointed by HL
pub(crate) fn dec_mem<S: I8080FamilyState>(state: &mut S, memory: &mut impl Memory) -> u8 {
    let address = state.get_hl();
    let value = dec_value(state, Wrapping(memory.load(address)));
    memory.store(address, value.0);
    10
}

/// Rotate the accumulator left and copy the original most significant bit to the carry flag
pub(crate) fn rlc(state: &mut impl I8080FamilyState) -> u8 {
    state.get_a_mut().0 = state.get_a().0.rotate_left(1);
    state.set_cf(state.get_a().0 & 0x1 != 0);
    4
}

/// Rotate the 9-bit value composed by the C flag and the accumulator to the left
pub(crate) fn ral(state: &mut impl I8080FamilyState) -> u8 {
    let new_c_flag = state.get_a().0 & (1 << 7) != 0;
    let new_a = state.get_a().0 << 1 | state.get_cf() as u8;
    *state.get_a_mut() = Wrapping(new_a);
    state.set_cf(new_c_flag);
    4
}

/// Rotate the accumulator right and copy the original least significant bit to the carry flag
pub(crate) fn rrc(state: &mut impl I8080FamilyState) -> u8 {
    state.set_cf(state.get_a().0 & 0x1 != 0);
    state.get_a_mut().0 = state.get_a().0.rotate_right(1);
    4
}

/// Rotate the 9-bit value composed by the C flag and the accumulator to the right
pub(crate) fn rar(state: &mut impl I8080FamilyState) -> u8 {
    let new_c_flag = state.get_a().0 & 0x1 != 0;
    state.get_a_mut().0 = (state.get_a().0 >> 1) | ((state.get_cf() as u8) << 7);
    state.set_cf(new_c_flag);
    4
}

/// Add a value to HL, updating the carry flag
#[inline]
pub(crate) fn dad(state: &mut State, register: fn(&State) -> u16) -> u8 {
    let (hl, carry) = state.hl().overflowing_add(register(state));
    state.set_hl(hl);
    state.cf = carry;
    10
}

/// Add a value and a carry to A, updating the flags
#[inline]
pub(crate) fn add_value(state: &mut State, value: u8, carry: bool) {
    let mut sum = Wrapping(state.a.0 as u16);
    sum += value as u16;
    sum += carry as u16;
    state.cf = sum.0 & (1 << 8) != 0;
    let sum = sum.0 as u8;
    state.af = (sum ^ value ^ state.a.0) & 0x10 != 0;
    state.a.0 = sum;
    state.flags_from_value(sum);
}

/// Subtract a value and a borrow from A, updating the flags
#[inline]
pub(crate) fn sub_value(state: &mut State, value: u8, carry: bool) {
    add_value(state, !value, !carry);
    state.cf = !state.cf;
    //state.af = !state.af;
}

/// Perform a logical AND between the value and the accumulator, updating the flags
#[inline]
pub(crate) fn and_value(state: &mut State, value: u8) {
    let a = state.a.0;
    state.a.0 &= value;
    state.flags_from_accumulator();
    state.cf = false;
    // What were the Intel engineers smoking?!
    state.af = (a | value) & 0b1000 != 0;
}

/// Perform a logical OR between the value and the accumulator, updating the flags
#[inline]
pub(crate) fn or_value(state: &mut State, value: u8) {
    state.a.0 |= value;
    state.flags_from_accumulator();
    state.cf = false;
    state.af = false;
}

/// Perform a logical XOR between the value and the accumulator, updating the flags
#[inline]
pub(crate) fn xor_value(state: &mut State, value: u8) {
    state.a.0 ^= value;
    state.flags_from_accumulator();
    state.cf = false;
    state.af = false;
}

/// Compare a value with the accumulator
pub(crate) fn cmp_value(state: &mut State, value: u8) {
    let (diff, carry) = state.a.0.overflowing_sub(value);
    state.cf = carry;
    state.flags_from_value(diff);
    state.af = (state.a.0 ^ diff ^ value) & 0x10 == 0;
}

/// Adjust the accumulator to BCD
pub(crate) fn daa(state: &mut State) -> u8 {
    let mut diff = 0;
    if state.a.0 & 0xf > 0x9 || state.af {
        diff += 0x06
    }
    if state.a.0 > 0x99 || state.cf {
        diff += 0x60;
    }
    let old_cf = state.cf;
    add_value(state, diff, false);
    state.cf |= old_cf;
    4
}
