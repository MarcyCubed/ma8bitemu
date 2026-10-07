//! Tests using actual testing programs

use ma8bitemu::i8080::I8080FamilyEmulator;
use ma8bitemu::{ExecEffect, i8080, z80};

/// Executes a CP/M program
struct CpmRunner<E> {
    /// The memory
    memory: [u8; 1 << 16],
    /// The emulator that will run the program
    emulator: E,
    /// The number of clock cycles taken
    cycles: u64,
    /// The program output
    output: Vec<u8>,
}

impl<E: I8080FamilyEmulator> CpmRunner<E> {
    fn new(emulator: E) -> Self {
        let mut emulator = emulator;
        emulator.set_pc(0x100);
        Self {
            // Initialize with HALT to find when programs go where they shouldn't
            memory: [0x76; _],
            emulator,
            cycles: 0,
            output: vec![],
        }
    }

    /// Load the program into memory at the address 0x100 and set up the CP/M call stubs
    fn load_program(&mut self, program: &[u8]) {
        self.memory[0x100..program.len() + 0x100].copy_from_slice(program);
        // Trap CP/M program exit with an "OUT 1, a" instruction
        self.memory[0x0] = 0xD3;
        self.memory[0x1] = 0x00;
        // CP/M BDOS call is an IN a, 0 instruction so we can trap and handle it
        self.memory[0x5] = 0xDB;
        self.memory[0x6] = 0x00;
        // Return from the BDOS call
        self.memory[0x7] = 0xC9;
    }

    /// Handle CP/M BDOS calls 2 and 9
    fn bdos_call(&mut self) {
        match self.emulator.get_c().0 {
            2 => {
                // Function 2: Print s character to the screen
                self.output.push(self.emulator.get_e().0);
            }
            9 => {
                // Function 9: Write a $ terminated string to the screen
                let mut addr = self.emulator.get_de() as usize;
                while self.memory[addr] != '$' as u8 {
                    self.output.push(self.memory[addr]);
                    addr += 1;
                }
            }
            _ => panic!("Unknown BDOS call"),
        }
    }

    /// Run the loaded program
    fn run(&mut self) {
        loop {
            let (cycles, effect) = self.emulator.run(&mut self.memory);
            self.cycles += cycles;
            match effect {
                ExecEffect::In { .. } => {
                    self.bdos_call();
                }
                ExecEffect::Out { .. } => {
                    // Finished execution
                    break;
                }
                ExecEffect::Halt => {
                    panic!("The program crashed");
                }
                _ => {}
            }
        }
    }
}

/// Run a test program
fn run_test(
    emulator: impl I8080FamilyEmulator,
    program: &[u8],
    expected_output: &[u8],
    expected_time: u64,
) {
    let mut runner = CpmRunner::new(emulator);
    runner.load_program(program);
    runner.run();
    assert_eq!(expected_output, runner.output);
    assert_eq!(expected_time, runner.cycles)
}

/// Run a test program in an Intel 8080 CPU
fn run_8080_test(program: &[u8], expected_output: &[u8], expected_time: u64) {
    run_test(
        i8080::Emulator::new(),
        program,
        expected_output,
        expected_time,
    )
}

/// Run a test program in a Z80 CPU
fn run_z80_test(program: &[u8], expected_output: &[u8], expected_time: u64) {
    run_test(
        z80::Emulator::new(),
        program,
        expected_output,
        expected_time,
    )
}

#[test]
fn test_z80_prelim() {
    run_z80_test(
        include_bytes!("com/prelim.com"),
        include_bytes!("com/prelim.com.output"),
        8721,
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "Slow test: run it in release mode")]
fn test_zexall() {
    run_z80_test(
        include_bytes!("com/zexall.com"),
        include_bytes!("com/zexall.com.output"),
        46734978649,
    );
}

#[test]
fn test_8080_pre() {
    run_8080_test(
        include_bytes!("com/8080pre.com"),
        include_bytes!("com/8080pre.com.output"),
        7817,
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "Slow test: run it in release mode")]
fn test_8080_exm() {
    run_8080_test(
        include_bytes!("com/8080exm.com"),
        include_bytes!("com/8080exm.com.output"),
        23803381171,
    );
}
