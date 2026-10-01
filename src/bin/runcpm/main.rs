use clap::Parser;
use ma8bitemu::i8080::I8080FamilyEmulator;
use ma8bitemu::{ExecEffect, i8080, z80};
use std::fs;
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(version, about = "Run simple CP/M-80 programs", long_about = None)]
struct Args {
    /// Start to display the emulator state after executing this number of instructions
    #[arg(short, long)]
    start: Option<u64>,

    /// Stop displaying the emulator state after executing this number of instructions
    #[arg(short, long)]
    end: Option<u64>,

    /// The CP/M-80 program to run
    program: PathBuf,

    /// Emulate the Z80 processor instead of the Intel 8080
    #[arg(short = 'z', long)]
    use_z80: bool,
}

/// Run a program in a very limited CP/M emulation
struct CpmRunner<E> {
    /// The number of instructions executed
    instruction_counter: u64,
    /// The number of cycles taken
    cycles: u64,
    /// The memory
    memory: [u8; 1 << 16],
    /// The emulator that will run the program
    emulator: E,
    /// Start to display the emulator state after executing this number of instructions
    start: u64,
    /// Stop displaying the emulator state after executing this number of instructions
    end: u64,
}

impl<E: I8080FamilyEmulator> CpmRunner<E> {
    /// Create a new runner
    fn new(program: &[u8], emulator: E, args: Args) -> Self {
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
            emulator,
            start: args
                .start
                .unwrap_or_else(|| if args.end.is_some() { 0 } else { u64::MAX }),
            end: args.end.unwrap_or(u64::MAX),
        };
        // Point PC to the start of the program
        runner.emulator.get_pc_mut().0 = 0x100;
        runner
    }

    /// Handle CP/M BDOS calls 2 and 9
    fn bdos_call(&self) {
        match self.emulator.get_c().0 {
            2 => {
                // Function 2: Print s character to the screen
                print!("{}", self.emulator.get_e().0 as char);
            }
            9 => {
                // Function 9: Write a $ terminated string to the screen
                let mut addr = self.emulator.get_de() as usize;
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
            if self.instruction_counter == self.end {
                return;
            } else if self.instruction_counter >= self.start {
                self.emulator.dump_memory(&self.memory);
            }
            let opcode = self.emulator.fetch_byte(&self.memory);
            self.instruction_counter += 1;
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
    let args = Args::parse();
    match fs::read(&args.program) {
        Ok(file) => {
            if args.use_z80 {
                let mut runner = CpmRunner::new(&file, z80::Emulator::new(), args);
                runner.run();
            } else {
                let mut runner = CpmRunner::new(&file, i8080::Emulator::new(), args);
                runner.run();
            }
        }
        Err(error) => {
            eprintln!(
                "Can't open file {} : {}",
                args.program.to_string_lossy(),
                error
            )
        }
    }
}
