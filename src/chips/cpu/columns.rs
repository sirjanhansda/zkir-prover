//! CPU trace column definitions (~32 columns)

use std::borrow::{Borrow, BorrowMut};

/// CPU trace columns
///
/// Total: 32 columns organized into logical groups
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuColumns<T> {
    // === State (4 columns) ===
    /// Program counter
    pub pc: T,
    /// Next program counter
    pub next_pc: T,
    /// Cycle counter
    pub cycle: T,
    /// Halt flag (1 when halted)
    pub is_halted: T,

    // === Instruction decode (6 columns) ===
    /// 7-bit opcode
    pub opcode: T,
    /// Destination register (rd)
    pub rd: T,
    /// Source register 1 (rs1)
    pub rs1: T,
    /// Source register 2 (rs2)
    pub rs2: T,
    /// Immediate value
    pub imm: T,
    /// Function code (funct3 + funct7 combined)
    pub funct: T,

    // === Operand values (3 columns) ===
    /// Value of rs1 register
    pub rs1_val: T,
    /// Value of rs2 register
    pub rs2_val: T,
    /// Value to write to rd register
    pub rd_val: T,

    // === Opcode flags (12 columns, one-hot) ===
    /// ALU operation (ADD, SUB, MUL, etc.)
    pub is_alu: T,
    /// ALU immediate operation (ADDI, etc.)
    pub is_alu_imm: T,
    /// Branch instruction (BEQ, BNE, etc.)
    pub is_branch: T,
    /// Jump instruction (JAL, JALR)
    pub is_jump: T,
    /// Load instruction (LW, LH, LB)
    pub is_load: T,
    /// Store instruction (SW, SH, SB)
    pub is_store: T,
    /// LUI or AUIPC
    pub is_lui_auipc: T,
    /// System instruction (ECALL, EBREAK)
    pub is_system: T,
    /// ZK custom instruction (ASSERT, COMMIT)
    pub is_zk_custom: T,
    /// ZK I/O instruction (READ, WRITE)
    pub is_zk_io: T,
    /// HALT instruction
    pub is_halt: T,
    /// NOP (padding rows)
    pub is_nop: T,

    // === ALU operation (4 columns) ===
    /// ALU operation selector
    pub alu_op: T,
    /// ALU result
    pub alu_result: T,
    /// Branch condition result (1 if taken)
    pub branch_taken: T,
    /// Comparison result (for SLT, etc.)
    pub comparison_result: T,

    // === Memory (3 columns) ===
    /// Memory address for load/store
    pub mem_addr: T,
    /// Memory value
    pub mem_val: T,
    /// Memory operation type (1 = write, 0 = read)
    pub mem_is_write: T,
}

/// Number of columns in the CPU trace
pub const CPU_NUM_COLUMNS: usize = 32;

impl<T> CpuColumns<T> {
    /// Number of columns in the CPU trace
    pub const NUM_COLUMNS: usize = CPU_NUM_COLUMNS;
}

impl<T: Copy> CpuColumns<T> {
    /// Get all opcode flag columns as a slice
    pub fn opcode_flags(&self) -> [T; 12] {
        [
            self.is_alu,
            self.is_alu_imm,
            self.is_branch,
            self.is_jump,
            self.is_load,
            self.is_store,
            self.is_lui_auipc,
            self.is_system,
            self.is_zk_custom,
            self.is_zk_io,
            self.is_halt,
            self.is_nop,
        ]
    }
}

// Allow converting between CpuColumns<T> and [T; 32]
impl<T> Borrow<CpuColumns<T>> for [T; CPU_NUM_COLUMNS] {
    fn borrow(&self) -> &CpuColumns<T> {
        // Safety: CpuColumns is repr(C) and has exactly NUM_COLUMNS fields of type T
        unsafe { &*(self.as_ptr() as *const CpuColumns<T>) }
    }
}

impl<T> BorrowMut<CpuColumns<T>> for [T; CPU_NUM_COLUMNS] {
    fn borrow_mut(&mut self) -> &mut CpuColumns<T> {
        unsafe { &mut *(self.as_mut_ptr() as *mut CpuColumns<T>) }
    }
}

impl<T> Borrow<[T; CPU_NUM_COLUMNS]> for CpuColumns<T> {
    fn borrow(&self) -> &[T; CPU_NUM_COLUMNS] {
        unsafe { &*(self as *const CpuColumns<T> as *const [T; CPU_NUM_COLUMNS]) }
    }
}

/// ALU operation codes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AluOp {
    Add = 0,
    Sub = 1,
    And = 2,
    Or = 3,
    Xor = 4,
    Sll = 5,  // Shift left logical
    Srl = 6,  // Shift right logical
    Sra = 7,  // Shift right arithmetic
    Slt = 8,  // Set less than (signed)
    Sltu = 9, // Set less than (unsigned)
    Mul = 10,
    Div = 11,
    Divu = 12,
    Rem = 13,
    Remu = 14,
}
