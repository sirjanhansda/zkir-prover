//! ZK IR Prover v2.1
//!
//! STARK prover for ZK IR using Plonky3 with Baby Bear field.
//!
//! # Architecture
//!
//! The prover uses a multi-chip design:
//! - CPU Chip: Executes ZK IR instructions (~32 trace columns)
//! - Memory Chip: Enforces memory consistency
//! - Range Check Chip: Validates 32-bit values
//! - Syscall Chips: Dedicated chips for cryptographic operations

pub mod chips;
pub mod machine;
pub mod proof;
pub mod prover;
pub mod trace;
pub mod verifier;

pub use machine::ZkIrMachine;
pub use proof::{Proof, PublicInputs};
pub use prover::{Prover, ProverConfig};
pub use trace::ExecutionTrace;
pub use verifier::Verifier;

use p3_baby_bear::BabyBear;

/// The field type used throughout the prover (Baby Bear: p = 2^31 - 2^27 + 1)
pub type F = BabyBear;

/// Baby Bear prime: 2^31 - 2^27 + 1 = 2013265921
pub const BABY_BEAR_PRIME: u32 = 2013265921;

/// Number of registers in the ZK IR VM
pub const NUM_REGISTERS: usize = 32;

/// Word size in bytes
pub const WORD_SIZE: usize = 4;
