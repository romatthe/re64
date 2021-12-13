mod core;
mod instruction;

use std::fs::File;
use std::io::Read;
use std::{env, io};

/// Default DRAM size (128MiB).
pub const DRAM_SIZE: u64 = 1024 * 1024 * 128;

/// Emulated 64-bit RISC-V CPU implementing the RV64G ISA
struct CPU {
    /// 32 general-purpose 64-bit CPU registers
    regs: [u64; 32],
    /// Program Counter
    pc: u64,
    /// Code to be executed by the CPU
    /// TODO: This should be replaced by access to the memory bus at a later point in time
    dram: Vec<u8>,
}

impl CPU {
    pub fn new(code: Vec<u8>) -> Self {
        let mut regs = [0; 32];

        // The stack pointer is located at the end of the DRAM.
        regs[2] = DRAM_SIZE;

        // The zero register is supposed to be hardwired to always return a 0 value
        regs[0] = 0;

        Self {
            regs,
            pc: 0,
            dram: code,
        }
    }

    /// Reads a 32-bit instruction from memory
    fn fetch(&self) -> u32 {
        let index = self.pc as usize;

        // Read the next 4 bytes from memory in LE ordering
        u32::from_le_bytes([
            self.dram[index],
            self.dram[index + 1],
            self.dram[index + 2],
            self.dram[index + 3],
        ])
    }

    /// Decodes a 32-bit instruction and executes it.
    ///
    /// RISC-V base instructions have the following layout:
    /// ```
    /// |31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00|
    /// |-----------------------------------------------------------------------------------------------|
    /// |       funct7       |      rs2     |      rs1     | funct3 |      rd      |       opcode       |  // Register/Register
    /// |-----------------------------------------------------------------------------------------------|
    /// |               imm[11:0]           |      rs1     | funct3 |      rd      |       opcode       |  // Immediate
    /// |-----------------------------------------------------------------------------------------------|
    /// |                          imm[31:12]                       |      rd      |       opcode       |  // Upper immediate
    /// |-----------------------------------------------------------------------------------------------|
    /// |      imm[11:5]     |      rs2     |      rs1     | funct3 |   imm[4:0]   |       opcode       |  // Store
    /// |-----------------------------------------------------------------------------------------------|
    /// |12|    imm[10:5]    |      rs2     |      rs1     | funct3 |  imm[4:1] |11|       opcode       |  // Branch
    /// |-----------------------------------------------------------------------------------------------|
    /// |20|          imm[10:1]          |11|      imm[19:12]       |      rd      |       opcode       |  // Jump
    /// |-----------------------------------------------------------------------------------------------|
    /// ```
    fn execute(&mut self, instruction: u32) {
        // Extract information from the instruction
        let opcode = instruction & 0x7f;
        let rd = ((instruction >> 7) & 0x1f) as usize;
        let rs1 = ((instruction >> 15) & 0x1f) as usize;
        let rs2 = ((instruction >> 20) & 0x1f) as usize;

        // Emulate that register x0 is hardwired with all bits equal to 0.
        self.regs[0] = 0;

        match opcode {
            // addi
            0x13 => {
                let imm = ((instruction & 0xfff00000) as i32 as i64 >> 20) as u64;
                self.regs[rd] = self.regs[rs1].wrapping_add(imm);
            }
            // add
            0x33 => {
                self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
            }
            _ => {
                dbg!(format!("not implemented yet: opcode {:#x}", opcode));
            }
        }
    }

    /// Print values in all registers (x0-x31).
    pub fn dump_registers(&self) {
        let mut output = String::from("");
        let abi = [
            "zero", " ra ", " sp ", " gp ", " tp ", " t0 ", " t1 ", " t2 ", " s0 ", " s1 ", " a0 ",
            " a1 ", " a2 ", " a3 ", " a4 ", " a5 ", " a6 ", " a7 ", " s2 ", " s3 ", " s4 ", " s5 ",
            " s6 ", " s7 ", " s8 ", " s9 ", " s10", " s11", " t3 ", " t4 ", " t5 ", " t6 ",
        ];
        for i in (0..32).step_by(4) {
            output = format!(
                "{}\n{}",
                output,
                format!(
                    "x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x}",
                    i,
                    abi[i],
                    self.regs[i],
                    i + 1,
                    abi[i + 1],
                    self.regs[i + 1],
                    i + 2,
                    abi[i + 2],
                    self.regs[i + 2],
                    i + 3,
                    abi[i + 3],
                    self.regs[i + 3],
                )
            );
        }
        println!("{}", output);
    }
}

fn main() -> io::Result<()> {
    // CLI
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: harvey <filename>");
    }

    // Read the binary file into memory
    let mut file = File::open(&args[1])?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;

    let mut cpu = CPU::new(code);

    // Fetch, decode, execute cycle
    while cpu.pc < cpu.dram.len() as u64 {
        // Fetch
        let instruction = cpu.fetch();

        // Move the PC forward
        cpu.pc += 4;

        // Decode and execute
        cpu.execute(instruction);
    }

    cpu.dump_registers();

    Ok(())
}
