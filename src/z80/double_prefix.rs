//! Instructions with two prefixes

use crate::memory::Memory;
use crate::z80::{Emulator, bits};

#[inline(always)]
fn bit(emulator: &mut Emulator, data: u8, bit_number: u32) -> u8 {
    bits::bit_flags(emulator, data, bit_number);
    emulator.xy_from_value((emulator.mem_ptr.0 >> 8) as u8);
    20
}

/// Run an opcode prefixed by DD/FD CB
pub(super) fn run_opcode(
    emulator: &mut Emulator,
    address: u16,
    opcode: u8,
    memory: &mut impl Memory,
) -> u8 {
    let data = memory.load(address);
    let data = match opcode {
        // Documented instructions
        0x06 => bits::rlc_flags(emulator, data), // rlc (iz + d)
        0x0e => bits::rrc_flags(emulator, data), // rrc (iz + d)
        0x16 => bits::rl_flags(emulator, data),  // rl (iz + d)
        0x1e => bits::rr_flags(emulator, data),  // rr (iz + d)
        0x26 => bits::sla_flags(emulator, data), // sla (iz + d)
        0x2e => bits::sra_flags(emulator, data), // sra (iz + d)
        0x36 => bits::sll_flags(emulator, data), // sll (iz + d)
        0x3e => bits::srl_flags(emulator, data), // srl (iz + d)
        0x40..=0x47 => return bit(emulator, data, 0), // bit 0,(iz + d)
        0x48..=0x4f => return bit(emulator, data, 1), // bit 1,(iz + d)
        0x50..=0x57 => return bit(emulator, data, 2), // bit 2,(iz + d)
        0x58..=0x5f => return bit(emulator, data, 3), // bit 3,(iz + d)
        0x60..=0x67 => return bit(emulator, data, 4), // bit 4,(iz + d)
        0x68..=0x6f => return bit(emulator, data, 5), // bit 5,(iz + d)
        0x70..=0x77 => return bit(emulator, data, 6), // bit 6,(iz + d)
        0x78..=0x7f => return bit(emulator, data, 7), // bit 7,(iz + d)
        0x86 => data & !(1 << 0),                // res 0, (iz + d)
        0x8e => data & !(1 << 1),                // res 1, (iz + d)
        0x96 => data & !(1 << 2),                // res 2, (iz + d)
        0x9e => data & !(1 << 3),                // res 3, (iz + d)
        0xa6 => data & !(1 << 4),                // res 4, (iz + d)
        0xae => data & !(1 << 5),                // res 5, (iz + d)
        0xb6 => data & !(1 << 6),                // res 6, (iz + d)
        0xbe => data & !(1 << 7),                // res 7, (iz + d)
        0xc6 => data | (1 << 0),                 // set 0, (iz + d)
        0xce => data | (1 << 1),                 // set 1, (iz + d)
        0xd6 => data | (1 << 2),                 // set 2, (iz + d)
        0xde => data | (1 << 3),                 // set 3, (iz + d)
        0xe6 => data | (1 << 4),                 // set 4, (iz + d)
        0xee => data | (1 << 5),                 // set 5, (iz + d)
        0xf6 => data | (1 << 6),                 // set 6, (iz + d)
        0xfe => data | (1 << 7),                 // set 7, (iz + d)
        // Undocumented instructions
        _ => {
            // Run recursively on the documented variant of this instruction
            let cycles = run_opcode(emulator, address, (opcode & !0b111) + 6, memory);
            // Get the data that was written to memory by the documented instruction
            let data = memory.load(address);
            // Write the value to a register
            match opcode & 0b111 {
                0 => emulator.b.0 = data,
                1 => emulator.c.0 = data,
                2 => emulator.d.0 = data,
                3 => emulator.e.0 = data,
                4 => emulator.h.0 = data,
                5 => emulator.l.0 = data,
                7 => emulator.a.0 = data,
                _ => unreachable!(),
            }
            // Return early
            return cycles;
        }
    };
    memory.store(address, data);
    23
}
