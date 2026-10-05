use crate::i8080::I8080FamilyEmulator;
use crate::memory::Memory;
use core::num::Wrapping;

/// Sets a 16-bit register with an immediate value
pub(crate) fn lxi<E: I8080FamilyEmulator, M: Memory>(
    state: &mut E,
    memory: &mut M,
    set_register: fn(&mut E, u16),
) -> u8 {
    let d = state.fetch_word(memory);
    set_register(state, d);
    10
}

/// Store the value in the register A into the memory pointed by a 16-bit register
pub(crate) fn stax<S: I8080FamilyEmulator, M: Memory>(
    state: &mut S,
    memory: &mut M,
    address_fn: impl Fn(&S) -> u16,
) -> u8 {
    let address = address_fn(state);
    state.set_memptr(u16::from_le_bytes([
        address.wrapping_add(1) as u8,
        state.get_a().0,
    ]));
    memory.store(address, state.get_a().0);
    7
}

/// Store the value of a 16-bit register in the immediate memory address
pub(crate) fn shld_rr<S: I8080FamilyEmulator>(
    state: &mut S,
    memory: &mut impl Memory,
    register: impl Fn(&S) -> u16,
) -> u8 {
    let address = state.fetch_word(memory);
    state.set_memptr(address.wrapping_add(1));
    memory.store_16(address, register(state));
    16
}

/// Store the value of A in the immediate memory address
pub(crate) fn sta(state: &mut impl I8080FamilyEmulator, memory: &mut impl Memory) -> u8 {
    let address = state.fetch_word(memory);
    state.set_memptr(u16::from_le_bytes([
        address.wrapping_add(1) as u8,
        state.get_a().0,
    ]));
    memory.store(address, state.get_a().0);
    13
}

/// Store an immediate value in a register
pub(crate) fn mvi<S: I8080FamilyEmulator, M: Memory>(
    state: &mut S,
    memory: &mut M,
    register: fn(&mut S) -> &mut Wrapping<u8>,
) -> u8 {
    let d = state.fetch_byte(memory);
    register(state).0 = d;
    7
}

/// Store an immediate value in the memory pointed by HL
pub(crate) fn mvi_mem<S: I8080FamilyEmulator, M: Memory>(state: &mut S, memory: &mut M) -> u8 {
    let address = state.get_hl();
    let d = state.fetch_byte(memory);
    memory.store(address, d);
    10
}

/// Load the value in the address pointed by the 16-bit register and put it in A.
pub(crate) fn ldax<S: I8080FamilyEmulator, M: Memory>(
    state: &mut S,
    memory: &M,
    address_fn: impl Fn(&S) -> u16,
) -> u8 {
    let address = address_fn(state);
    let value = memory.load(address);
    state.set_memptr(address.wrapping_add(1));
    state.get_a_mut().0 = value;
    7
}

/// Load the value pointed by the immediate address into the 16-bit register
pub(crate) fn lhld_rr<S: I8080FamilyEmulator>(
    state: &mut S,
    memory: &mut impl Memory,
    setter: impl Fn(&mut S, u16),
) -> u8 {
    let address = state.fetch_word(memory);
    state.set_memptr(address.wrapping_add(1));
    setter(state, memory.load_16(address));
    16
}

/// Load the value pointed by the immediate address into A
pub(crate) fn lda(state: &mut impl I8080FamilyEmulator, memory: &mut impl Memory) -> u8 {
    let address = state.fetch_word(memory);
    state.set_memptr(address.wrapping_add(1));
    state.get_a_mut().0 = memory.load(address);
    16
}

/// Move the value in the source register to the destination register
pub(crate) fn mov<S: I8080FamilyEmulator>(
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
pub(crate) fn mov_r_mem<S: I8080FamilyEmulator>(
    state: &mut S,
    register: fn(&mut S) -> &mut Wrapping<u8>,
    memory: &mut impl Memory,
) -> u8 {
    let value = memory.load(state.get_hl());
    register(state).0 = value;
    7
}

/// Move the value in the register to the memory pointed by HL
pub(crate) fn mov_mem_r<S: I8080FamilyEmulator>(
    state: &S,
    memory: &mut impl Memory,
    register: fn(&S) -> Wrapping<u8>,
) -> u8 {
    memory.store(state.get_hl(), register(state).0);
    7
}

/// Exchanges HL with the top of the stack
pub(crate) fn xthl(
    emulator: &mut impl I8080FamilyEmulator,
    memory: &mut impl Memory,
    clock_cycles: u8,
) -> u8 {
    let sp_data = memory.load_16(emulator.get_sp().0);
    memory.store_16(emulator.get_sp().0, emulator.get_hl());
    emulator.set_hl(sp_data);
    clock_cycles
}

/// Move HL to SP
pub(crate) fn sphl(state: &mut impl I8080FamilyEmulator, cycles: u8) -> u8 {
    state.get_sp_mut().0 = state.get_hl();
    cycles
}

/// Exchanges DE and HL
pub(crate) fn xchg(state: &mut impl I8080FamilyEmulator) -> u8 {
    let d = state.get_d();
    *state.get_d_mut() = state.get_h();
    *state.get_h_mut() = d;
    let e = state.get_e();
    *state.get_e_mut() = state.get_l();
    *state.get_l_mut() = e;
    4
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
