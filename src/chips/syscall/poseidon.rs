//! Poseidon2 Chip implementation
//!
//! Implements Poseidon2 hash function constraints.
//! ~200 constraints per hash invocation.

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

use super::SyscallChip;
use crate::trace::{SyscallCode, SyscallRecord};

/// Poseidon2 state width
pub const POSEIDON2_WIDTH: usize = 16;
/// Number of full rounds
pub const POSEIDON2_FULL_ROUNDS: usize = 8;
/// Number of partial rounds
pub const POSEIDON2_PARTIAL_ROUNDS: usize = 14;

/// Poseidon2 trace columns
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Poseidon2Columns<T> {
    /// Cycle when syscall was invoked
    pub cycle: T,
    /// Round number (0 to FULL_ROUNDS + PARTIAL_ROUNDS)
    pub round: T,
    /// Is this a full round?
    pub is_full_round: T,
    /// Current state (16 field elements)
    pub state: [T; POSEIDON2_WIDTH],
    /// State after S-box application
    pub state_after_sbox: [T; POSEIDON2_WIDTH],
}

impl<T: Default + Copy> Default for Poseidon2Columns<T> {
    fn default() -> Self {
        Self {
            cycle: T::default(),
            round: T::default(),
            is_full_round: T::default(),
            state: [T::default(); POSEIDON2_WIDTH],
            state_after_sbox: [T::default(); POSEIDON2_WIDTH],
        }
    }
}

impl<T> Poseidon2Columns<T> {
    pub const NUM_COLUMNS: usize = 3 + POSEIDON2_WIDTH * 2;
}

/// Poseidon2 Chip for hash operations
pub struct Poseidon2Chip {
    /// Round constants
    pub round_constants: Vec<[u32; POSEIDON2_WIDTH]>,
    /// MDS matrix (internal linear layer)
    pub mds_matrix: [[u32; POSEIDON2_WIDTH]; POSEIDON2_WIDTH],
}

impl Default for Poseidon2Chip {
    fn default() -> Self {
        Self::new()
    }
}

impl Poseidon2Chip {
    pub fn new() -> Self {
        // Initialize with placeholder constants
        // Real implementation would use proper Poseidon2 constants for Baby Bear
        let num_rounds = POSEIDON2_FULL_ROUNDS + POSEIDON2_PARTIAL_ROUNDS;
        let round_constants = vec![[0u32; POSEIDON2_WIDTH]; num_rounds];
        let mds_matrix = [[0u32; POSEIDON2_WIDTH]; POSEIDON2_WIDTH];

        Self {
            round_constants,
            mds_matrix,
        }
    }
}

impl SyscallChip for Poseidon2Chip {
    fn syscall_code(&self) -> u32 {
        SyscallCode::Poseidon2 as u32
    }

    fn constraints_per_call(&self) -> usize {
        200
    }
}

impl<F: Field> BaseAir<F> for Poseidon2Chip {
    fn width(&self) -> usize {
        Poseidon2Columns::<F>::NUM_COLUMNS
    }
}

impl<AB: AirBuilder> Air<AB> for Poseidon2Chip {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let _local = main.row_slice(0);
        let _next = main.row_slice(1);

        // Poseidon2 round constraints:
        // 1. S-box: x^7 (or x^5 depending on field)
        // 2. Linear layer (MDS matrix multiplication)
        // 3. Add round constants

        // For Baby Bear, we use S-box x^7
        // state_after_sbox[i] = state[i]^7

        // Full rounds: apply S-box to all elements
        // Partial rounds: apply S-box only to first element

        // This is a simplified placeholder - full implementation would have:
        // - S-box constraints for each element
        // - MDS matrix multiplication constraints
        // - Round constant addition
        // - Transition constraints between rounds
    }
}

impl Poseidon2Chip {
    /// Generate trace for Poseidon2 syscalls
    pub fn generate_trace<F: Field>(&self, syscalls: &[SyscallRecord]) -> RowMajorMatrix<F> {
        let poseidon_calls: Vec<_> = syscalls
            .iter()
            .filter(|s| s.code == SyscallCode::Poseidon2 as u32)
            .collect();

        let num_rounds = POSEIDON2_FULL_ROUNDS + POSEIDON2_PARTIAL_ROUNDS;
        let rows_per_call = num_rounds;
        let total_rows = poseidon_calls.len() * rows_per_call;
        let trace_len = total_rows.next_power_of_two().max(2);

        let values = vec![F::ZERO; trace_len * Poseidon2Columns::<F>::NUM_COLUMNS];

        // TODO: Populate trace with actual Poseidon2 computation
        // For each syscall:
        //   - Initialize state from inputs
        //   - Compute each round
        //   - Store intermediate states

        RowMajorMatrix::new(values, Poseidon2Columns::<F>::NUM_COLUMNS)
    }
}
