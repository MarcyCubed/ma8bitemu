#![cfg_attr(not(feature = "std"), no_std)]

pub mod i8080;
pub mod memory;

use crate::memory::Memory;

/// Trait for things that fetch an instruction from memory
pub trait Fetch {
    /// Load one byte from memory.
    ///
    /// It should fetch the next instruction and advance the program counter.
    fn fetch_byte(&mut self, memory: &impl Memory) -> u8;

    /// Load a 16-bit word from memory
    fn fetch_word(&mut self, memory: &impl Memory) -> u16 {
        let byte_0 = self.fetch_byte(memory);
        let byte_1 = self.fetch_byte(memory);
        u16::from_le_bytes([byte_0, byte_1])
    }
}

/// Trait for emulators.
///
/// This is used for the regular execution of instructions. It doesn't manage interrupts since they
/// work differently on each processor.
pub trait EmulatorCore: Fetch {
    /// Get the next instruction to be executed.
    ///
    /// Reimplement this if you need something more complex than fetching the next instruction, like
    /// executing an instruction that's not in memory as 8080 interrupts do.
    fn next_instruction(&mut self, memory: &impl Memory) -> u8 {
        self.fetch_byte(memory)
    }

    /// Execute a single opcode.
    ///
    /// Return the number of clock cycles it took to execute the instruction and the result of the
    /// execution.
    ///
    /// This is the core of an emulator.
    fn run_opcode(&mut self, opcode: u8, memory: &mut impl Memory) -> (u8, ExecEffect);

    /// Execute the next instruction
    ///
    /// Return the number of clock cycles it took to execute the instruction and the result of the execution
    fn step(&mut self, memory: &mut impl Memory) -> (u8, ExecEffect) {
        let opcode = self.next_instruction(memory);
        let result = self.run_opcode(opcode, memory);
        result
    }

    /// Run a program until it exceeds the number of clock cycles, halts or performs I/O
    ///
    /// Return the number of clock cycles it took to execute the program and the result of the execution
    fn run_limited_clock(&mut self, memory: &mut impl Memory, limit: usize) -> (usize, ExecEffect) {
        let mut clock_cycles = 0;

        let result = loop {
            if clock_cycles > limit {
                break ExecEffect::Normal;
            }

            let (cycles, result) = self.step(memory);
            clock_cycles += cycles as usize;
            match result {
                ExecEffect::Normal | ExecEffect::InterruptDelay => continue,
                result => break result,
            }
        };

        (clock_cycles, result)
    }

    /// Run a program until it halts or performs I/O
    ///
    /// Return the number of clock cycles it took to execute the program and the result of the execution.
    ///
    /// Since this function only stops when the program needs data or halts, it may loop forever.
    fn run(&mut self, memory: &mut impl Memory) -> (usize, ExecEffect) {
        let mut clock_cycles = 0;

        loop {
            let (cycles, result) = self.step(memory);
            clock_cycles += cycles as usize;
            match result {
                ExecEffect::Normal | ExecEffect::InterruptDelay => continue,
                result => return (clock_cycles, result),
            }
        }
    }
}

/// The effect of executing an instruction
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExecEffect {
    /// The instruction doesn't affect anything else beyond the processor state
    Normal,
    /// Halt the emulator
    Halt,
    /// Disable interrupts for the next instruction
    InterruptDelay,
    /// The emulator wants to read the given port
    In { port: u8 },
    /// The emulator wants to write data to the given port
    Out { port: u8, data: u8 },
}
