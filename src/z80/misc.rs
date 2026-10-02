//! Miscellaneous instructions (prefix ED)

use crate::i8080::I8080FamilyEmulator;
use crate::i8080::load::{lhld_rr, shld_rr};
use crate::memory::Memory;
use crate::z80::emulator::{InputContinuation, InterruptMode};
use crate::z80::z8080::double_add_flags;
use crate::z80::{Emulator, z8080};
use crate::{ExecEffect, i8080};
use std::num::Wrapping;

macro_rules! in_r_bc {
    ($emulator:ident, $body:expr) => {{
        $emulator.input_continuation = Some(|emu, _, input| {
            $body(emu, input);
            emu.nf = false;
            emu.hf = false;
            emu.parity_from_value(input);
            emu.flags_from_value(input);
        });
        (
            12,
            ExecEffect::In {
                port: $emulator.get_bc(),
            },
        )
    }};
}

/// Input continuation for INI and IND
macro_rules! inx_continuation {
    ($offset: literal) => {
        |emulator, memory, input| {
            let hl = emulator.get_hl();
            memory.store(hl, input);
            emulator.set_hl(hl.wrapping_add_signed($offset));
            emulator.b -= 1;
            emulator.nf = true;
            emulator.zf = emulator.b.0 == 0;
            emulator.mem_ptr.0 = emulator.get_bc();
            emulator.mem_ptr += 1;
        }
    };
}

/// INI and IND
macro_rules! inx {
    ($emulator: ident, $offset: literal) => {{
        $emulator.input_continuation = Some(inx_continuation!($offset));
        (
            16,
            ExecEffect::In {
                port: $emulator.get_bc(),
            },
        )
    }};
}

/// INIR and INDR
macro_rules! inxr {
    ($emulator: ident, $offset: literal) => {{
        // inir
        $emulator.input_continuation = Some(|emulator, memory, input| {
            let inx_cont: InputContinuation = inx_continuation!($offset);
            inx_cont(emulator, memory, input);
            if !emulator.zf {
                emulator.pc -= 2;
            }
        });

        (
            if $emulator.b.0 == 1 { 16 } else { 21 },
            ExecEffect::In {
                port: $emulator.get_bc(),
            },
        )
    }};
}

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
            in_r_bc!(emulator, |emu: &mut Emulator, input| emu.a.0 = input)
        }
        0x40 => in_r_bc!(emulator, |emu: &mut Emulator, input| emu.b.0 = input),
        0x48 => in_r_bc!(emulator, |emu: &mut Emulator, input| emu.c.0 = input),
        0x50 => in_r_bc!(emulator, |emu: &mut Emulator, input| emu.d.0 = input),
        0x58 => in_r_bc!(emulator, |emu: &mut Emulator, input| emu.e.0 = input),
        0x60 => in_r_bc!(emulator, |emu: &mut Emulator, input| emu.h.0 = input),
        0x68 => in_r_bc!(emulator, |emu: &mut Emulator, input| emu.l.0 = input),
        0x70 => in_r_bc!(emulator, |_: &mut Emulator, _| {}),
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
        0xa0 => ldx(emulator, memory, 1),                     // ldi
        0xa8 => ldx(emulator, memory, -1),                    // ldd
        0xb0 => ldxr(emulator, memory, 1),                    // ldir
        0xb8 => ldxr(emulator, memory, -1),                   // lddr
        0xa1 => cpx(emulator, memory, 1),                     // cpi
        0xa9 => cpx(emulator, memory, -1),                    // cpd
        0xb1 => cpxr(emulator, memory, 1),                    // cpir
        0xb9 => cpxr(emulator, memory, -1),                   // cpdr
        0xa3 => outx(emulator, memory, 1),                    // outi
        0xab => outx(emulator, memory, -1),                   // outd
        0xb3 => repeat(outx, |s| s.zf, emulator, memory, 1),  // otir
        0xbb => repeat(outx, |s| s.zf, emulator, memory, -1), // otdr
        0xa2 => inx!(emulator, 1),                            // ini
        0xaa => inx!(emulator, -1),                           // ind
        0xb2 => inxr!(emulator, 1),                           // inir
        0xba => inxr!(emulator, -1),                          // indr
        // NOP is the default
        _ => (8, ExecEffect::Normal),
    }
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
    adc_hl_flags(emulator, !value, !carry).0;
    emulator.nf = true;
    emulator.hf = !emulator.hf;
    emulator.cf = !emulator.cf;
    (15, ExecEffect::Normal)
}

/// Add the value and the carry to HL
fn adc_hl_flags(emulator: &mut Emulator, value: u16, carry: bool) -> (u8, ExecEffect) {
    let hl = emulator.get_hl();
    let sum = double_add_flags(emulator, hl, value, carry);
    emulator.set_hl(sum);
    emulator.zf = sum == 0;
    emulator.sf = sum >> 15 != 0;
    let sign = 1 << 15;
    emulator.pf = hl & sign == value & sign && hl & sign != sum & sign;
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
    emulator.pf = bc != 0;
    // XY flags are weird here
    let data = emulator.a + Wrapping(data);
    emulator.xf = data.0 & 0b1000 != 0;
    emulator.yf = data.0 & 0b10 != 0;
    (16, ExecEffect::Normal)
}

/// LDXR: Repeat LDX until the counter is 0
fn ldxr(emulator: &mut Emulator, memory: &mut impl Memory, offset: i16) -> (u8, ExecEffect) {
    ldx(emulator, memory, offset);
    (
        if emulator.pf {
            emulator.mem_ptr = emulator.pc;
            emulator.mem_ptr -= 1;
            emulator.pc -= 2;
            21
        } else {
            16
        },
        ExecEffect::Normal,
    )
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
    // Compare
    let diff = z8080::sub_flags(emulator, emulator.a.0, data, false);
    emulator.cf = carry;

    let bc = emulator.get_bc().wrapping_sub(1);
    emulator.set_bc(bc);
    emulator.pf = bc != 0;
    // XY flags are extra weird here
    let xy_source = diff.wrapping_sub(emulator.hf as u8);
    emulator.xf = xy_source & 0b1000 != 0;
    emulator.yf = xy_source & 0b10 != 0;
    emulator.mem_ptr.0 = emulator.mem_ptr.0.wrapping_add_signed(offset);
    (16, ExecEffect::Normal)
}

/// CPXR: Repeat CPX until the counter is 0 or the pointed values are equal
fn cpxr(emulator: &mut Emulator, memory: &mut impl Memory, offset: i16) -> (u8, ExecEffect) {
    cpx(emulator, memory, offset);
    (
        if emulator.pf && !emulator.zf {
            emulator.mem_ptr = emulator.pc;
            emulator.mem_ptr += 1;
            emulator.pc -= 2;
            21
        } else {
            16
        },
        ExecEffect::Normal,
    )
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
