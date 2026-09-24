use crate::Fetch;
use crate::i8080::I8080FamilyState;
use crate::memory::Memory;
use core::num::Wrapping;

/// Sets a 16-bit register with an immediate value
pub(crate) fn lxi<S: I8080FamilyState, M: Memory>(
    state: &mut S,
    memory: &mut M,
    set_register: fn(&mut S, u16),
) -> u8 {
    let d = state.fetch_word(memory);
    set_register(state, d);
    10
}

/// Store the value in the register A into the memory pointed by a 16-bit register
pub(crate) fn stax<S: I8080FamilyState, M: Memory>(
    state: &S,
    memory: &mut M,
    address_fn: impl Fn(&S) -> u16,
) -> u8 {
    memory.store(address_fn(state), state.get_a().0);
    7
}

/// Store the value of HL in the immediate memory address
pub(crate) fn shld(state: &mut impl I8080FamilyState, memory: &mut impl Memory) -> u8 {
    let address_0 = state.fetch_word(memory);
    let address_1 = address_0.wrapping_add(1);
    memory.store(address_0, state.get_l().0);
    memory.store(address_1, state.get_h().0);
    16
}

/// Store the value of A in the immediate memory address
pub(crate) fn sta(state: &mut impl I8080FamilyState, memory: &mut impl Memory) -> u8 {
    let address = state.fetch_word(memory);
    memory.store(address, state.get_a().0);
    13
}

/// Store an immediate value in a register
pub(crate) fn mvi<S: I8080FamilyState, M: Memory>(
    state: &mut S,
    memory: &mut M,
    register: fn(&mut S) -> &mut Wrapping<u8>,
) -> u8 {
    let d = state.fetch_byte(memory);
    register(state).0 = d;
    7
}

/// Store an immediate value in the memory pointed by HL
pub(crate) fn mvi_mem<S: I8080FamilyState, M: Memory>(state: &mut S, memory: &mut M) -> u8 {
    let address = state.get_hl();
    let d = state.fetch_byte(memory);
    memory.store(address, d);
    10
}

/// Load the value in the address pointed by the 16-bit register and put it in A.
pub(crate) fn ldax<S: I8080FamilyState, M: Memory>(
    state: &mut S,
    memory: &M,
    address_fn: impl Fn(&S) -> u16,
) -> u8 {
    let value = memory.load(address_fn(state));
    state.get_a_mut().0 = value;
    7
}

/// Load the value pointed by the immediate address into HL
pub(crate) fn lhld(state: &mut impl I8080FamilyState, memory: &mut impl Memory) -> u8 {
    let address = state.fetch_word(memory);
    state.get_l_mut().0 = memory.load(address);
    state.get_h_mut().0 = memory.load(address.wrapping_add(1));
    16
}

/// Load the value pointed by the immediate address into A
pub(crate) fn lda(state: &mut impl I8080FamilyState, memory: &mut impl Memory) -> u8 {
    let address = state.fetch_word(memory);
    state.get_a_mut().0 = memory.load(address);
    16
}

/// Move the value in the source register to the destination register
pub(crate) fn mov<S: I8080FamilyState>(
    state: &mut S,
    dst: fn(&mut S) -> &mut Wrapping<u8>,
    src: fn(&S) -> Wrapping<u8>,
    clock_cycles: u8,
) -> u8 {
    let value = src(state);
    *dst(state) = value;
    clock_cycles
}

/// Move the value in the memory pointed by HL to the register
pub(crate) fn mov_r_mem<S: I8080FamilyState>(
    state: &mut S,
    register: fn(&mut S) -> &mut Wrapping<u8>,
    memory: &mut impl Memory,
) -> u8 {
    let value = memory.load(state.get_hl());
    register(state).0 = value;
    7
}

/// Move the value in the register to the memory pointed by HL
pub(crate) fn mov_mem_r<S: I8080FamilyState>(
    state: &S,
    memory: &mut impl Memory,
    register: fn(&S) -> Wrapping<u8>,
) -> u8 {
    memory.store(state.get_hl(), register(state).0);
    7
}

/*
macro_rules! stax {
    ($state: expr, $register: ident, $memory: expr) => {{
        $memory.store($state.$register(), $state.a.0);
        7
    }};
}

pub(crate) use stax;
*/
