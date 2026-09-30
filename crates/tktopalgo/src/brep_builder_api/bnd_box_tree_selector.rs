// Rust translation of BRepBuilderAPI_BndBoxTreeSelector.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_BndBoxTreeSelector`: the UB-tree selector of the boxes that meet a
//! current box.
use rustycad_tkernel::ncollection::{List, TreeSelector};
use rustycad_tkmath::bnd::BoundingBox3;

/// `BRepBuilderAPI_BndBoxTreeSelector` (docs/topalgo-design.md section 11): rejects the
/// branches whose box is out of the current box and collects the accepted objects (the
/// indices of the boxes) in visiting order.
///
/// ```
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIBndBoxTreeSelector;
/// let selector = BRepBuilderAPIBndBoxTreeSelector::new();
/// assert!(selector.res_ind().is_empty());
/// ```
#[derive(Clone, Default)]
pub struct BRepBuilderAPIBndBoxTreeSelector {
    my_res_ind: List<i32>,
    my_box: BoundingBox3,
}

impl BRepBuilderAPIBndBoxTreeSelector {
    /// `BRepBuilderAPI_BndBoxTreeSelector()`: a void current box and no index;
    /// `BRepBuilderAPIBndBoxTreeSelector::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Reject(theBox)`: whether `bounds` is out of the current box;
    /// `selector.reject_box(&bounds)`.
    pub fn reject_box(&self, bounds: &BoundingBox3) -> bool {
        self.my_box.is_out_box(bounds)
    }

    /// `Accept(theObj)`: records the object and accepts it;
    /// `selector.accept_object(3)`.
    pub fn accept_object(&mut self, object: i32) -> bool {
        self.my_res_ind.append(object);
        true
    }

    /// `ClearResList()`: forgets the accepted objects; `selector.clear_res_list()`.
    pub fn clear_res_list(&mut self) {
        self.my_res_ind.clear();
    }

    /// `SetCurrent(theBox)`: the box the tree boxes are compared with;
    /// `selector.set_current(bounds)`.
    pub fn set_current(&mut self, bounds: BoundingBox3) {
        self.my_box = bounds;
    }

    /// `ResInd()`: the accepted objects, in visiting order;
    /// `selector.res_ind().extent()`.
    pub fn res_ind(&self) -> &List<i32> {
        &self.my_res_ind
    }
}

impl TreeSelector<i32, BoundingBox3> for BRepBuilderAPIBndBoxTreeSelector {
    fn reject(&self, bounds: &BoundingBox3) -> bool {
        self.reject_box(bounds)
    }
    fn accept(&mut self, object: &i32) -> bool {
        self.accept_object(*object)
    }
    // The base Selector's `Stop()` reads `myStop`, which this class never sets.
    fn stop(&self) -> bool {
        false
    }
}
