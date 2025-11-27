//! CPU trace generation from execution trace

use std::borrow::BorrowMut;

use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;

use super::columns::{CpuColumns, CPU_NUM_COLUMNS};
use crate::trace::{ExecutionTrace, Step};

/// Opcode constants matching ZK IR spec
pub mod opcodes {
    // R-type (register-register)
    pub const OP_ALU: u8 = 0b0110011;
    // I-type (immediate)
    pub const OP_ALU_IMM: u8 = 0b0010011;
    pub const OP_LOAD: u8 = 0b0000011;
    pub const OP_JALR: u8 = 0b1100111;
    // S-type (store)
    pub const OP_STORE: u8 = 0b0100011;
    // B-type (branch)
    pub const OP_BRANCH: u8 = 0b1100011;
    // U-type (upper immediate)
    pub const OP_LUI: u8 = 0b0110111;
    pub const OP_AUIPC: u8 = 0b0010111;
    // J-type (jump)
    pub const OP_JAL: u8 = 0b1101111;
    // System
    pub const OP_SYSTEM: u8 = 0b1110011;
    // ZK custom
    pub const OP_ZK_CUSTOM: u8 = 0b0001011;
    pub const OP_ZK_IO: u8 = 0b0101011;
    pub const OP_HALT: u8 = 0b1111111;
}

/// Generate the CPU trace from an execution trace
pub fn generate_cpu_trace<F: Field>(trace: &ExecutionTrace) -> RowMajorMatrix<F> {
    let num_steps = trace.steps.len();
    // Pad to next power of 2
    let trace_len = num_steps.next_power_of_two().max(2);

    let mut values = vec![F::ZERO; trace_len * CpuColumns::<F>::NUM_COLUMNS];

    for (i, step) in trace.steps.iter().enumerate() {
        let row_offset = i * CpuColumns::<F>::NUM_COLUMNS;
        let row = &mut values[row_offset..row_offset + CpuColumns::<F>::NUM_COLUMNS];
        populate_row_from_step::<F>(row, step, i == num_steps - 1);
    }

    // Fill padding rows with NOP
    for i in num_steps..trace_len {
        let row_offset = i * CpuColumns::<F>::NUM_COLUMNS;
        let row = &mut values[row_offset..row_offset + CpuColumns::<F>::NUM_COLUMNS];
        populate_nop_row::<F>(row, i as u64);
    }

    RowMajorMatrix::new(values, CpuColumns::<F>::NUM_COLUMNS)
}

fn populate_row_from_step<F: Field>(row: &mut [F], step: &Step, is_last: bool) {
    let row_arr: &mut [F; CPU_NUM_COLUMNS] = row.try_into().unwrap();
    let cols: &mut CpuColumns<F> = row_arr.borrow_mut();

    // State
    cols.pc = F::from_canonical_u32(step.pc);
    cols.cycle = F::from_canonical_u64(step.cycle);

    // Instruction decode
    cols.opcode = F::from_canonical_u32(step.opcode as u32);
    cols.rd = F::from_canonical_u32(step.rd as u32);
    cols.rs1 = F::from_canonical_u32(step.rs1 as u32);
    cols.rs2 = F::from_canonical_u32(step.rs2 as u32);
    cols.imm = F::from_canonical_u32(step.imm as u32);
    cols.funct = F::from_canonical_u32(step.funct as u32);

    // Operand values
    cols.rs1_val = F::from_canonical_u32(step.registers[step.rs1 as usize]);
    cols.rs2_val = F::from_canonical_u32(step.registers[step.rs2 as usize]);
    cols.rd_val = F::from_canonical_u32(step.registers[step.rd as usize]);

    // Set opcode flags (one-hot)
    reset_flags(cols);
    match step.opcode {
        opcodes::OP_ALU => cols.is_alu = F::ONE,
        opcodes::OP_ALU_IMM => cols.is_alu_imm = F::ONE,
        opcodes::OP_BRANCH => cols.is_branch = F::ONE,
        opcodes::OP_JAL | opcodes::OP_JALR => cols.is_jump = F::ONE,
        opcodes::OP_LOAD => cols.is_load = F::ONE,
        opcodes::OP_STORE => cols.is_store = F::ONE,
        opcodes::OP_LUI | opcodes::OP_AUIPC => cols.is_lui_auipc = F::ONE,
        opcodes::OP_SYSTEM => cols.is_system = F::ONE,
        opcodes::OP_ZK_CUSTOM => cols.is_zk_custom = F::ONE,
        opcodes::OP_ZK_IO => cols.is_zk_io = F::ONE,
        opcodes::OP_HALT => {
            cols.is_halt = F::ONE;
            cols.is_halted = F::ONE;
        }
        _ => cols.is_nop = F::ONE,
    }

    // Compute next_pc (simplified - actual implementation would check all cases)
    cols.next_pc = F::from_canonical_u32(step.pc.wrapping_add(4));

    // ALU operation (would need to decode from funct)
    cols.alu_op = F::ZERO; // Placeholder
    cols.alu_result = cols.rd_val; // Simplified

    // Handle halt
    if is_last || step.opcode == opcodes::OP_HALT {
        cols.is_halted = F::ONE;
        cols.next_pc = cols.pc;
    }
}

fn populate_nop_row<F: Field>(row: &mut [F], cycle: u64) {
    let row_arr: &mut [F; CPU_NUM_COLUMNS] = row.try_into().unwrap();
    let cols: &mut CpuColumns<F> = row_arr.borrow_mut();

    cols.cycle = F::from_canonical_u64(cycle);
    reset_flags(cols);
    cols.is_nop = F::ONE;
}

fn reset_flags<F: Field>(cols: &mut CpuColumns<F>) {
    cols.is_alu = F::ZERO;
    cols.is_alu_imm = F::ZERO;
    cols.is_branch = F::ZERO;
    cols.is_jump = F::ZERO;
    cols.is_load = F::ZERO;
    cols.is_store = F::ZERO;
    cols.is_lui_auipc = F::ZERO;
    cols.is_system = F::ZERO;
    cols.is_zk_custom = F::ZERO;
    cols.is_zk_io = F::ZERO;
    cols.is_halt = F::ZERO;
    cols.is_nop = F::ZERO;
}
