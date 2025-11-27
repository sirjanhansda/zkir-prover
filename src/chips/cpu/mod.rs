//! CPU Chip implementation
//!
//! The CPU chip handles ZK IR instruction execution with ~32 trace columns.

mod air;
mod columns;
mod trace;

pub use air::CpuChip;
pub use columns::CpuColumns;
pub use trace::generate_cpu_trace;
