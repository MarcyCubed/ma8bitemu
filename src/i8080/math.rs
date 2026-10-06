//! Math instructions

use crate::i8080::{Emulator, I8080FamilyEmulator};
use crate::memory::Memory;
use core::num::Wrapping;

/// Increment a 16-bit register
pub(crate) fn inx<E: I8080FamilyEmulator>(
    emulator: &mut E,
    getter: fn(&E) -> u16,
    setter: fn(&mut E, u16),
    clock_cycles: u8,
) -> u8 {
    let inc = getter(emulator).wrapping_add(1);
    setter(emulator, inc);
    clock_cycles
}

/// Decrement a 16-bit register
pub(crate) fn dcx<E: I8080FamilyEmulator>(
    emulator: &mut E,
    getter: fn(&E) -> u16,
    setter: fn(&mut E, u16),
    clock_cycles: u8,
) -> u8 {
    let inc = getter(emulator).wrapping_sub(1);
    setter(emulator, inc);
    clock_cycles
}

/// Increment a register and set the appropriate flags
#[inline]
pub(crate) fn inc<E: I8080FamilyEmulator>(
    emulator: &mut E,
    get_register: impl Fn(&mut E) -> &mut Wrapping<u8>,
    clock_cycles: u8,
) -> u8 {
    let old_value = *get_register(emulator);
    let new_value = inc_value(emulator, old_value);
    *get_register(emulator) = new_value;
    clock_cycles
}

/// Increment a value and set the appropriate flags.
///
/// Return the incremented value.
#[inline]
pub(crate) fn inc_value(
    emulator: &mut impl I8080FamilyEmulator,
    value: Wrapping<u8>,
) -> Wrapping<u8> {
    let mut new_value = value;
    new_value += 1;
    emulator.flags_from_value(new_value.0);
    emulator.overflow_flag(new_value.0 == 1 << 7);
    emulator.set_hf(0x10 & (value.0 ^ new_value.0) != 0);
    emulator.set_nf(false);
    new_value
}

/// Increments the value in memory pointed by HL
pub(crate) fn inc_mem<E: I8080FamilyEmulator>(
    emulator: &mut E,
    memory: &mut impl Memory,
    clock_cycles: u8,
) -> u8 {
    let address = emulator.get_hl();
    let value = inc_value(emulator, Wrapping(memory.load(address)));
    memory.store(address, value.0);
    clock_cycles
}

/// Decrement a register and set the appropriate flags
#[inline]
pub(crate) fn dec(
    emulator: &mut Emulator,
    get_register: impl Fn(&mut Emulator) -> &mut Wrapping<u8>,
) -> u8 {
    let old_value = *get_register(emulator);
    let new_value = dec_value(emulator, old_value);
    *get_register(emulator) = new_value;
    5
}

/// Decrement a value and set the appropriate flags.
///
/// Return the decremented value.
#[inline]
fn dec_value(emulator: &mut Emulator, value: Wrapping<u8>) -> Wrapping<u8> {
    let mut new_value = value;
    new_value -= 1;
    emulator.flags_from_value(new_value.0);
    emulator.af = new_value.0 & 0xf != 0xf;
    new_value
}

/// Decrements the value in memory pointed by HL
pub(crate) fn dec_mem(emulator: &mut Emulator, memory: &mut impl Memory) -> u8 {
    let address = emulator.get_hl();
    let value = dec_value(emulator, Wrapping(memory.load(address)));
    memory.store(address, value.0);
    10
}

/// Rotate the accumulator left and copy the original most significant bit to the carry flag
pub(super) fn rlc(emulator: &mut Emulator) -> u8 {
    emulator.a.0 = emulator.a.0.rotate_left(1);
    emulator.cf = emulator.a.0 & 0x1 != 0;
    4
}

/// Rotate the 9-bit value composed by the C flag and the accumulator to the left
pub(crate) fn ral(emulator: &mut impl I8080FamilyEmulator) -> u8 {
    let new_c_flag = emulator.get_a().0 & (1 << 7) != 0;
    let new_a = emulator.get_a().0 << 1 | emulator.get_cf() as u8;
    *emulator.get_a_mut() = Wrapping(new_a);
    emulator.set_cf(new_c_flag);
    4
}

/// Rotate the accumulator right and copy the original least significant bit to the carry flag
pub(super) fn rrc(emulator: &mut Emulator) -> u8 {
    emulator.cf = emulator.get_a().0 & 0x1 != 0;
    emulator.a.0 = emulator.a.0.rotate_right(1);
    4
}

/// Rotate the 9-bit value composed by the C flag and the accumulator to the right
pub(crate) fn rar(emulator: &mut impl I8080FamilyEmulator) -> u8 {
    let new_c_flag = emulator.get_a().0 & 0x1 != 0;
    emulator.get_a_mut().0 = (emulator.get_a().0 >> 1) | ((emulator.get_cf() as u8) << 7);
    emulator.set_cf(new_c_flag);
    4
}

/// Add a value to HL, updating the carry flag
#[inline]
pub(crate) fn dad(emulator: &mut Emulator, register: fn(&Emulator) -> u16) -> u8 {
    let (hl, carry) = emulator.get_hl().overflowing_add(register(emulator));
    emulator.set_hl(hl);
    emulator.cf = carry;
    10
}

/// Check if the operation `a + b = c` had a carry on the specified bit.
#[inline]
pub(crate) fn check_carry(bit: u32, a: u16, b: u16, sum: u16) -> bool {
    (sum ^ a ^ b) & (1 << bit) != 0
}

/// Add a value and a carry to A, updating the flags
#[inline]
pub(crate) fn add_value(emulator: &mut impl I8080FamilyEmulator, value: u8, carry: bool) {
    let a = emulator.get_a().0 as u16;
    let value = value as u16;
    let mut sum = Wrapping(a);
    sum += value;
    sum += carry as u16;
    emulator.set_nf(false);
    emulator.set_cf(sum.0 & (1 << 8) != 0);
    emulator.set_hf(check_carry(4, a, value, sum.0));
    emulator.overflow_flag(check_carry(7, a, value, sum.0) != emulator.get_cf());
    emulator.get_a_mut().0 = sum.0 as u8;
    emulator.flags_from_accumulator();
}

/// Add the value of a register to A, updating the flags
#[inline]
pub(crate) fn add_r<E: I8080FamilyEmulator>(
    emulator: &mut E,
    get_register: fn(&E) -> Wrapping<u8>,
) -> u8 {
    add_value(emulator, get_register(emulator).0, false);
    4
}

/// Add the value pointed by HL to A, updating the flags
#[inline]
pub(crate) fn add_mem<E: I8080FamilyEmulator>(emulator: &mut E, memory: &impl Memory) -> u8 {
    add_value(emulator, memory.load(emulator.get_hl()), false);
    7
}

/// Add the value of a register and the carry flag to A, updating the flags
#[inline]
pub(crate) fn adc_r<E: I8080FamilyEmulator>(
    emulator: &mut E,
    get_register: fn(&E) -> Wrapping<u8>,
) -> u8 {
    add_value(emulator, get_register(emulator).0, emulator.get_cf());
    4
}

/// Add the value pointed by HL and the carry flag to A, updating the flags
#[inline]
pub(crate) fn adc_mem<E: I8080FamilyEmulator>(emulator: &mut E, memory: &impl Memory) -> u8 {
    let carry = emulator.get_cf();
    add_value(emulator, memory.load(emulator.get_hl()), carry);
    7
}

/// Subtract a value and a borrow from A, updating the flags
#[inline]
pub(crate) fn sub_value(emulator: &mut Emulator, value: u8, carry: bool) {
    add_value(emulator, !value, !carry);
    emulator.cf = !emulator.cf;
}

/// Subtract the value of a register from A, updating the flags
#[inline]
pub(crate) fn sub_r(emulator: &mut Emulator, get_register: fn(&Emulator) -> Wrapping<u8>) -> u8 {
    let value = get_register(emulator).0;
    sub_value(emulator, value, false);
    4
}

/// Subtract the value pointed by HL from A, updating the flags
#[inline]
pub(crate) fn sub_mem(emulator: &mut Emulator, memory: &impl Memory) -> u8 {
    sub_value(emulator, memory.load(emulator.get_hl()), false);
    7
}

/// Subtract the value of a register and the carry flag from A, updating the flags
#[inline]
pub(crate) fn sbb_r(emulator: &mut Emulator, get_register: fn(&Emulator) -> Wrapping<u8>) -> u8 {
    let value = get_register(emulator).0;
    let carry = emulator.cf;
    sub_value(emulator, value, carry);
    4
}

/// Subtract the value pointed by HL and the carry flag from A, updating the flags
#[inline]
pub(crate) fn sbb_mem(emulator: &mut Emulator, memory: &impl Memory) -> u8 {
    sub_value(emulator, memory.load(emulator.get_hl()), emulator.cf);
    7
}

/// Perform a logical AND between the value and the accumulator, updating the flags
#[inline]
pub(crate) fn and_value(emulator: &mut Emulator, value: u8) {
    let a = emulator.a.0;
    emulator.a.0 &= value;
    emulator.flags_from_accumulator();
    emulator.cf = false;
    // What were the Intel engineers smoking?!
    emulator.af = (a | value) & 0b1000 != 0;
}

/// Perform a logical AND between the register and the accumulator, updating the flags
#[inline]
pub(crate) fn ana_r(emulator: &mut Emulator, get_register: fn(&Emulator) -> Wrapping<u8>) -> u8 {
    and_value(emulator, get_register(emulator).0);
    4
}

/// Perform a logical AND between the value pointed by HL and the accumulator, updating the flags
#[inline]
pub(crate) fn ana_mem(emulator: &mut Emulator, memory: &impl Memory) -> u8 {
    let value = memory.load(emulator.get_hl());
    and_value(emulator, value);
    7
}

/// Perform a logical OR between the value and the accumulator, updating the flags
#[inline]
pub(crate) fn or_value(emulator: &mut impl I8080FamilyEmulator, value: u8) {
    emulator.get_a_mut().0 |= value;
    emulator.flags_from_accumulator();
    emulator.parity_from_accumulator();
    emulator.set_cf(false);
    emulator.set_hf(false);
    emulator.set_nf(false);
}

/// Perform a logical OR between the register and the accumulator, updating the flags
#[inline]
pub(crate) fn ora_r<E: I8080FamilyEmulator>(
    emulator: &mut E,
    get_register: fn(&E) -> Wrapping<u8>,
) -> u8 {
    or_value(emulator, get_register(emulator).0);
    4
}

/// Perform a logical OR between the value pointed by HL and the accumulator, updating the flags
#[inline]
pub(crate) fn ora_mem(emulator: &mut impl I8080FamilyEmulator, memory: &impl Memory) -> u8 {
    let value = memory.load(emulator.get_hl());
    or_value(emulator, value);
    7
}

/// Perform a logical XOR between the value and the accumulator, updating the flags
#[inline]
pub(crate) fn xor_value(emulator: &mut impl I8080FamilyEmulator, value: u8) {
    emulator.get_a_mut().0 ^= value;
    emulator.flags_from_accumulator();
    emulator.parity_from_accumulator();
    emulator.set_cf(false);
    emulator.set_hf(false);
    emulator.set_nf(false);
}

/// Perform a logical XOR between the register and the accumulator, updating the flags
#[inline]
pub(crate) fn xra_r<E: I8080FamilyEmulator>(
    emulator: &mut E,
    get_register: fn(&E) -> Wrapping<u8>,
) -> u8 {
    xor_value(emulator, get_register(emulator).0);
    4
}

/// Perform a logical XOR between the value pointed by HL and the accumulator, updating the flags
#[inline]
pub(crate) fn xra_mem(emulator: &mut impl I8080FamilyEmulator, memory: &impl Memory) -> u8 {
    let value = memory.load(emulator.get_hl());
    xor_value(emulator, value);
    7
}

/// Compare a value with the accumulator
#[inline]
pub(crate) fn cmp_value(emulator: &mut Emulator, value: u8) {
    let (diff, carry) = emulator.a.0.overflowing_sub(value);
    emulator.cf = carry;
    emulator.flags_from_value(diff);
    emulator.af = (emulator.a.0 ^ diff ^ value) & 0x10 == 0;
}

/// Compare a register with the accumulator
#[inline]
pub(crate) fn cmp_r(emulator: &mut Emulator, get_register: fn(&Emulator) -> Wrapping<u8>) -> u8 {
    cmp_value(emulator, get_register(emulator).0);
    4
}

/// Compare a value in memory with the accumulator
#[inline]
pub(crate) fn cmp_mem(emulator: &mut Emulator, memory: &impl Memory) -> u8 {
    let value = memory.load(emulator.get_hl());
    cmp_value(emulator, value);
    7
}

/// Adjust the accumulator to BCD
pub(crate) fn daa(emulator: &mut Emulator) -> u8 {
    let mut diff = 0;
    if emulator.a.0 & 0xf > 0x9 || emulator.af {
        diff += 0x06
    }
    if emulator.a.0 > 0x99 || emulator.cf {
        diff += 0x60;
    }
    let old_cf = emulator.cf;
    add_value(emulator, diff, false);
    emulator.cf |= old_cf;
    4
}

/// Set the carry flag
pub(crate) fn stc(emulator: &mut Emulator) -> u8 {
    emulator.cf = true;
    4
}

/// Complement of A
pub(crate) fn cma(emulator: &mut Emulator) -> u8 {
    emulator.a = !emulator.a;
    4
}

/// Invert the carry flag
pub(crate) fn cmc(emulator: &mut Emulator) -> u8 {
    emulator.cf = !emulator.cf;
    4
}

/// Perform an ALU operation on an immediate value
pub(crate) fn alu_imm<E: I8080FamilyEmulator>(
    emulator: &mut E,
    operation: fn(&mut E, u8),
    memory: &impl Memory,
) -> u8 {
    let value = emulator.fetch_byte(memory);
    operation(emulator, value);
    7
}
