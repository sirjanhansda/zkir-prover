//! Syscall Chips for cryptographic operations
//!
//! Dedicated chips for expensive cryptographic operations:
//! - Poseidon2: Hash function (~200 constraints per hash)
//! - SHA256: Hash function (~20,000 constraints per block)

mod poseidon;
mod sha256;

pub use poseidon::Poseidon2Chip;
pub use sha256::Sha256Chip;

/// Common interface for syscall chips
pub trait SyscallChip {
    /// Syscall code this chip handles
    fn syscall_code(&self) -> u32;

    /// Number of constraints per invocation
    fn constraints_per_call(&self) -> usize;
}
