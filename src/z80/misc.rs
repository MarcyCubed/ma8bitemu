//! Miscellaneous instructions (prefix ED)

use crate::i8080::I8080FamilyState;
use crate::i8080::load::{lhld_rr, shld_rr};
use crate::memory::Memory;
use crate::z80::state::InterruptMode;
use crate::z80::z8080::add_hl_value;
use crate::z80::{State, z8080};
use crate::{ExecEffect, i8080};
use std::num::Wrapping;

/// Execute a prefix ED instruction
///
/// Return the number of clock cycles it took to execute the instruction and the result of the
/// execution.
pub(super) fn run_opcode(
    state: &mut State,
    opcode: u8,
    memory: &mut impl Memory,
) -> (u8, ExecEffect) {
    state.inc_r();

    match opcode {
        // IN a, (C)
        0x78 => {
            state.mem_ptr.0 = state.get_bc();
            state.mem_ptr += 1;
            in_r_bc(state, opcode)
        }
        // IN r, (C)
        0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x70 => in_r_bc(state, opcode),
        // OUT (C), a
        0x79 => {
            state.mem_ptr.0 = state.get_bc();
            state.mem_ptr += 1;
            out_value(state, state.a.0)
        }
        0x41 => out_value(state, state.b.0), // out (c), b
        0x49 => out_value(state, state.c.0), // out (c), c
        0x51 => out_value(state, state.d.0), // out (c), d
        0x59 => out_value(state, state.e.0), // out (c), e
        0x61 => out_value(state, state.h.0), // out (c), h
        0x69 => out_value(state, state.l.0), // out (c), l
        0x71 => out_value(state, 0),         // out (c), 0
        0x42 => sbc_hl_flags(state, state.get_bc(), state.cf), // sbc hl, bc
        0x52 => sbc_hl_flags(state, state.get_de(), state.cf), // sbc hl, de
        0x62 => sbc_hl_flags(state, state.get_hl(), state.cf), // sbc hl, hl
        0x72 => sbc_hl_flags(state, state.sp.0, state.cf), // sbc hl, sp
        0x4a => adc_hl_flags(state, state.get_bc(), state.cf), // adc hl, bc
        0x5a => adc_hl_flags(state, state.get_de(), state.cf), // adc hl, de
        0x6a => adc_hl_flags(state, state.get_hl(), state.cf), // adc hl, hl
        0x7a => adc_hl_flags(state, state.sp.0, state.cf), // adc hl, sp
        0x43 => {
            // ld (nn), bc
            shld_rr(state, memory, State::get_bc);
            (20, ExecEffect::Normal)
        }
        0x53 => {
            // ld (nn), de
            shld_rr(state, memory, State::get_de);
            (20, ExecEffect::Normal)
        }
        0x63 => {
            // ld (nn), hl
            shld_rr(state, memory, State::get_hl);
            (20, ExecEffect::Normal)
        }
        0x73 => {
            // ld (nn), sp
            shld_rr(state, memory, State::get_sp_u16);
            (20, ExecEffect::Normal)
        }
        0x44 => neg(state), // neg
        0x4b => {
            // ld bc, (nn)
            lhld_rr(state, memory, State::set_bc);
            (20, ExecEffect::Normal)
        }
        0x5b => {
            // ld de, (nn)
            lhld_rr(state, memory, State::set_de);
            (20, ExecEffect::Normal)
        }
        0x6b => {
            // ld hl, (nn)
            lhld_rr(state, memory, State::set_hl);
            (20, ExecEffect::Normal)
        }
        0x7b => {
            // ld sp, (nn)
            lhld_rr(state, memory, State::set_sp);
            (20, ExecEffect::Normal)
        }
        0x46 => im(state, InterruptMode::I8080),    // im 0
        0x56 => im(state, InterruptMode::Rst38h),   // im 1
        0x5e => im(state, InterruptMode::Vectored), // im 1
        0x47 => {
            // ld i, a
            state.i = state.a.0;
            (9, ExecEffect::Normal)
        }
        0x57 => ld_a_r(state, state.i), // ld a, i
        0x4f => {
            // ld r, a
            state.r = state.a;
            (9, ExecEffect::Normal)
        }
        0x5f => ld_a_r(state, state.r.0), // ld a, r
        0x67 => rrd(state, memory),       // rrd
        0x6f => rld(state, memory),       // rld
        0x45 => {
            // retn
            i8080::jump::ret(state, memory);
            state.iff1 = state.iff2;
            (14, ExecEffect::Normal)
        }
        0x4d => {
            // reti
            i8080::jump::ret(state, memory);
            (14, ExecEffect::Normal)
        }
        0xa0 => ldx(state, memory, 1),                           // ldi
        0xa8 => ldx(state, memory, -1),                          // ldd
        0xb0 => repeat(ldx, |s| s.pf, state, memory, 1),         // ldir
        0xb8 => repeat(ldx, |s| s.pf, state, memory, -1),        // lddr
        0xa1 => cpx(state, memory, 1),                           // cpi
        0xa9 => cpx(state, memory, -1),                          // cpd
        0xb1 => repeat(cpx, |s| s.pf || s.zf, state, memory, 1), // cpir
        0xb9 => repeat(cpx, |s| s.pf || s.zf, state, memory, -1), // cpdr
        0xa2 | 0xaa | 0xb2 | 0xba => todo!("Input needs to be reworked"),
        0xa3 => outx(state, memory, 1),                    // outi
        0xab => outx(state, memory, -1),                   // outd
        0xb3 => repeat(outx, |s| s.zf, state, memory, 1),  // otir
        0xbb => repeat(outx, |s| s.zf, state, memory, -1), // otdr

        // NOP is the default
        _ => (8, ExecEffect::Normal),
    }
}

/// IN instruction
fn in_r_bc(state: &mut State, opcode: u8) -> (u8, ExecEffect) {
    state.in_opcode = opcode;
    (
        12,
        ExecEffect::In {
            port: state.get_bc(),
        },
    )
}

/// Run an OUT instruction
fn out_value(state: &mut State, data: u8) -> (u8, ExecEffect) {
    (
        12,
        ExecEffect::Out {
            port: state.get_bc(),
            data,
        },
    )
}

/// Subtract the value and the carry from HL
fn sbc_hl_flags(state: &mut State, value: u16, carry: bool) -> (u8, ExecEffect) {
    add_hl_value(state, !value, !carry);
    state.nf = true;
    state.hf = !state.hf;
    state.cf = !state.cf;
    (15, ExecEffect::Normal)
}

/// Add the value and the carry to HL
fn adc_hl_flags(state: &mut State, value: u16, carry: bool) -> (u8, ExecEffect) {
    add_hl_value(state, value, carry);
    (15, ExecEffect::Normal)
}

/// - A
fn neg(state: &mut State) -> (u8, ExecEffect) {
    let a = state.a.0;
    state.a.0 = 0;
    z8080::sub_value(state, a, false);
    (8, ExecEffect::Normal)
}

/// Set the interrupt mode
fn im(state: &mut State, mode: InterruptMode) -> (u8, ExecEffect) {
    state.interrupt_mode = mode;
    (8, ExecEffect::Normal)
}

/// Load a special register into A
fn ld_a_r(state: &mut State, value: u8) -> (u8, ExecEffect) {
    state.a.0 = value;
    state.nf = false;
    state.hf = false;
    state.pf = state.iff2;
    state.flags_from_accumulator();
    (9, ExecEffect::Normal)
}

/// Perform a nybble rotate right between the memory pointed by `HL` and the least significant
/// nybble of `A`
fn rrd(state: &mut State, memory: &mut impl Memory) -> (u8, ExecEffect) {
    state.mem_ptr.0 = state.get_hl();
    state.mem_ptr += 1;
    let data = state.load_hl(memory);
    state.store_hl(memory, (data >> 4) | (state.a.0 << 4));
    state.a.0 = (state.a.0 & 0xf0) | (data & 0x0f);
    state.hf = false;
    state.nf = false;
    state.flags_from_accumulator();
    state.parity_from_accumulator();
    (18, ExecEffect::Normal)
}

/// Perform a nybble rotate left between the memory pointed by `HL` and the least significant nybble
/// of `A`
fn rld(state: &mut State, memory: &mut impl Memory) -> (u8, ExecEffect) {
    state.mem_ptr.0 = state.get_hl();
    state.mem_ptr += 1;
    let data = state.load_hl(memory);
    state.store_hl(memory, (data << 4) | (state.a.0 & 0x0f));
    state.a.0 = (state.a.0 & 0xf0) | (data >> 4);
    state.hf = false;
    state.nf = false;
    state.flags_from_accumulator();
    state.parity_from_accumulator();
    (18, ExecEffect::Normal)
}

/// LDI and LDD: Transfer data between two memory locations and increase or decrease the pointers
/// and decrease the counter
fn ldx(state: &mut State, memory: &mut impl Memory, offset: i16) -> (u8, ExecEffect) {
    let data = memory.load(state.get_hl());
    memory.store(state.get_de(), data);
    state.set_de(state.get_de().wrapping_add_signed(offset));
    state.set_hl(state.get_hl().wrapping_add_signed(offset));
    let bc = state.get_bc().wrapping_sub(1);
    state.set_bc(bc);
    state.hf = false;
    state.nf = false;
    state.pf = bc == 0;
    // XY flags are weird here
    let data = state.a + Wrapping(data);
    state.xf = data.0 & 0b1000 != 0;
    state.yf = data.0 & 0b10 != 0;
    (16, ExecEffect::Normal)
}

/// Implement one of the repeated instructions
fn repeat<M: Memory>(
    instruction: fn(&mut State, &mut M, i16) -> (u8, ExecEffect),
    exit_cond: impl Fn(&State) -> bool,
    state: &mut State,
    memory: &mut M,
    offset: i16,
) -> (u8, ExecEffect) {
    instruction(state, memory, offset);
    (
        if exit_cond(state) {
            16
        } else {
            state.pc -= 2;
            21
        },
        ExecEffect::Normal,
    )
}

/// Compare memory to accumulator, increase or decrease the pointer accordingly and decrease the
/// counter
fn cpx(state: &mut State, memory: &mut impl Memory, offset: i16) -> (u8, ExecEffect) {
    let data = memory.load(state.get_hl());
    state.set_hl(state.get_hl().wrapping_add_signed(offset));
    let carry = state.cf;
    let half_carry = state.hf;
    // Compare
    z8080::sub_flags(state, state.a.0, data, false);
    state.cf = carry;

    let bc = state.get_bc().wrapping_sub(1);
    state.set_bc(bc);
    state.pf = bc == 0;
    // XY flags are extra weird here
    let xy_source = data.wrapping_sub(half_carry as u8);
    state.xf = xy_source & 0b1000 != 0;
    state.yf = xy_source & 0b10 != 0;
    (16, ExecEffect::Normal)
}

fn outx(state: &mut State, memory: &mut impl Memory, offset: i16) -> (u8, ExecEffect) {
    let data = state.load_hl(memory);
    state.set_hl(state.get_hl().wrapping_add_signed(offset));
    state.b -= 1;
    state.zf = state.b.0 == 0;
    state.nf = true;
    state.mem_ptr.0 = state.get_bc().wrapping_add_signed(offset);
    (
        16,
        ExecEffect::Out {
            port: state.get_bc(),
            data,
        },
    )
}
