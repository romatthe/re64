/// A trait for objects that are able to take an instruction type and do something useful with it.
/// There is one function per RISC-V instruction format. All functions return the
/// [InstructionProcessor::InstructionResult] associated type.
pub trait InstructionProcessor {
    type InstructionResult;

    /// Process an R-type instruction
    fn process_r(&mut self, instruction: RFormat) -> Self::InstructionResult;
    /// Process an I-type instruction
    fn process_i(&mut self, instruction: IFormat) -> Self::InstructionResult;
    /// Process an S-type instruction
    fn process_s(&mut self, instruction: SFormat) -> Self::InstructionResult;
    /// Process an B-type instruction
    fn process_b(&mut self, instruction: BFormat) -> Self::InstructionResult;
    /// Process an U-type instruction
    fn process_u(&mut self, instruction: UFormat) -> Self::InstructionResult;
    /// Process an J-type instruction
    fn process_j(&mut self, instruction: JFormat) -> Self::InstructionResult;
}

/// The different instruction formats supported on the RISC-V architecture.
pub enum Instruction {
    R(RFormat),
    I(IFormat),
    S(SFormat),
    B(BFormat),
    U(UFormat),
    J(JFormat),
    NOP(InstructionBytes),
}

/// Wrapped instruction bytes
// TODO: Actually use this? Change the From<u32> to From<InstructionBytes>
pub struct InstructionBytes(pub u32);

impl InstructionBytes {
    pub fn opcode(&self) -> u32 {
        &self.0 & 0x7f
    }

    pub fn funct3(&self) -> u32 {
        (&self.0 >> 12) & 0x7
    }
}

impl From<InstructionBytes> for Instruction {
    fn from(instruction: InstructionBytes) -> Self {
        let opcode = instruction.opcode();
        let funct3 = instruction.funct3();

        // Decoded according to https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf, Chapter 19
        // TODO: Only includes RV32I Base Instruction Set so far
        match (opcode, funct3) {
            // Base Instruction Set
            (0110111, _) => Instruction::U(UFormat::from(instruction)), // LUI
            (0010111, _) => Instruction::U(UFormat::from(instruction)), // AUIPC
            (1101111, _) => Instruction::J(JFormat::from(instruction)), // JAL
            (1100111, _) => Instruction::I(IFormat::from(instruction)), // JALR
            (1100011, _) => Instruction::B(BFormat::from(instruction)), // BEQ, BNE, BLT, BGE, BLTU, BGEU
            (0000011, _) => Instruction::I(IFormat::from(instruction)), // LB, LH, LW, LBU, LHU
            (0010011, 001) => Instruction::S(SFormat::from(instruction)), // SLLI
            (0010011, 101) => Instruction::S(SFormat::from(instruction)), // SRLI, SRAI
            (0010011, _) => Instruction::I(IFormat::from(instruction)), // ADDI, SLTI, SLTIU, XORI, ORI, ANDI
            (0100011, _) => Instruction::S(SFormat::from(instruction)), // SB, SH, SW
            (0110011, _) => Instruction::R(RFormat::from(instruction)), // ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND

            // TODO: Currently unsupported FENCE, FENCE.I, ECALL and EBREAK and CSR calls
            (0001111, _) => Instruction::NOP(instruction), // FENCE, FENC.I
            (1110011, _) => Instruction::NOP(instruction), // ECALL, EBREAK, CSRRW, CSRRS, CSRRC, CSRRWI, CSRRSI, CSRRCI

            (_, _) => unimplemented!(
                "Instruction (opcode: {}, func3: {}) not implemented",
                opcode,
                funct3,
            ),
        }
    }
}

pub enum InstructionException {}

/// An instruction in the R-type format, which are instructions that use 3 register inputs. It has the following layout:
/// ```
/// |31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00|
/// |-----------------------------------------------------------------------------------------------|
/// |       funct7       |      rs2     |      rs1     | funct3 |      rd      |       opcode       |  // Register/Register
/// |-----------------------------------------------------------------------------------------------|
/// ```
pub struct RFormat {
    /// Operation bits 1. Combine with opcode for complete operation description. 3-bit.
    pub funct3: u32,
    /// Operation bits 2.Combine with opcode for complete operation description. 7-bit.
    pub funct7: u32,
    /// First instruction operand, aka source register 1. 5-bit.
    pub rs1: usize,
    /// Second instruction operand, aka source register 2. 5-bit.
    pub rs2: usize,
    /// Destination register, receives the result of the computation. 5-bit.
    pub rd: usize,
    /// Instruction opcode. Partially specifies the operation. 7-bit.
    pub opcode: u32,
}

impl From<InstructionBytes> for RFormat {
    fn from(instruction: InstructionBytes) -> Self {
        Self {
            funct3: (instruction.0 >> 12) & 0x7,
            funct7: (instruction.0 >> 25) & 0x7f,
            rs1: ((instruction.0 >> 15) & 0x1f) as usize,
            rs2: ((instruction.0 >> 20) & 0x1f) as usize,
            rd: ((instruction.0 >> 7) & 0x1f) as usize,
            opcode: instruction.0 & 0x7f,
        }
    }
}

/// An instruction in the I-type format, which are instructions that use immediates. It has the following layout:
/// ```
/// |31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00|
/// |-----------------------------------------------------------------------------------------------|
/// |             imm[11:0]             |      rs1     | funct3 |      rd      |       opcode       |  // Immediate
/// |-----------------------------------------------------------------------------------------------|
/// ```
pub struct IFormat {
    /// Immediate value, sign-extended to 32-bits. 12-bit.
    pub imm: i32,
    /// Operation bits. Combine with opcode for complete operation description. 3-bit.
    pub funct3: u32,
    /// Register operand, aka source register. 5-bit.
    pub rs1: usize,
    /// Destination register, receives the result of the computation. 5-bit.
    pub rd: usize,
    /// Instruction opcode. Uniquely specifies the operation. 7-bit.
    pub opcode: u32,
}

impl From<InstructionBytes> for IFormat {
    fn from(instruction: InstructionBytes) -> Self {
        let uimm: i32 = ((instruction.0 >> 20) & 0x7ff) as i32;

        let imm: i32 = if (instruction.0 & 0x8000_0000) != 0 {
            uimm - (1 << 11)
        } else {
            uimm
        };

        Self {
            imm,
            funct3: instruction.funct3(),
            rs1: ((instruction.0 >> 15) & 0x1f) as usize,
            rd: ((instruction.0 >> 7) & 0x1f) as usize,
            opcode: instruction.opcode(),
        }
    }
}

/// An instruction in the I/SHAMT-type format, which is a specliazed version of the I-type format.
/// It is used for shift instructions and has the following layout:
/// ```
/// |31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00|
/// |-----------------------------------------------------------------------------------------------|
/// |      imm[11:5]     |     shamt    |      rs1     | funct3 |      rd      |       opcode       |  // Shift
/// |-----------------------------------------------------------------------------------------------|
/// ```
pub struct ISType {
    /// The right shift type is encoded in this immediate field. 7-bit.
    pub imm: u32,
    /// The shift amount is encoded in this shamt field, 5-bit.
    pub shamt: u32,
    /// Operation bits. Combine with opcode for complete operation description. 3-bit.
    pub funct3: u32,
    /// The operand to be shifted. 5-bit.
    pub rs1: usize,
    /// Destination register. 5-bit.
    pub rd: usize,
    /// Instruction opcode. Uniquely specifies the operation. 7-bit.
    pub opcode: u32,
}

impl From<InstructionBytes> for ISType {
    fn from(instruction: InstructionBytes) -> Self {
        let i = IFormat::from(instruction);
        let shamt = (i.imm as u32) & 0x1f;

        Self {
            imm: (instruction.0 >> 25) & 0x7f,
            shamt,
            funct3: i.funct3,
            rs1: i.rs1,
            rd: i.rd,
            opcode: instruction.0 & 0x7f,
        }
    }
}

/// An instruction in the I-type format, which are store instructions using two registers. It has the following layout:
/// ```
/// |31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00|
/// |-----------------------------------------------------------------------------------------------|
/// |      imm[11:5]     |      rs2     |      rs1     | funct3 |   imm[4:0]   |       opcode       |  // Store
/// |-----------------------------------------------------------------------------------------------|
/// ```
pub struct SFormat {
    /// Combined immediate value of 12-bits. Obtained by combining `imm[11:5]` with `imm[4:0]`.
    pub imm: i32,
    /// Register with base memory address. 5-bit.
    pub rs1: usize,
    /// Register with the data to be stored. 5-bit.
    pub rs2: usize,
    /// Operation bits. Combine with opcode for complete operation description. 3-bit.
    pub funct3: u32,
    /// Instruction opcode. Uniquely specifies the operation. 7-bit.
    pub opcode: u32,
}

impl From<InstructionBytes> for SFormat {
    fn from(instruction: InstructionBytes) -> Self {
        let uimm: i32 = (((instruction.0 >> 20) & 0x7e0) | ((instruction.0 >> 7) & 0x1f)) as i32;

        let imm: i32 = if (instruction.0 & 0x8000_0000) != 0 {
            uimm - (1 << 11)
        } else {
            uimm
        };

        Self {
            imm,
            rs1: ((instruction.0 >> 15) & 0x1f) as usize,
            rs2: ((instruction.0 >> 20) & 0x1f) as usize,
            funct3: (instruction.0 >> 12) & 0x7,
            opcode: instruction.0 & 0x7f,
        }
    }
}

/// An instruction in the B-type fomat, which are branch instructions. It has the following layout:
/// ```
/// |31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00|
/// |-----------------------------------------------------------------------------------------------|
/// |12|    imm[10:5]    |      rs2     |      rs1     | funct3 |  imm[4:1] |11|       opcode       |  // Branch
/// |-----------------------------------------------------------------------------------------------|
/// ```
pub struct BFormat {
    /// Combined immediate value of 12-bits. Obtained by combining `imm[12|10:5]` with `imm[4:1|11]`.
    pub imm: i32,
    /// First instruction operand, aka source register 1. 5-bit.
    pub rs1: usize,
    /// Second instruction operand, aka source register 2. 5-bit.
    pub rs2: usize,
    /// Operation bits. Combine with opcode for complete operation description. 3-bit.
    pub funct3: u32,
    /// Instruction opcode. Uniquely specifies the operation. 7-bit.
    pub opcode: u32,
}

impl From<InstructionBytes> for BFormat {
    fn from(instruction: InstructionBytes) -> Self {
        let uimm: i32 = (((instruction.0 >> 20) & 0x7e0)
            | ((instruction.0 >> 7) & 0x1e)
            | ((instruction.0 & 0x80) << 4)) as i32;

        let imm: i32 = if (instruction.0 & 0x8000_0000) != 0 {
            uimm - (1 << 12)
        } else {
            uimm
        };

        Self {
            imm,
            rs1: ((instruction.0 >> 15) & 0x1f) as usize,
            rs2: ((instruction.0 >> 20) & 0x1f) as usize,
            funct3: (instruction.0 >> 12) & 0x7,
            opcode: instruction.0 & 0x7f,
        }
    }
}

/// An instruction in the U-type format, which are instructions that use "upper immediates" (aka 32-bit immediate).
/// It has the following layout:
/// ```
/// |31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00|
/// |-----------------------------------------------------------------------------------------------|
/// |                          imm[31:12]                       |      rd      |       opcode       |  // Upper immediate
/// |-----------------------------------------------------------------------------------------------|
/// ```
pub struct UFormat {
    /// Immediate value, sign-extended to 32-bits. 20-bit.
    pub imm: i32,
    /// Destination register, receives the result of the computation. 5-bit.
    pub rd: usize,
    /// Instruction opcode. Uniquely specifies the operation. 7-bit.
    pub opcode: u32,
}

impl From<InstructionBytes> for UFormat {
    fn from(instruction: InstructionBytes) -> Self {
        Self {
            imm: (instruction.0 & 0xffff_f000) as i32,
            rd: ((instruction.0 >> 7) & 0x1f) as usize,
            opcode: instruction.0 & 0x7f,
        }
    }
}

/// An instruction in the J-type format, which are jump instructions. It has the following layout:
/// ```
/// |31|30|29|28|27|26|25|24|23|22|21|20|19|18|17|16|15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00|
/// |-----------------------------------------------------------------------------------------------|
/// |20|          imm[10:1]          |11|      imm[19:12]       |      rd      |       opcode       |  // Jump
/// |-----------------------------------------------------------------------------------------------|
/// ```
pub struct JFormat {
    /// Immediate value, sign-extended to 32-bits. 20-bit.
    pub imm: i32,
    /// Holds the return address for the jump return. 5-bit.
    pub rd: usize,
    /// Instruction opcode. Uniquely specifies the operation. 7-bit.
    pub opcode: u32,
}

impl From<InstructionBytes> for JFormat {
    fn from(instruction: InstructionBytes) -> Self {
        let uimm: i32 = ((instruction.0 & 0xff000)
            | ((instruction.0 & 0x100000) >> 9)
            | ((instruction.0 >> 20) & 0x7fe)) as i32;

        let imm: i32 = if (instruction.0 & 0x8000_0000) != 0 {
            uimm - (1 << 20)
        } else {
            uimm
        };

        Self {
            imm,
            rd: ((instruction.0 >> 7) & 0x1f) as usize,
            opcode: instruction.0 & 0x7f,
        }
    }
}
