//! Miscellaneous instructions (prefix ED)

use crate::i8080::I8080FamilyEmulator;
use crate::i8080::load::{lhld_rr, shld_rr};
use crate::memory::Memory;
use crate::z80::emulator::InterruptMode;
use crate::z80::z8080::add_hl_value;
use crate::z80::{Emulator, z8080};
use crate::{ExecEffect, i8080};
use std::num::Wrapping;

/// Execute a prefix ED instruction
///
/// Return the number of clock cycles it took to execute the instruction and the result of the
/// execution.
pub(super) fn run_opcode(
    emulator: &mut Emulator,
    opcode: u8,
    memory: &mut impl Memory,
) -> (u8, ExecEffect) {
    emulator.inc_r();

    match opcode {
        // IN a, (C)
        0x78 => {
            emulator.mem_ptr.0 = emulator.get_bc();
            emulator.mem_ptr += 1;
            in_r_bc(emulator, opcode)
        }
        // IN r, (C)
        0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x70 => in_r_bc(emulator, opcode),
        // OUT (C), a
        0x79 => {
            emulator.mem_ptr.0 = emulator.get_bc();
            emulator.mem_ptr += 1;
            out_value(emulator, emulator.a.0)
        }
        0x41 => out_value(emulator, emulator.b.0), // out (c), b
        0x49 => out_value(emulator, emulator.c.0), // out (c), c
        0x51 => out_value(emulator, emulator.d.0), // out (c), d
        0x59 => out_value(emulator, emulator.e.0), // out (c), e
        0x61 => out_value(emulator, emulator.h.0), // out (c), h
        0x69 => out_value(emulator, emulator.l.0), // out (c), l
        0x71 => out_value(emulator, 0),            // out (c), 0
        0x42 => sbc_hl_flags(emulator, emulator.get_bc(), emulator.cf), // sbc hl, bc
        0x52 => sbc_hl_flags(emulator, emulator.get_de(), emulator.cf), // sbc hl, de
        0x62 => sbc_hl_flags(emulator, emulator.get_hl(), emulator.cf), // sbc hl, hl
        0x72 => sbc_hl_flags(emulator, emulator.sp.0, emulator.cf), // sbc hl, sp
        0x4a => adc_hl_flags(emulator, emulator.get_bc(), emulator.cf), // adc hl, bc
        0x5a => adc_hl_flags(emulator, emulator.get_de(), emulator.cf), // adc hl, de
        0x6a => adc_hl_flags(emulator, emulator.get_hl(), emulator.cf), // adc hl, hl
        0x7a => adc_hl_flags(emulator, emulator.sp.0, emulator.cf), // adc hl, sp
        0x43 => {
            // ld (nn), bc
            shld_rr(emulator, memory, Emulator::get_bc);
            (20, ExecEffect::Normal)
        }
        0x53 => {
            // ld (nn), de
            shld_rr(emulator, memory, Emulator::get_de);
            (20, ExecEffect::Normal)
        }
        0x63 => {
            // ld (nn), hl
            shld_rr(emulator, memory, Emulator::get_hl);
            (20, ExecEffect::Normal)
        }
        0x73 => {
            // ld (nn), sp
            shld_rr(emulator, memory, Emulator::get_sp_u16);
            (20, ExecEffect::Normal)
        }
        0x44 => neg(emulator), // neg
        0x4b => {
            // ld bc, (nn)
            lhld_rr(emulator, memory, Emulator::set_bc);
            (20, ExecEffect::Normal)
        }
        0x5b => {
            // ld de, (nn)
            lhld_rr(emulator, memory, Emulator::set_de);
            (20, ExecEffect::Normal)
        }
        0x6b => {
            // ld hl, (nn)
            lhld_rr(emulator, memory, Emulator::set_hl);
            (20, ExecEffect::Normal)
        }
        0x7b => {
            // ld sp, (nn)
            lhld_rr(emulator, memory, Emulator::set_sp);
            (20, ExecEffect::Normal)
        }
        0x46 => im(emulator, InterruptMode::I8080), // im 0
        0x56 => im(emulator, InterruptMode::Rst38h), // im 1
        0x5e => im(emulator, InterruptMode::Vectored), // im 1
        0x47 => {
            // ld i, a
            emulator.i = emulator.a.0;
            (9, ExecEffect::Normal)
        }
        0x57 => ld_a_r(emulator, emulator.i), // ld a, i
        0x4f => {
            // ld r, a
            emulator.r = emulator.a;
            (9, ExecEffect::Normal)
        }
        0x5f => ld_a_r(emulator, emulator.r.0), // ld a, r
        0x67 => rrd(emulator, memory),          // rrd
        0x6f => rld(emulator, memory),          // rld
        0x45 => {
            // retn
            i8080::jump::ret(emulator, memory);
            emulator.iff1 = emulator.iff2;
            (14, ExecEffect::Normal)
        }
        0x4d => {
            // reti
            i8080::jump::ret(emulator, memory);
            (14, ExecEffect::Normal)
        }
        0xa0 => ldx(emulator, memory, 1),                    // ldi
        0xa8 => ldx(emulator, memory, -1),                   // ldd
        0xb0 => repeat(ldx, |s| s.pf, emulator, memory, 1),  // ldir
        0xb8 => repeat(ldx, |s| s.pf, emulator, memory, -1), // lddr
        0xa1 => cpx(emulator, memory, 1),                    // cpi
        0xa9 => cpx(emulator, memory, -1),                   // cpd
        0xb1 => repeat(cpx, |s| s.pf || s.zf, emulator, memory, 1), // cpir
        0xb9 => repeat(cpx, |s| s.pf || s.zf, emulator, memory, -1), // cpdr
        0xa2 | 0xaa | 0xb2 | 0xba => todo!("Input needs to be reworked"),
        0xa3 => outx(emulator, memory, 1),  // outi
        0xab => outx(emulator, memory, -1), // outd
        0xb3 => repeat(outx, |s| s.zf, emulator, memory, 1), // otir
        0xbb => repeat(outx, |s| s.zf, emulator, memory, -1), // otdr

        // NOP is the default
        _ => (8, ExecEffect::Normal),
    }
}

/// IN instruction
fn in_r_bc(emulator: &mut Emulator, opcode: u8) -> (u8, ExecEffect) {
    emulator.in_opcode = opcode;
    (
        12,
        ExecEffect::In {
            port: emulator.get_bc(),
        },
    )
}

/// Run an OUT instruction
fn out_value(emulator: &mut Emulator, data: u8) -> (u8, ExecEffect) {
    (
        12,
        ExecEffect::Out {
            port: emulator.get_bc(),
            data,
        },
    )
}

/// Subtract the value and the carry from HL
fn sbc_hl_flags(emulator: &mut Emulator, value: u16, carry: bool) -> (u8, ExecEffect) {
    add_hl_value(emulator, !value, !carry);
    emulator.nf = true;
    emulator.hf = !emulator.hf;
    emulator.cf = !emulator.cf;
    (15, ExecEffect::Normal)
}

/// Add the value and the carry to HL
fn adc_hl_flags(emulator: &mut Emulator, value: u16, carry: bool) -> (u8, ExecEffect) {
    add_hl_value(emulator, value, carry);
    (15, ExecEffect::Normal)
}

/// - A
fn neg(emulator: &mut Emulator) -> (u8, ExecEffect) {
    let a = emulator.a.0;
    emulator.a.0 = 0;
    z8080::sub_value(emulator, a, false);
    (8, ExecEffect::Normal)
}

/// Set the interrupt mode
fn im(emulator: &mut Emulator, mode: InterruptMode) -> (u8, ExecEffect) {
    emulator.interrupt_mode = mode;
    (8, ExecEffect::Normal)
}

/// Load a special register into A
fn ld_a_r(emulator: &mut Emulator, value: u8) -> (u8, ExecEffect) {
    emulator.a.0 = value;
    emulator.nf = false;
    emulator.hf = false;
    emulator.pf = emulator.iff2;
    emulator.flags_from_accumulator();
    (9, ExecEffect::Normal)
}

/// Perform a nybble rotate right between the memory pointed by `HL` and the least significant
/// nybble of `A`
fn rrd(emulator: &mut Emulator, memory: &mut impl Memory) -> (u8, ExecEffect) {
    emulator.mem_ptr.0 = emulator.get_hl();
    emulator.mem_ptr += 1;
    let data = emulator.load_hl(memory);
    emulator.store_hl(memory, (data >> 4) | (emulator.a.0 << 4));
    emulator.a.0 = (emulator.a.0 & 0xf0) | (data & 0x0f);
    emulator.hf = false;
    emulator.nf = false;
    emulator.flags_from_accumulator();
    emulator.parity_from_accumulator();
    (18, ExecEffect::Normal)
}

/// Perform a nybble rotate left between the memory pointed by `HL` and the least significant nybble
/// of `A`
fn rld(emulator: &mut Emulator, memory: &mut impl Memory) -> (u8, ExecEffect) {
    emulator.mem_ptr.0 = emulator.get_hl();
    emulator.mem_ptr += 1;
    let data = emulator.load_hl(memory);
    emulator.store_hl(memory, (data << 4) | (emulator.a.0 & 0x0f));
    emulator.a.0 = (emulator.a.0 & 0xf0) | (data >> 4);
    emulator.hf = false;
    emulator.nf = false;
    emulator.flags_from_accumulator();
    emulator.parity_from_accumulator();
    (18, ExecEffect::Normal)
}

/// LDI and LDD: Transfer data between two memory locations and increase or decrease the pointers
/// and decrease the counter
fn ldx(emulator: &mut Emulator, memory: &mut impl Memory, offset: i16) -> (u8, ExecEffect) {
    let data = memory.load(emulator.get_hl());
    memory.store(emulator.get_de(), data);
    emulator.set_de(emulator.get_de().wrapping_add_signed(offset));
    emulator.set_hl(emulator.get_hl().wrapping_add_signed(offset));
    let bc = emulator.get_bc().wrapping_sub(1);
    emulator.set_bc(bc);
    emulator.hf = false;
    emulator.nf = false;
    emulator.pf = bc == 0;
    // XY flags are weird here
    let data = emulator.a + Wrapping(data);
    emulator.xf = data.0 & 0b1000 != 0;
    emulator.yf = data.0 & 0b10 != 0;
    (16, ExecEffect::Normal)
}

/// Implement one of the repeated instructions
fn repeat<M: Memory>(
    instruction: fn(&mut Emulator, &mut M, i16) -> (u8, ExecEffect),
    exit_cond: impl Fn(&Emulator) -> bool,
    emulator: &mut Emulator,
    memory: &mut M,
    offset: i16,
) -> (u8, ExecEffect) {
    instruction(emulator, memory, offset);
    (
        if exit_cond(emulator) {
            16
        } else {
            emulator.pc -= 2;
            21
        },
        ExecEffect::Normal,
    )
}

/// Compare memory to accumulator, increase or decrease the pointer accordingly and decrease the
/// counter
fn cpx(emulator: &mut Emulator, memory: &mut impl Memory, offset: i16) -> (u8, ExecEffect) {
    let data = memory.load(emulator.get_hl());
    emulator.set_hl(emulator.get_hl().wrapping_add_signed(offset));
    let carry = emulator.cf;
    let half_carry = emulator.hf;
    // Compare
    z8080::sub_flags(emulator, emulator.a.0, data, false);
    emulator.cf = carry;

    let bc = emulator.get_bc().wrapping_sub(1);
    emulator.set_bc(bc);
    emulator.pf = bc == 0;
    // XY flags are extra weird here
    let xy_source = data.wrapping_sub(half_carry as u8);
    emulator.xf = xy_source & 0b1000 != 0;
    emulator.yf = xy_source & 0b10 != 0;
    (16, ExecEffect::Normal)
}

fn outx(emulator: &mut Emulator, memory: &mut impl Memory, offset: i16) -> (u8, ExecEffect) {
    let data = emulator.load_hl(memory);
    emulator.set_hl(emulator.get_hl().wrapping_add_signed(offset));
    emulator.b -= 1;
    emulator.zf = emulator.b.0 == 0;
    emulator.nf = true;
    emulator.mem_ptr.0 = emulator.get_bc().wrapping_add_signed(offset);
    (
        16,
        ExecEffect::Out {
            port: emulator.get_bc(),
            data,
        },
    )
}
