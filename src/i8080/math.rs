//! Math instructions

use crate::Fetch;
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
    state.set_hf(0x10 & (value.0 ^ new_value.0) != 0);
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
pub(crate) fn dec(state: &mut State, get_register: impl Fn(&mut State) -> &mut Wrapping<u8>) -> u8 {
    let old_value = *get_register(state);
    let new_value = dec_value(state, old_value);
    *get_register(state) = new_value;
    5
}

/// Decrement a value and set the appropriate flags on the state.
///
/// Return the decremented value.
#[inline]
fn dec_value(state: &mut State, value: Wrapping<u8>) -> Wrapping<u8> {
    let mut new_value = value;
    new_value -= 1;
    state.flags_from_value(new_value.0);
    state.af = new_value.0 & 0xf != 0xf;
    new_value
}

/// Decrements the value in memory pointed by HL
pub(crate) fn dec_mem(state: &mut State, memory: &mut impl Memory) -> u8 {
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
    let (hl, carry) = state.get_hl().overflowing_add(register(state));
    state.set_hl(hl);
    state.cf = carry;
    10
}

/// Check if the operation `a + b = c` had a carry on the specified bit.
#[inline]
pub(crate) fn check_carry(bit: u32, a: u16, b: u16, sum: u16) -> bool {
    (sum ^ a ^ b) & (1 << bit) != 0
}

/// Add a value and a carry to A, updating the flags
#[inline]
pub(crate) fn add_value(state: &mut impl I8080FamilyState, value: u8, carry: bool) {
    let a = state.get_a().0 as u16;
    let value = value as u16;
    let mut sum = Wrapping(a);
    sum += value;
    sum += carry as u16;
    state.set_nf(false);
    state.set_cf(sum.0 & (1 << 8) != 0);
    state.set_hf(check_carry(4, a, value, sum.0));
    state.overflow_flag(check_carry(7, a, value, sum.0) != state.get_cf());
    state.get_a_mut().0 = sum.0 as u8;
    state.flags_from_accumulator();
}

/// Add the value of a register to A, updating the flags
#[inline]
pub(crate) fn add_r<S: I8080FamilyState>(
    state: &mut S,
    get_register: fn(&S) -> Wrapping<u8>,
) -> u8 {
    add_value(state, get_register(state).0, false);
    4
}

/// Add the value pointed by HL to A, updating the flags
#[inline]
pub(crate) fn add_mem<S: I8080FamilyState>(state: &mut S, memory: &impl Memory) -> u8 {
    add_value(state, memory.load(state.get_hl()), false);
    7
}

/// Add the value of a register and the carry flag to A, updating the flags
#[inline]
pub(crate) fn adc_r<S: I8080FamilyState>(
    state: &mut S,
    get_register: fn(&S) -> Wrapping<u8>,
) -> u8 {
    add_value(state, get_register(state).0, state.get_cf());
    4
}

/// Add the value pointed by HL and the carry flag to A, updating the flags
#[inline]
pub(crate) fn adc_mem<S: I8080FamilyState>(state: &mut S, memory: &impl Memory) -> u8 {
    let carry = state.get_cf();
    add_value(state, memory.load(state.get_hl()), carry);
    7
}

/// Subtract a value and a borrow from A, updating the flags
#[inline]
pub(crate) fn sub_value(state: &mut State, value: u8, carry: bool) {
    add_value(state, !value, !carry);
    state.cf = !state.cf;
}

/// Subtract the value of a register from A, updating the flags
#[inline]
pub(crate) fn sub_r(state: &mut State, get_register: fn(&State) -> Wrapping<u8>) -> u8 {
    let value = get_register(state).0;
    sub_value(state, value, false);
    4
}

/// Subtract the value pointed by HL from A, updating the flags
#[inline]
pub(crate) fn sub_mem(state: &mut State, memory: &impl Memory) -> u8 {
    sub_value(state, memory.load(state.get_hl()), false);
    7
}

/// Subtract the value of a register and the carry flag from A, updating the flags
#[inline]
pub(crate) fn sbb_r(state: &mut State, get_register: fn(&State) -> Wrapping<u8>) -> u8 {
    let value = get_register(state).0;
    let carry = state.cf;
    sub_value(state, value, carry);
    4
}

/// Subtract the value pointed by HL and the carry flag from A, updating the flags
#[inline]
pub(crate) fn sbb_mem(state: &mut State, memory: &impl Memory) -> u8 {
    sub_value(state, memory.load(state.get_hl()), state.cf);
    7
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

/// Perform a logical AND between the register and the accumulator, updating the flags
#[inline]
pub(crate) fn ana_r(state: &mut State, get_register: fn(&State) -> Wrapping<u8>) -> u8 {
    and_value(state, get_register(state).0);
    4
}

/// Perform a logical AND between the value pointed by HL and the accumulator, updating the flags
#[inline]
pub(crate) fn ana_mem(state: &mut State, memory: &impl Memory) -> u8 {
    let value = memory.load(state.get_hl());
    and_value(state, value);
    7
}

/// Perform a logical OR between the value and the accumulator, updating the flags
#[inline]
pub(crate) fn or_value(state: &mut impl I8080FamilyState, value: u8) {
    state.get_a_mut().0 |= value;
    state.flags_from_accumulator();
    state.parity_from_accumulator();
    state.set_cf(false);
    state.set_hf(false);
    state.set_nf(false);
}

/// Perform a logical OR between the register and the accumulator, updating the flags
#[inline]
pub(crate) fn ora_r<S: I8080FamilyState>(
    state: &mut S,
    get_register: fn(&S) -> Wrapping<u8>,
) -> u8 {
    or_value(state, get_register(state).0);
    4
}

/// Perform a logical OR between the value pointed by HL and the accumulator, updating the flags
#[inline]
pub(crate) fn ora_mem(state: &mut State, memory: &impl Memory) -> u8 {
    let value = memory.load(state.get_hl());
    or_value(state, value);
    7
}

/// Perform a logical XOR between the value and the accumulator, updating the flags
#[inline]
pub(crate) fn xor_value(state: &mut impl I8080FamilyState, value: u8) {
    state.get_a_mut().0 ^= value;
    state.flags_from_accumulator();
    state.parity_from_accumulator();
    state.set_cf(false);
    state.set_hf(false);
    state.set_nf(false);
}

/// Perform a logical XOR between the register and the accumulator, updating the flags
#[inline]
pub(crate) fn xra_r<S: I8080FamilyState>(
    state: &mut S,
    get_register: fn(&S) -> Wrapping<u8>,
) -> u8 {
    xor_value(state, get_register(state).0);
    4
}

/// Perform a logical XOR between the value pointed by HL and the accumulator, updating the flags
#[inline]
pub(crate) fn xra_mem(state: &mut State, memory: &impl Memory) -> u8 {
    let value = memory.load(state.get_hl());
    xor_value(state, value);
    7
}

/// Compare a value with the accumulator
#[inline]
pub(crate) fn cmp_value(state: &mut State, value: u8) {
    let (diff, carry) = state.a.0.overflowing_sub(value);
    state.cf = carry;
    state.flags_from_value(diff);
    state.af = (state.a.0 ^ diff ^ value) & 0x10 == 0;
}

/// Compare a register with the accumulator
#[inline]
pub(crate) fn cmp_r(state: &mut State, get_register: fn(&State) -> Wrapping<u8>) -> u8 {
    cmp_value(state, get_register(state).0);
    4
}

/// Compare a value in memory with the accumulator
#[inline]
pub(crate) fn cmp_mem(state: &mut State, memory: &impl Memory) -> u8 {
    let value = memory.load(state.get_hl());
    cmp_value(state, value);
    7
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

/// Set the carry flag
pub(crate) fn stc(state: &mut State) -> u8 {
    state.cf = true;
    4
}

/// Complement of A
pub(crate) fn cma(state: &mut State) -> u8 {
    state.a = !state.a;
    4
}

/// Invert the carry flag
pub(crate) fn cmc(state: &mut State) -> u8 {
    state.cf = !state.cf;
    4
}

/// Perform an ALU operation on an immediate value
pub(crate) fn alu_imm<S: I8080FamilyState>(
    state: &mut S,
    operation: fn(&mut S, u8),
    memory: &impl Memory,
) -> u8 {
    let value = state.fetch_byte(memory);
    operation(state, value);
    7
}
