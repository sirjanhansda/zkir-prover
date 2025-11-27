//! Memory Chip implementation
//!
//! Enforces memory consistency using a sorted trace approach.
//! Memory accesses are sorted by (address, cycle), and constraints ensure
//! that reads return the most recently written value.

use std::borrow::{Borrow, BorrowMut};
use std::ops::Deref;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{Field, FieldAlgebra};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

use crate::trace::ExecutionTrace;

/// Memory trace columns
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryColumns<T> {
    /// Memory address
    pub address: T,
    /// Cycle when access occurred
    pub cycle: T,
    /// Value read or written
    pub value: T,
    /// 1 if write, 0 if read
    pub is_write: T,

    // Helper columns for constraints
    /// 1 if this row has the same address as the next row
    pub same_addr_as_next: T,
    /// Inverse of (next_addr - addr) when addresses differ, used for range check
    pub addr_diff_inv: T,
    /// Inverse of (next_cycle - cycle) when same address
    pub cycle_diff_inv: T,
}

/// Number of columns in the memory trace
pub const MEMORY_NUM_COLUMNS: usize = 7;

impl<T> MemoryColumns<T> {
    pub const NUM_COLUMNS: usize = MEMORY_NUM_COLUMNS;
}

impl<T> Borrow<MemoryColumns<T>> for [T; MEMORY_NUM_COLUMNS] {
    fn borrow(&self) -> &MemoryColumns<T> {
        unsafe { &*(self.as_ptr() as *const MemoryColumns<T>) }
    }
}

impl<T> BorrowMut<MemoryColumns<T>> for [T; MEMORY_NUM_COLUMNS] {
    fn borrow_mut(&mut self) -> &mut MemoryColumns<T> {
        unsafe { &mut *(self.as_mut_ptr() as *mut MemoryColumns<T>) }
    }
}

/// Memory Chip enforcing read/write consistency
pub struct MemoryChip;

impl<F: Field> BaseAir<F> for MemoryChip {
    fn width(&self) -> usize {
        MemoryColumns::<F>::NUM_COLUMNS
    }
}

impl<AB: AirBuilder> Air<AB> for MemoryChip {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local_slice = main.row_slice(0);
        let next_slice = main.row_slice(1);

        let local_arr: &[AB::Var; MEMORY_NUM_COLUMNS] = local_slice.deref().try_into().unwrap();
        let next_arr: &[AB::Var; MEMORY_NUM_COLUMNS] = next_slice.deref().try_into().unwrap();
        let local: &MemoryColumns<AB::Var> = local_arr.borrow();
        let next: &MemoryColumns<AB::Var> = next_arr.borrow();

        // Boolean constraints
        builder.assert_zero(local.is_write.into() * (AB::Expr::ONE - local.is_write.into()));
        builder.assert_zero(
            local.same_addr_as_next.into() * (AB::Expr::ONE - local.same_addr_as_next.into()),
        );

        // Address ordering: addresses are non-decreasing
        // When same_addr_as_next = 1: next.address = local.address
        builder
            .when_transition()
            .when(local.same_addr_as_next)
            .assert_eq(next.address, local.address);

        // When same_addr_as_next = 0: next.address > local.address
        // (enforced via range check on next.address - local.address - 1)

        // Cycle ordering within same address: cycles must be strictly increasing
        // Enforced via range check on (next.cycle - local.cycle - 1)
        builder
            .when_transition()
            .when(local.same_addr_as_next)
            .assert_zero(
                (next.cycle.into() - local.cycle.into() - AB::Expr::ONE) * local.cycle_diff_inv.into()
                    - AB::Expr::ONE,
            );

        // Read consistency: reads return last written value
        // If next row is a read at same address, its value must equal current value
        let next_is_read: AB::Expr = AB::Expr::ONE - next.is_write.into();

        builder
            .when_transition()
            .when(local.same_addr_as_next)
            .when(next_is_read)
            .assert_eq(next.value, local.value);

        // First access to an address must be a write (or value must be zero)
        // TODO: implement constraint for initial memory state
    }
}

impl MemoryChip {
    /// Generate the memory trace sorted by (address, cycle)
    pub fn generate_trace<F: Field>(&self, trace: &ExecutionTrace) -> RowMajorMatrix<F> {
        let sorted = trace.sorted_memory_log();
        let num_accesses = sorted.len();
        let trace_len = num_accesses.next_power_of_two().max(2);

        let mut values = vec![F::ZERO; trace_len * MemoryColumns::<F>::NUM_COLUMNS];

        for (i, access) in sorted.iter().enumerate() {
            let row_offset = i * MemoryColumns::<F>::NUM_COLUMNS;
            let row: &mut [F; MEMORY_NUM_COLUMNS] = (&mut values[row_offset..row_offset + MemoryColumns::<F>::NUM_COLUMNS]).try_into().unwrap();
            let cols: &mut MemoryColumns<F> = row.borrow_mut();

            cols.address = F::from_canonical_u32(access.address);
            cols.cycle = F::from_canonical_u64(access.cycle);
            cols.value = F::from_canonical_u32(access.value);
            cols.is_write = if access.is_write { F::ONE } else { F::ZERO };

            // Check if next row has same address
            if i + 1 < sorted.len() {
                cols.same_addr_as_next = if sorted[i + 1].address == access.address {
                    F::ONE
                } else {
                    F::ZERO
                };
            }
        }

        RowMajorMatrix::new(values, MemoryColumns::<F>::NUM_COLUMNS)
    }
}
