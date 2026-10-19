//! Intel 8080 emulator

pub mod emulator;
pub(crate) mod jump;
pub(crate) mod load;
pub(crate) mod math;

use crate::memory::Memory;
use crate::{EmulatorCore, Fetch};
use core::num::Wrapping;
pub use emulator::Emulator;

/// Abstraction over the emulator of Intel 8080 processors and its relatives, so we can reuse
/// instructions
pub trait I8080FamilyEmulator: EmulatorCore {
    /// Get the value of the register A
    fn get_a(&self) -> Wrapping<u8>;

    /// Get the value of the register B
    fn get_b(&self) -> Wrapping<u8>;

    /// Get the value of the register C
    fn get_c(&self) -> Wrapping<u8>;

    /// Get the value of the register D
    fn get_d(&self) -> Wrapping<u8>;

    /// Get the value of the register E
    fn get_e(&self) -> Wrapping<u8>;

    /// Get the value of the register H
    fn get_h(&self) -> Wrapping<u8>;

    /// Get the value of the register L
    fn get_l(&self) -> Wrapping<u8>;

    /// Mutable access the register A
    fn get_a_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register B
    fn get_b_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register C
    fn get_c_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register D
    fn get_d_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register E
    fn get_e_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register H
    fn get_h_mut(&mut self) -> &mut Wrapping<u8>;

    /// Mutable access the register L
    fn get_l_mut(&mut self) -> &mut Wrapping<u8>;

    /// Get the value of the BC register pair
    #[inline]
    fn get_bc(&self) -> u16 {
        u16::from_le_bytes([self.get_c().0, self.get_b().0])
    }

    /// Set the value of the BC register pair
    #[inline]
    fn set_bc(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.get_c_mut().0 = bytes[0];
        self.get_b_mut().0 = bytes[1];
    }

    /// Get the value of the DE register pair
    #[inline]
    fn get_de(&self) -> u16 {
        u16::from_le_bytes([self.get_e().0, self.get_d().0])
    }

    /// Set the value of the DE register pair
    #[inline]
    fn set_de(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.get_e_mut().0 = bytes[0];
        self.get_d_mut().0 = bytes[1];
    }

    /// Get the value of the HL register pair
    #[inline]
    fn get_hl(&self) -> u16 {
        u16::from_le_bytes([self.get_l().0, self.get_h().0])
    }

    /// Set the value of the HL register pair
    #[inline]
    fn set_hl(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.get_l_mut().0 = bytes[0];
        self.get_h_mut().0 = bytes[1];
    }

    /// Get the value of the AF register pair
    #[inline]
    fn get_af(&self) -> u16 {
        u16::from_le_bytes([self.serialize_flags(), self.get_a().0])
    }

    /// Set the value of the AF register pair
    #[inline]
    fn set_af(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.deserialize_flags(bytes[0]);
        self.get_a_mut().0 = bytes[1];
    }

    /// Get the value of the PC register
    fn get_pc(&self) -> Wrapping<u16>;

    /// Mutable access the PC register
    fn get_pc_mut(&mut self) -> &mut Wrapping<u16>;

    /// Set the value of the PC register
    fn set_pc(&mut self, value: u16) {
        self.get_pc_mut().0 = value;
    }

    /// Get the value of the Stack Pointer register
    fn get_sp(&self) -> Wrapping<u16>;

    /// Get the value of the Stack Pointer register as an u16
    fn get_sp_u16(&self) -> u16 {
        self.get_sp().0
    }

    /// Set the value of the Stack Pointer register
    fn set_sp(&mut self, value: u16) {
        self.get_sp_mut().0 = value
    }

    /// Mutable access the Stack Pointer register
    fn get_sp_mut(&mut self) -> &mut Wrapping<u16>;

    /// Set the flags from a value
    fn flags_from_value(&mut self, value: u8);

    /// Get the default flags from the accumulator
    #[inline]
    fn flags_from_accumulator(&mut self) {
        let a = self.get_a().0;
        self.flags_from_value(a);
    }

    /// Set the overflow flag to the boolean value
    fn overflow_flag(&mut self, overflow: bool);

    /// Set the parity flag from a value
    fn parity_from_value(&mut self, value: u8);

    /// Get the parity flag from the accumulator
    #[inline]
    fn parity_from_accumulator(&mut self) {
        let a = self.get_a().0;
        self.parity_from_value(a);
    }

    /// Turn the flags into a bitmap
    fn serialize_flags(&self) -> u8;

    /// Load the flags from the bit flags
    fn deserialize_flags(&mut self, flags: u8);

    /// Get the carry flag
    fn get_cf(&self) -> bool;

    /// Set the carry flag
    fn set_cf(&mut self, flag: bool);

    /// Get the parity flag
    fn get_pf(&self) -> bool;

    /// Set the parity flag
    fn set_pf(&mut self, flag: bool);

    /// Get the zero flag
    fn get_zf(&self) -> bool;

    /// Set the zero flag
    fn set_zf(&mut self, flag: bool);

    /// Get the sign flag
    fn get_sf(&self) -> bool;

    /// Set the sign flag
    fn set_sf(&mut self, flag: bool);

    /// Get the auxiliary carry flag
    fn get_hf(&self) -> bool;

    /// Set the auxiliary carry flag
    fn set_hf(&mut self, flag: bool);

    /// Set the subtraction flag
    fn set_nf(&mut self, flag: bool);

    /// Load the data pointed by HL
    fn load_hl(&self, memory: &impl Memory) -> u8 {
        memory.load(self.get_hl())
    }

    /// Load the data pointed by HL
    fn store_hl(&self, memory: &mut impl Memory, data: u8) {
        memory.store(self.get_hl(), data)
    }

    /// Write the state to the screen.
    #[cfg(feature = "std")]
    fn dump(&self, opcode: u8);

    /// Write the state of the emulator and some memory data to the screen
    #[cfg(feature = "std")]
    fn dump_memory(&self, memory: &impl Memory);

    /// Sets the value of the internal register memptr
    fn set_memptr(&mut self, address: u16);

    /// Give the emulator an input requested by the IN instruction
    fn input(&mut self, memory: &mut impl Memory, value: u8);

    /// Request an interrupt to be serviced
    ///
    /// Return the number of clock cycles it takes to trigger the interrupt
    fn interrupt(&mut self, vector: u8) -> u8;
}

impl<T: I8080FamilyEmulator> Fetch for T {
    #[inline]
    fn fetch_byte(&mut self, memory: &impl Memory) -> u8 {
        let address = self.get_pc().0;
        *self.get_pc_mut() += 1;
        memory.load(address)
    }
}

/// Tests for emulators of processors in the Intel 8080 family
#[cfg(test)]
pub(crate) mod tests {
    use crate::i8080::I8080FamilyEmulator;
    use crate::memory::Memory;
    use crate::{ExecEffect, memory};
    use core::assert_matches;

    /// Test the `IN` instruction of a processor in the 8080 family
    pub(crate) fn test_input<E: I8080FamilyEmulator>(new: fn() -> E, clock_cycles: u8) {
        // in 0 in 0 in 0 ...
        let mut memory = memory::Repeat([0xdb, 0x0]);
        let mut emulator = new();
        for port in 0..=0xff {
            for input in 0..=0xff {
                // Set the port
                memory.0[1] = port;
                let (cycles, effect) = emulator.step(&mut memory);
                assert_eq!(clock_cycles, cycles);
                assert_eq!(effect, ExecEffect::In { port: port as u16 });
                emulator.input(&mut memory, input);
                assert_eq!(input, emulator.get_a().0);
            }
        }
    }
    /// Check if the OUT instruction is working properly
    pub(crate) fn test_output<E: I8080FamilyEmulator>(new: fn() -> E, clock_cycles: u8) {
        // out 0 out 0 out 0 ...
        let mut memory = memory::Repeat([0xd3, 0x0]);
        let mut emulator = new();
        for port in 0..=0xff {
            for output in 0..=0xff {
                // Set the port
                memory.0[1] = port;
                emulator.get_a_mut().0 = output;
                let (
                    cycles,
                    ExecEffect::Out {
                        port: eff_port,
                        data: eff_data,
                    },
                ) = emulator.step(&mut memory)
                else {
                    panic!("Out instruction didn't produce an Out effect")
                };
                assert_eq!(cycles, clock_cycles);
                assert_eq!(port, eff_port as u8);
                assert_eq!(output, eff_data);
            }
        }
    }

    /// Make a program to test interrupts
    ///
    ///  We can track where we are in the program
    fn make_program() -> [u8; 256] {
        [
            // This originally was written for a Z80 emulator, that's why it has Z80 assembly
            0xd3, 0x01, // 00h: out (1), a
            // Put SP within the memory
            0x31, 0xff, 0x00, // 02h: ld sp, 0xff
            0xfb, // 05h: ei
            0xd3, 0x02, // 06h: out (2), a
            0xd3, 0x03, // 08h: out (3), a
            // Block with interrupts disabled
            0xf3, // 0ah: di
            0xd3, 0x04, // 0bh: out (4), a
            0xfb, // 0dh: ei
            // An out immediately after ei
            0xd3, 0x05, // 0eh: out (5), a
            0xd3, 0x06, // 10h: out (6), a
            0x76, // 12h: halt
            0x00, 0x00, 0x00, 0x00, 0x00, // 13h: nop *  5
            // an interruption handler at 18h
            0xf3, // 18h: di
            0xd3, 0x07, // 19h: out (7), a
            0xfb, // 1bh: ei
            0xc9, // 1ch: ret
            0xd3, 0x06, // 1dh: out (6), a
            0x00, //nop
            // an interruption handler with 2 outs at 20h
            0xf3, // 20h: di
            0xd3, 0x20, // 21h: out (20h), a
            0xd3, 0x21, // 23h: out (21h), a
            0xfb, // 25h: ei
            0xc9, // 26h: ret
            0x00, // nop
            // A different interruption handler at 28h
            0xf3, // 28h: di
            0x00, // 29h: nop
            0xd3, 0x28, // 2ah: out (0x28), a
            0x00, // 2ch: nop
            0xfb, // 2dh: ei
            0xc9, // 2eh: ret
            0x00, // nop
            // Filler
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 30h: nop *  8
            // rst 38h
            0xf3, // 38h: di
            0x00, // 39h: nop
            0xd3, 0x38, // 3ah: out (38h), a
            0x00, // 3ch nop
            0xfb, // 3dh: ei
            0xc9, // 3eh: ret
            0x00, // nop
            // Filler
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 40h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 48h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 50h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 58h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 60h: nop * 6
            // NMI handler
            0xd3, 0x66, // 66h: out (66h), a
            0xed, 0x45, // 68h: retn
            // Should be unreachable
            0xd3, 0xbb, // 6ah: out (bbh), a
            0x76, // 6ch: halt
            0x00, 0x00, 0x00, // 6dh: nop
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 70h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 78h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 80h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // a0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // a8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // b0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // b8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // c0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // c8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // d0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // d8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // e0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // e8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f8h: nop *  8
        ]
    }

    macro_rules! assert_port {
        ($number:literal, $result:expr, $text: literal) => {
            core::assert_matches!($result, ExecEffect::Out { .. });
            if let ExecEffect::Out { port, .. } = $result {
                assert_eq!($number, port & 0xff, $text);
            }
        };
    }

    /// Run until we execute an Out or reach the limit
    fn run_until_out(
        emulator: &mut impl I8080FamilyEmulator,
        program: &mut impl Memory,
        limit: u32,
    ) -> ExecEffect {
        let mut limit = limit;
        loop {
            let effect = emulator.run(program).1;
            if let ExecEffect::Out { .. } = effect {
                break effect;
            } else if limit == 0 {
                break ExecEffect::Normal;
            } else {
                limit -= 1;
            }
        }
    }

    /// Test if interrupts trigger as expected
    pub(crate) fn test_interrupts(mut emulator: impl I8080FamilyEmulator) {
        //let mut emulator = Emulator::new();
        let mut program = make_program();
        // Request an interrupt that runs rst 18h
        emulator.interrupt(0xdf);
        // Should run into the first out before the ei
        let (_, effect) = emulator.run(&mut program);
        assert_port!(1, effect, "Interrupt handled while interrupts are disabled");
        // Set up the stack
        emulator.step(&mut program);
        // Run ei. The next instruction should be still not interrupted
        let (_, effect) = emulator.step(&mut program);
        assert_matches!(effect, ExecEffect::InterruptDelay);
        // Should run into the out after the ei
        let (_, effect) = emulator.run(&mut program);
        assert_port!(
            2,
            effect,
            "ei didn't block interruptions for the next instruction"
        );
        // Should do a rst 18h to the interrupt handler and hit the OUT there
        let (_, effect) = emulator.run(&mut program);
        assert_port!(7, effect, "Didn't start handling interruption");
        // New interrupt with rst 20h
        emulator.interrupt(0xe7);
        // Should hit EI at the end of the handler
        let (_, effect) = emulator.step(&mut program);
        assert_matches!(effect, ExecEffect::InterruptDelay);
        // Should run the return and jump to where we were before
        let _ = emulator.step(&mut program);
        assert_eq!(
            emulator.get_pc().0,
            0x8,
            "interruption didn't return properly"
        );
        // Now it should handle the interrupt requested in the middle of the other one.
        let effect = run_until_out(&mut emulator, &mut program, 2000);
        assert_port!(0x20, effect, "Didn't start handling second interruption");
        let effect = run_until_out(&mut emulator, &mut program, 2000);
        assert_port!(
            0x21,
            effect,
            "Stopped in the middle of second interruption somehow"
        );
        // EI
        let (_, effect) = emulator.step(&mut program);
        assert_matches!(effect, ExecEffect::InterruptDelay);
        // RET
        let (_, effect) = emulator.step(&mut program);
        assert_matches!(effect, ExecEffect::Normal);
        // Did we really return?
        assert_eq!(
            emulator.get_pc().0,
            0x8,
            "interruption didn't return properly"
        );
        let (_, effect) = emulator.step(&mut program);
        assert_port!(3, effect, "interruption didn't return properly");
    }

    /// Test the method `run_limited_clock`
    pub(crate) fn test_run_limited_clock(mut emulator: impl I8080FamilyEmulator) {
        let mut program = [0xc3, 0x00, 0x00]; // jmp 0x00 : Infinite loop
        const LIMIT: u64 = 10000;
        let (clock_cycles, _) = emulator.run_limited_clock(&mut program, LIMIT);
        assert!(clock_cycles >= LIMIT, "Stopped before reaching the limit.");
    }

    /// Test the method `run_n_instructions`
    pub(crate) fn test_run_n_instructions(mut emulator: impl I8080FamilyEmulator) {
        let mut program = [0xc3, 0x00, 0x00]; // jmp 0x00 : Infinite loop
        for n in [0, 1, 1234, 1 << 18] {
            let (clock_cycles, counter, _) = emulator.run_n_instructions(&mut program, n);
            assert_eq!(n, counter);
            assert_eq!(n * 10, clock_cycles);
        }
    }
}
