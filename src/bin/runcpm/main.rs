//use ma8080emu::emulator::{Emulator, EmulatorState};
//use
use ma8bitemu::i8080::{Emulator, I8080FamilyState};
use ma8bitemu::{EmulatorCore, ExecEffect, Fetch};
use std::fs;

const MIN_PRINT: usize = usize::MAX;
const MAX_PRINT: usize = usize::MAX;

//const MIN_PRINT: usize = 000000;
//const MAX_PRINT: usize = 100000;

/// Run a program in a very limited CP/M emulation
struct CpmRunner {
    /// The number of instructions executed
    instruction_counter: usize,
    /// The number of cycles taken
    cycles: u64,
    /// The memory
    memory: [u8; 1 << 16],
    /// The emulator that will run the program
    emulator: Emulator,
}

impl CpmRunner {
    /// Create a new runner
    fn new(program: &[u8]) -> Self {
        // We initialize the memory with "HALT" instructions, so whenever we go where we shouldn't the
        // program crashes.
        let mut memory = [0x76; 0x10000];
        memory[0x100..program.len() + 0x100].copy_from_slice(program);
        // Trap CP/M program exit with an "OUT 0, A" instruction
        memory[0x0] = 0xD3;
        memory[0x1] = 0x00;
        // CP/M BDOS call is an "OUT 1, A" instruction so we can trap and handle it
        memory[0x5] = 0xD3;
        memory[0x6] = 0x01;
        // Return from the BDOS call
        memory[0x7] = 0xC9;
        let mut runner = Self {
            instruction_counter: 0,
            cycles: 0,
            memory,
            emulator: Emulator::new(),
        };
        // Point PC to the start of the program
        runner.emulator.state.pc.0 = 0x100;
        runner
    }

    /// Handle CP/M BDOS calls 2 and 9
    fn bdos_call(&self) {
        match self.emulator.state.c.0 {
            2 => {
                // Function 2: Print s character to the screen
                print!("{}", self.emulator.state.e.0 as char);
            }
            9 => {
                // Function 9: Write a $ terminated string to the screen
                let mut addr = self.emulator.state.get_de() as usize;
                while self.memory[addr] != '$' as u8 {
                    print!("{}", self.memory[addr] as char);
                    addr += 1;
                }
            }
            _ => panic!("Unknown BDOS call"),
        }
    }

    /// Run the program stored in memory
    fn run(&mut self) {
        loop {
            let opcode = self.emulator.fetch_byte(&self.memory);
            self.instruction_counter += 1;
            if self.instruction_counter == MAX_PRINT {
                return;
            } else if self.instruction_counter >= MIN_PRINT {
                self.emulator.state.dump(opcode);
            }
            let (cycles, result) = self.emulator.run_opcode(opcode, &mut self.memory);
            self.cycles += cycles as u64;
            match result {
                ExecEffect::Halt => {
                    println!();
                    println!("Crashed");
                    break;
                }
                ExecEffect::Out { port, .. } => {
                    if port == 0 {
                        println!();
                        println!("Finished execution");
                        break;
                    } else {
                        self.bdos_call();
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() {
    for file in std::env::args().skip(1) {
        match fs::read(&file) {
            Ok(file) => {
                let mut runner = CpmRunner::new(&file);
                runner.run();
            }
            Err(error) => {
                eprintln!("Can't open file {} : {}", file, error)
            }
        }
    }
}
