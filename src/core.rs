use crate::instruction::{
    BFormat, IFormat, Instruction, InstructionException, InstructionProcessor, JFormat, RFormat,
    SFormat, UFormat,
};

/// Enum indicating whether the PC was updated after the execution of an instruction.
pub enum CounterState {
    Updated,
    NotUpdated,
}

/// A RISC-V hardware thread. A RISC-V compatible core might support multiple RISC-V-
/// compatible hardware threads, or harts, through multithreading.
pub struct Hart {
    /// 32 general-purpose 64-bit CPU registers
    regs: [usize; 32],
    /// Program Counter
    pc: u64,
}

impl Hart {
    pub fn step(&mut self) -> Result<(), InstructionException> {
        if let Some(instruction) = unimplemented!() {
            let result = match instruction {
                Instruction::R(r) => self.process_r(r),
                Instruction::I(i) => self.process_i(i),
                Instruction::S(s) => self.process_s(s),
                Instruction::B(b) => self.process_b(b),
                Instruction::U(u) => self.process_u(u),
                Instruction::J(j) => self.process_j(j),
            };
        }

        Ok(())
    }
}

impl InstructionProcessor for Hart {
    type InstructionResult = Result<CounterState, InstructionException>;

    fn process_r(&mut self, instruction: RFormat) -> Self::InstructionResult {
        todo!()
    }

    fn process_i(&mut self, instruction: IFormat) -> Self::InstructionResult {
        todo!()
    }

    fn process_s(&mut self, instruction: SFormat) -> Self::InstructionResult {
        todo!()
    }

    fn process_b(&mut self, instruction: BFormat) -> Self::InstructionResult {
        todo!()
    }

    fn process_u(&mut self, instruction: UFormat) -> Self::InstructionResult {
        todo!()
    }

    fn process_j(&mut self, instruction: JFormat) -> Self::InstructionResult {
        todo!()
    }
}
