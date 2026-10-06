//! Protected reset/clean operate on validated canonical headers under the existing engine lock.
use super::{Failure, IncBlockObservation, IncEngine};

/// Copied incremental state, collected while holding the canonical engine's lock.
/// `let snapshot = arena.incremental_extension()?.observation();`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncAllocatorObservation {
    /// Native next-block capacity.
    pub block_size: u32,
    /// Native allocation-attempt count, including failed fresh-block requests.
    pub block_count: u32,
    /// Native enabled fast-path policy (Rust synchronization remains enabled in both modes).
    pub thread_safe: bool,
    /// Native allocation-heap head address, zero for null.
    pub allocation_heap: usize,
    /// Native used-heap head address, zero for null.
    pub used_heap: usize,
    /// Native creation-order head address, zero for null.
    pub ordered_blocks: usize,
    /// Actual headers traversed in creation-reverse order; this copy does not own topology.
    pub blocks: Vec<IncBlockObservation>,
}

impl IncEngine {
    pub(in crate::ncollection::allocator) fn observation(&self) -> IncAllocatorObservation {
        IncAllocatorObservation {
            block_size: self.block_size,
            block_count: self.block_count,
            thread_safe: self.thread_safe_policy,
            allocation_heap: self
                .allocation_heap
                .map_or(0, |link| link.observation().address),
            used_heap: self.used_heap.map_or(0, |link| link.observation().address),
            ordered_blocks: self
                .ordered_blocks
                .map_or(0, |link| link.observation().address),
            blocks: self.block_observations(),
        }
    }

    fn block_observations(&self) -> Vec<IncBlockObservation> {
        let mut blocks = Vec::new();
        let mut iterator = self.ordered_blocks;
        while let Some(block) = iterator {
            blocks.push(block.observation());
            iterator = block.ordered_next();
        }
        blocks
    }

    pub(in crate::ncollection::allocator) fn reset_block(
        &mut self,
        address: usize,
    ) -> Result<(), Failure> {
        self.require_no_live("resetBlock")?;
        let mut iterator = self.ordered_blocks;
        while let Some(block) = iterator {
            if block.observation().address == address {
                block.reset();
                return Ok(());
            }
            iterator = block.ordered_next();
        }
        Err(Failure::with_message(
            "Standard_NoSuchObject",
            &format!("block address {address:#x}: expected a live NCollection_IncAllocator header"),
        ))
    }

    pub(in crate::ncollection::allocator) fn clean_checked(&mut self) -> Result<(), Failure> {
        self.require_no_live("clean")?;
        self.clean();
        Ok(())
    }

    pub(super) fn require_no_live(&self, operation: &str) -> Result<(), Failure> {
        if self.live == 0 {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_ProgramError",
            &format!(
                "{} live allocations: expected 0 before NCollection_IncAllocator::{operation}",
                self.live
            ),
            "",
        ))
    }
}
