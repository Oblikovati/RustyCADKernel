//! Intersection-curve storage corresponding to `BOPDS_Curve`.
use super::{PaveBlock, PaveBlockHandle, pave_block_handle};
use crate::int_tools::Curve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;

/// Stores one intersection curve and its topology metadata; `BopdsCurve::default()`.
#[derive(Clone, Default)]
pub struct BopdsCurve {
    curve: Curve,
    pave_blocks: Vec<PaveBlockHandle>,
    techno_vertices: Vec<i32>,
    box_bounds: BoundingBox3,
    tolerance: f64,
}

impl BopdsCurve {
    /// Copies an `IntTools_Curve`; `record.set_curve(&curve)`.
    pub fn set_curve(&mut self, curve: &Curve) {
        self.curve = curve.clone();
    }

    /// Borrows the copied intersection curve; `record.curve().tangential_tolerance()`.
    pub fn curve(&self) -> &Curve {
        &self.curve
    }

    /// Replaces curve bounds; `record.set_box(&box_bounds)`.
    pub fn set_box(&mut self, box_bounds: &BoundingBox3) {
        self.box_bounds = box_bounds.clone();
    }

    /// Borrows the curve's box; `record.box_bounds().is_void()`.
    pub fn box_bounds(&self) -> &BoundingBox3 {
        &self.box_bounds
    }

    /// Mutably borrows the curve's box; `record.change_box().set_gap(1e-6)`.
    pub fn change_box(&mut self) -> &mut BoundingBox3 {
        &mut self.box_bounds
    }

    /// Copies a list of shared pave-block handles, preserving order and duplicates.
    /// `record.set_pave_blocks(&[block.clone()])`.
    pub fn set_pave_blocks(&mut self, pave_blocks: &[PaveBlockHandle]) {
        self.pave_blocks.clear();
        self.pave_blocks.extend(pave_blocks.iter().cloned());
    }

    /// Borrows pave blocks in native list order; `record.pave_blocks().len()`.
    pub fn pave_blocks(&self) -> &[PaveBlockHandle] {
        &self.pave_blocks
    }

    /// Mutably borrows pave blocks in native list order; `record.change_pave_blocks().clear()`.
    pub fn change_pave_blocks(&mut self) -> &mut Vec<PaveBlockHandle> {
        &mut self.pave_blocks
    }

    /// Creates one default initial pave block only when the list is empty; `record.init_pave_block1()`.
    pub fn init_pave_block1(&mut self) {
        if self.pave_blocks.is_empty() {
            self.pave_blocks
                .push(pave_block_handle(PaveBlock::default()));
        }
    }

    /// Mutably borrows the first block; reports OCCT's empty-list failure; `record.change_pave_block1()?`.
    pub fn change_pave_block1(&mut self) -> Result<&mut PaveBlockHandle, Failure> {
        self.pave_blocks.first_mut().ok_or_else(|| {
            Failure::with_message(
                "Standard_NoSuchObject",
                "NCollection_List::First on empty BOPDS_Curve pave-block list; expected at least one block",
            )
        })
    }

    /// Borrows technological-vertex indices, retaining duplicate entries.
    pub fn techno_vertices(&self) -> &[i32] {
        &self.techno_vertices
    }

    /// Mutably borrows the technological-vertex sequence; `record.change_techno_vertices().push(4)`.
    pub fn change_techno_vertices(&mut self) -> &mut Vec<i32> {
        &mut self.techno_vertices
    }

    /// Reports whether any pave block has a result edge; `record.has_edge()`.
    pub fn has_edge(&self) -> bool {
        self.pave_blocks.iter().any(|block| {
            block
                .lock()
                .expect("pave-block lock must not be poisoned")
                .has_edge()
        })
    }

    /// Sets the intersection-curve tolerance; `record.set_tolerance(1e-7)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }

    /// Reads the intersection-curve tolerance; `record.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }

    /// Reads tangential tolerance from the stored `IntTools_Curve`; `record.tangential_tolerance()`.
    pub fn tangential_tolerance(&self) -> f64 {
        self.curve.tangential_tolerance()
    }
}
