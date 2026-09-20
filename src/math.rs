//! Math instructions

use crate::state::State;
use core::num::Wrapping;

/// Increment a register and set the appropriate flags
#[inline]
pub(crate) fn inc(state: &mut State, get_register: impl Fn(&mut State) -> &mut Wrapping<u8>) -> u8 {
    //let register = register(state);
    let old_value = *get_register(state);
    let new_value = inc_value(state, old_value);
    *get_register(state) = new_value;
    5
}

/// Increment a value and set the appropriate flags on the state.
///
/// Return the incremented value.
#[inline]
pub(crate) fn inc_value(state: &mut State, value: Wrapping<u8>) -> Wrapping<u8> {
    let mut new_value = value;
    new_value += 1;
    state.flags_from_value(new_value.0);
    state.af = (1 << State::A_FLAG_BIT) & (value.0 ^ new_value.0) != 0;
    new_value
}

/// Decrement a register and set the appropriate flags
#[inline]
pub(crate) fn dec(state: &mut State, get_register: impl Fn(&mut State) -> &mut Wrapping<u8>) -> u8 {
    //let register = register(state);
    let old_value = *get_register(state);
    let new_value = dec_value(state, old_value);
    *get_register(state) = new_value;
    5
}

/// Decrement a value and set the appropriate flags on the state.
///
/// Return the incremented value.
#[inline]
pub(crate) fn dec_value(state: &mut State, value: Wrapping<u8>) -> Wrapping<u8> {
    let mut new_value = value;
    new_value -= 1;
    state.flags_from_value(new_value.0);
    state.af = new_value.0 & 0xf != 0xf;
    new_value
}

/// Add a value to HL, updating the carry flag
#[inline]
pub(crate) fn dad_value(state: &mut State, value: u16) {
    let (hl, carry) = state.hl().overflowing_add(value);
    state.set_hl(hl);
    state.cf = carry;
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
pub(crate) fn daa(state: &mut State) {
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
}
