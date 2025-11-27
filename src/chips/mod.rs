//! Chip implementations for the ZK IR prover
//!
//! The prover uses a multi-chip architecture:
//! - CPU Chip: Main execution with ~32 trace columns
//! - Memory Chip: Memory consistency via sorted trace
//! - Range Check Chip: 32-bit value validation
//! - Syscall Chips: Dedicated chips for crypto operations

pub mod cpu;
pub mod memory;
pub mod range;
pub mod syscall;

pub use cpu::CpuChip;
pub use memory::MemoryChip;
pub use range::RangeCheckChip;
pub use syscall::{Poseidon2Chip, Sha256Chip};
