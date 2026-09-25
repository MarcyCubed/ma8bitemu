//! Instructions that control the program flow and relatives.

use crate::Fetch;
use crate::i8080::I8080FamilyState;
use crate::memory::Memory;

/// Perform a conditional return.
#[inline]
pub(crate) fn ret_cond<S: I8080FamilyState, F: Fn(&S) -> bool>(
    state: &mut S,
    memory: &impl Memory,
    cond: F,
) -> u8 {
    if cond(state) {
        ret(state, memory);
        11
    } else {
        5
    }
}

/// Perform a return
#[inline]
pub(crate) fn ret(state: &mut impl I8080FamilyState, memory: &impl Memory) -> u8 {
    let pc = pop_stack(state, memory);
    state.set_pc(pc);
    10
}

/// Pop a value from the stack
#[inline]
pub(crate) fn pop_stack(state: &mut impl I8080FamilyState, memory: &impl Memory) -> u16 {
    let data_0 = memory.load(state.get_sp().0);
    *state.get_sp_mut() += 1;
    let data_1 = memory.load(state.get_sp().0);
    *state.get_sp_mut() += 1;
    u16::from_le_bytes([data_0, data_1])
}

/// Pop a value from the stack into a register
#[inline]
pub(crate) fn pop<S: I8080FamilyState>(
    state: &mut S,
    memory: &impl Memory,
    set_register: fn(&mut S, u16),
) -> u8 {
    let value = pop_stack(state, memory);
    set_register(state, value);
    10
}

/// Push a value to the stack
#[inline]
pub(crate) fn push_stack(state: &mut impl I8080FamilyState, memory: &mut impl Memory, value: u16) {
    let value = value.to_le_bytes();
    *state.get_sp_mut() -= 1;
    memory.store(state.get_sp().0, value[1]);
    *state.get_sp_mut() -= 1;
    memory.store(state.get_sp().0, value[0]);
}

/// Pop a register to the stack
#[inline]
pub(crate) fn push<S: I8080FamilyState>(
    state: &mut S,
    memory: &mut impl Memory,
    get_register: fn(&S) -> u16,
) -> u8 {
    let value = get_register(state);
    push_stack(state, memory, value);
    11
}

/// Call a function in the immediate address if the condition is true.
#[inline]
pub(crate) fn call_cond_nn<S: I8080FamilyState, F: Fn(&S) -> bool>(
    state: &mut S,
    memory: &mut impl Memory,
    cond: F,
    clock_cycles_if_true: u8,
    clock_cycles_if_false: u8,
) -> u8 {
    let address = state.fetch_word(memory);
    if cond(state) {
        call_address(state, memory, address);
        clock_cycles_if_true
    } else {
        clock_cycles_if_false
    }
}

/// Call a function
#[inline]
pub(crate) fn call_address(
    state: &mut impl I8080FamilyState,
    memory: &mut impl Memory,
    address: u16,
) {
    let pc = state.get_pc().0;
    push_stack(state, memory, pc);
    state.get_pc_mut().0 = address;
}

/// Perform a conditional return to an immediate address.
#[inline]
pub(crate) fn jp_cond_nn<S: I8080FamilyState, F: Fn(&S) -> bool>(
    state: &mut S,
    memory: &impl Memory,
    cond: F,
) -> u8 {
    let address = state.fetch_word(memory);
    if cond(state) {
        state.set_pc(address);
    }
    10
}

/// RST instruction
pub(crate) fn rst(state: &mut impl I8080FamilyState, memory: &mut impl Memory, address: u16) -> u8 {
    call_address(state, memory, address);
    11
}

/// Jump to the address in HL
pub(crate) fn jp_hl(state: &mut impl I8080FamilyState, cycles: u8) -> u8 {
    state.get_pc_mut().0 = state.get_hl();
    cycles
}
