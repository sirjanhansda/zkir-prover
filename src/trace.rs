//! Execution trace types for the ZK IR VM

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::NUM_REGISTERS;

/// A single step of execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    /// Program counter
    pub pc: u32,
    /// Cycle number
    pub cycle: u64,
    /// Opcode
    pub opcode: u8,
    /// Destination register
    pub rd: u8,
    /// Source register 1
    pub rs1: u8,
    /// Source register 2
    pub rs2: u8,
    /// Immediate value
    pub imm: i32,
    /// Function code (funct3 + funct7)
    pub funct: u8,
    /// Register file state after this step
    pub registers: [u32; NUM_REGISTERS],
}

/// A memory access record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryAccess {
    /// Memory address
    pub address: u32,
    /// Cycle when access occurred
    pub cycle: u64,
    /// Value read or written
    pub value: u32,
    /// True if write, false if read
    pub is_write: bool,
}

/// A syscall record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyscallRecord {
    /// Syscall code
    pub code: u32,
    /// Cycle when syscall was invoked
    pub cycle: u64,
    /// Input data (depends on syscall type)
    pub inputs: Vec<u32>,
    /// Output data (depends on syscall type)
    pub outputs: Vec<u32>,
}

/// Syscall codes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SyscallCode {
    Poseidon2 = 0x01,
    Keccak256 = 0x02,
    Sha256 = 0x03,
    Blake3 = 0x04,
    EcdsaVerify = 0x10,
    Ed25519Verify = 0x11,
    BigintAdd = 0x20,
    BigintMul = 0x21,
}

/// Complete execution trace
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Program bytecode hash
    pub program_hash: [u8; 32],
    /// Public inputs
    pub inputs: Vec<u32>,
    /// Public outputs
    pub outputs: Vec<u32>,
    /// Execution steps
    pub steps: Vec<Step>,
    /// Memory accesses (in execution order)
    pub memory_log: Vec<MemoryAccess>,
    /// Syscall records
    pub syscalls: Vec<SyscallRecord>,
}

impl ExecutionTrace {
    /// Create a new empty execution trace
    pub fn new(program_hash: [u8; 32]) -> Self {
        Self {
            program_hash,
            inputs: Vec::new(),
            outputs: Vec::new(),
            steps: Vec::new(),
            memory_log: Vec::new(),
            syscalls: Vec::new(),
        }
    }

    /// Load an execution trace from a file
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        let trace: Self = bincode::deserialize(&data)?;
        Ok(trace)
    }

    /// Save the execution trace to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let data = bincode::serialize(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Number of execution cycles
    pub fn num_cycles(&self) -> u64 {
        self.steps.last().map(|s| s.cycle).unwrap_or(0)
    }

    /// Get memory accesses sorted by (address, cycle) for the memory chip
    pub fn sorted_memory_log(&self) -> Vec<MemoryAccess> {
        let mut sorted = self.memory_log.clone();
        sorted.sort_by(|a, b| {
            a.address.cmp(&b.address).then(a.cycle.cmp(&b.cycle))
        });
        sorted
    }

    /// Get syscalls by type
    pub fn syscalls_by_code(&self, code: SyscallCode) -> Vec<&SyscallRecord> {
        self.syscalls
            .iter()
            .filter(|s| s.code == code as u32)
            .collect()
    }
}
