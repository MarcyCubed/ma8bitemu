//! Instructions that control the program flow and relatives.

use crate::memory::Memory;
use crate::state::State;

/// Perform a return if the condition is true.
///
/// Return `cond`,
#[inline]
pub(crate) fn ret_if(state: &mut State, memory: &impl Memory, cond: bool) -> bool {
    if cond {
        ret(state, memory)
    }
    cond
}

/// Perform a return
#[inline]
pub(crate) fn ret(state: &mut State, memory: &impl Memory) {
    let pc = pop(state, memory);
    state.pc.0 = pc;
}

/// Pop a value from the stack
#[inline]
pub(crate) fn pop(state: &mut State, memory: &impl Memory) -> u16 {
    let data_0 = memory.load(state.sp.0);
    state.sp += 1;
    let data_1 = memory.load(state.sp.0);
    state.sp += 1;
    u16::from_le_bytes([data_0, data_1])
}

/// Push a value to the stack
#[inline]
pub(crate) fn push(state: &mut State, memory: &mut impl Memory, value: u16) {
    let value = value.to_le_bytes();
    state.sp -= 1;
    memory.store(state.sp.0, value[1]);
    state.sp -= 1;
    memory.store(state.sp.0, value[0]);
}

/// Call a function in a given address if the condition is true
///
/// Return `cond`,
#[inline]
pub(crate) fn call_if(
    state: &mut State,
    memory: &mut impl Memory,
    cond: bool,
    address: u16,
) -> bool {
    if cond {
        call(state, memory, address)
    }
    cond
}

/// Call a function
#[inline]
pub(crate) fn call(state: &mut State, memory: &mut impl Memory, address: u16) {
    let pc = state.pc.0;
    push(state, memory, pc);
    state.pc.0 = address;
}
