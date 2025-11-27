//! CPU AIR (Algebraic Intermediate Representation) constraints

use std::borrow::Borrow;
use std::ops::Deref;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

use super::columns::{CpuColumns, CPU_NUM_COLUMNS};

/// CPU Chip for ZK IR execution
pub struct CpuChip;

impl<F: Field> BaseAir<F> for CpuChip {
    fn width(&self) -> usize {
        CpuColumns::<F>::NUM_COLUMNS
    }
}

impl<AB: AirBuilder> Air<AB> for CpuChip {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local_slice = main.row_slice(0);
        let next_slice = main.row_slice(1);

        // Convert row slices to typed columns
        let local_arr: &[AB::Var; CPU_NUM_COLUMNS] = local_slice.deref().try_into().unwrap();
        let next_arr: &[AB::Var; CPU_NUM_COLUMNS] = next_slice.deref().try_into().unwrap();
        let local: &CpuColumns<AB::Var> = local_arr.borrow();
        let next: &CpuColumns<AB::Var> = next_arr.borrow();

        // Opcode flags must be one-hot (exactly one set)
        let flag_sum = local.is_alu.into()
            + local.is_alu_imm.into()
            + local.is_branch.into()
            + local.is_jump.into()
            + local.is_load.into()
            + local.is_store.into()
            + local.is_lui_auipc.into()
            + local.is_system.into()
            + local.is_zk_custom.into()
            + local.is_zk_io.into()
            + local.is_halt.into()
            + local.is_nop.into();

        builder.assert_one(flag_sum);

        // Boolean constraints for flags
        self.assert_bool(builder, local.is_alu);
        self.assert_bool(builder, local.is_alu_imm);
        self.assert_bool(builder, local.is_branch);
        self.assert_bool(builder, local.is_jump);
        self.assert_bool(builder, local.is_load);
        self.assert_bool(builder, local.is_store);
        self.assert_bool(builder, local.is_lui_auipc);
        self.assert_bool(builder, local.is_system);
        self.assert_bool(builder, local.is_zk_custom);
        self.assert_bool(builder, local.is_zk_io);
        self.assert_bool(builder, local.is_halt);
        self.assert_bool(builder, local.is_nop);
        self.assert_bool(builder, local.is_halted);
        self.assert_bool(builder, local.branch_taken);
        self.assert_bool(builder, local.mem_is_write);

        // ALU operations write result to rd
        builder
            .when(local.is_alu)
            .assert_eq(local.rd_val, local.alu_result);

        // ALU immediate operations
        builder
            .when(local.is_alu_imm)
            .assert_eq(local.rd_val, local.alu_result);

        // PC transitions
        let pc_plus_4: AB::Expr = local.pc.into() + AB::Expr::from_canonical_u32(4);
        let pc_plus_imm: AB::Expr = local.pc.into() + local.imm.into();

        // Sequential instructions: next_pc = pc + 4
        let is_sequential: AB::Expr = local.is_alu.into()
            + local.is_alu_imm.into()
            + local.is_load.into()
            + local.is_store.into()
            + local.is_lui_auipc.into()
            + local.is_zk_custom.into()
            + local.is_zk_io.into();

        builder
            .when(is_sequential.clone())
            .assert_eq(local.next_pc, pc_plus_4.clone());

        // Branch logic: next_pc = branch_taken ? pc+imm : pc+4
        // When branch taken: next_pc = pc + imm
        // When not taken: next_pc = pc + 4
        let branch_target: AB::Expr = local.branch_taken.into() * pc_plus_imm.clone()
            + (AB::Expr::ONE - local.branch_taken.into()) * pc_plus_4.clone();

        builder
            .when(local.is_branch)
            .assert_eq(local.next_pc, branch_target);

        // Jump instructions save return address in rd
        builder
            .when(local.is_jump)
            .assert_eq(local.rd_val, pc_plus_4.clone());

        // JAL: next_pc = pc + imm (handled by funct disambiguation)
        // JALR: next_pc = rs1_val + imm (need additional constraint)

        // Halt behavior
        builder
            .when(local.is_halt)
            .assert_eq(local.next_pc, local.pc);

        // Once halted, stay halted
        builder
            .when(local.is_halted)
            .assert_one(next.is_halted);

        // PC continuity: next.pc = local.next_pc (except for padding)
        let not_halted_or_nop: AB::Expr =
            AB::Expr::ONE - local.is_halt.into() - local.is_nop.into();

        builder
            .when_transition()
            .when(not_halted_or_nop)
            .assert_eq(next.pc, local.next_pc);

        // Cycle counter increments each non-NOP step
        builder
            .when_transition()
            .when(AB::Expr::ONE - local.is_nop.into())
            .assert_eq(next.cycle, local.cycle.into() + AB::Expr::ONE);

        // Register r0 is always zero (enforced via register file lookup)

        // Memory constraints (linked via permutation with memory chip)
        // Load: mem_is_write = 0, mem_addr = rs1_val + imm, rd_val = mem_val
        builder.when(local.is_load).assert_zero(local.mem_is_write);

        // Store: mem_is_write = 1, mem_addr = rs1_val + imm, mem_val = rs2_val
        builder.when(local.is_store).assert_one(local.mem_is_write);
    }
}

impl CpuChip {
    /// Assert that a value is boolean (0 or 1)
    fn assert_bool<AB: AirBuilder>(&self, builder: &mut AB, val: AB::Var) {
        // val * (1 - val) = 0
        builder.assert_zero(val.into() * (AB::Expr::ONE - val.into()));
    }

    /// Generate the trace matrix for this chip
    pub fn generate_trace<F: Field>(
        &self,
        trace: &crate::ExecutionTrace,
    ) -> RowMajorMatrix<F> {
        super::trace::generate_cpu_trace(trace)
    }
}
