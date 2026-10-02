//! A pair of pave-block handles and their interference metadata.
use super::PaveBlockHandle;

/// `BOPDS_CoupleOfPaveBlocks`; retains two nullable handles and scalar indices.
#[derive(Clone)]
pub struct CoupleOfPaveBlocks {
    interference_index: i32,
    index: i32,
    pave_blocks: [Option<PaveBlockHandle>; 2],
    tolerance: f64,
}

impl CoupleOfPaveBlocks {
    /// Creates the native default record; `CoupleOfPaveBlocks::default().index() == -1`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a record with both nullable handles; `CoupleOfPaveBlocks::with_pave_blocks(a, b)`.
    pub fn with_pave_blocks(
        first: Option<PaveBlockHandle>,
        second: Option<PaveBlockHandle>,
    ) -> Self {
        let mut value = Self::default();
        value.set_pave_blocks(first, second);
        value
    }

    /// Sets the record index; `value.set_index(7)`.
    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }

    /// Reads the record index; `CoupleOfPaveBlocks::default().index() == -1`.
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Sets the interference index; `value.set_interference_index(9)`.
    pub fn set_interference_index(&mut self, index: i32) {
        self.interference_index = index;
    }

    /// Reads the interference index; `CoupleOfPaveBlocks::default().interference_index() == -1`.
    pub fn interference_index(&self) -> i32 {
        self.interference_index
    }

    /// Replaces both nullable handles; `value.set_pave_blocks(Some(a), Some(b))`.
    pub fn set_pave_blocks(
        &mut self,
        first: Option<PaveBlockHandle>,
        second: Option<PaveBlockHandle>,
    ) {
        self.pave_blocks = [first, second];
    }

    /// Copies both nullable handles into the legacy output parameters.
    pub fn pave_blocks_into(
        &self,
        first: &mut Option<PaveBlockHandle>,
        second: &mut Option<PaveBlockHandle>,
    ) {
        *first = self.pave_blocks[0].clone();
        *second = self.pave_blocks[1].clone();
    }

    /// Replaces the first nullable handle; `value.set_pave_block1(Some(block))`.
    pub fn set_pave_block1(&mut self, block: Option<PaveBlockHandle>) {
        self.pave_blocks[0] = block;
    }

    /// Borrows the first nullable handle; `value.pave_block1()`.
    pub fn pave_block1(&self) -> Option<&PaveBlockHandle> {
        self.pave_blocks[0].as_ref()
    }

    /// Replaces the second nullable handle; `value.set_pave_block2(Some(block))`.
    pub fn set_pave_block2(&mut self, block: Option<PaveBlockHandle>) {
        self.pave_blocks[1] = block;
    }

    /// Borrows the second nullable handle; `value.pave_block2()`.
    pub fn pave_block2(&self) -> Option<&PaveBlockHandle> {
        self.pave_blocks[1].as_ref()
    }

    /// Sets the associated tolerance without validation; `value.set_tolerance(-1.0)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }

    /// Reads the associated tolerance; `CoupleOfPaveBlocks::default().tolerance() == 0.0`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

impl Default for CoupleOfPaveBlocks {
    fn default() -> Self {
        Self {
            interference_index: -1,
            index: -1,
            pave_blocks: [None, None],
            tolerance: 0.0,
        }
    }
}
