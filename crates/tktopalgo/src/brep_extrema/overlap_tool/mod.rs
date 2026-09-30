// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_OverlapTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_OverlapTool`: the pairs of overlapping mesh elements of two triangle
//! sets, found by a parallel descent of their BVH trees. The triangle tests are in
//! `geometry`.
mod geometry;

pub(crate) use geometry::projection_axes;

use super::{BRepExtremaElementFilter, BRepExtremaFilterResult, BRepExtremaTriangleSet};
use geometry::{BoundingPrism, overlap_boxes, prisms_intersected, triangles_intersected};
use rustycad_tkernel::ncollection::{DataMap, PackedMap};
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bvh::{BvhBaseTraverse, BvhPairTraverse, BvhTree, BvhVec3d};

/// `NCollection_DataMap<int, TColStd_PackedMapOfInteger>`: the overlapped sub-shapes of
/// each sub-shape (by their indices in the shape lists, from 0).
pub type BRepExtremaOverlapMap = DataMap<i32, PackedMap<i32>>;

/// `getSetOfFaces`: the set of `face_idx`, bound empty first when missing.
fn set_of_faces(faces: &mut BRepExtremaOverlapMap, face_idx: i32) -> &mut PackedMap<i32> {
    faces.bound(face_idx, PackedMap::default())
}

/// `BRepExtrema_OverlapTool` (docs/topalgo-design.md section 13): the faces of two
/// triangle sets whose triangles overlap (exactly, or within a tolerance with the
/// bounding prisms of the triangles), with an optional filter of element pairs.
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaOverlapTool;
/// let tool = BRepExtremaOverlapTool::new();
/// assert!(!tool.is_done());
/// assert_eq!(tool.overlap_sub_shapes1().extent(), 0);
/// ```
pub struct BRepExtremaOverlapTool {
    my_set1: Handle<BRepExtremaTriangleSet>,
    my_set2: Handle<BRepExtremaTriangleSet>,
    my_filter: Option<Box<dyn BRepExtremaElementFilter>>,
    my_overlap_sub_shapes1: BRepExtremaOverlapMap,
    my_overlap_sub_shapes2: BRepExtremaOverlapMap,
    my_is_done: bool,
    my_tolerance: f64,
}

impl Default for BRepExtremaOverlapTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaOverlapTool {
    /// `BRepExtrema_OverlapTool()`: no sets, no filter, tolerance 0;
    /// `BRepExtremaOverlapTool::new()`.
    pub fn new() -> Self {
        Self {
            my_set1: Handle::default(),
            my_set2: Handle::default(),
            my_filter: None,
            my_overlap_sub_shapes1: DataMap::default(),
            my_overlap_sub_shapes2: DataMap::default(),
            my_is_done: false,
            my_tolerance: 0.0,
        }
    }

    /// `BRepExtrema_OverlapTool(theSet1, theSet2)`;
    /// `BRepExtremaOverlapTool::with_sets(&set1, &set2)`.
    pub fn with_sets(
        set1: &Handle<BRepExtremaTriangleSet>,
        set2: &Handle<BRepExtremaTriangleSet>,
    ) -> Self {
        let mut tool: Self = Self::new();
        tool.load_triangle_sets(set1, set2);
        tool
    }

    /// `LoadTriangleSets(theSet1, theSet2)`: the sets of the next test (the results of
    /// an earlier test are kept); `tool.load_triangle_sets(&set1, &set2)`.
    pub fn load_triangle_sets(
        &mut self,
        set1: &Handle<BRepExtremaTriangleSet>,
        set2: &Handle<BRepExtremaTriangleSet>,
    ) {
        self.my_set1 = set1.clone();
        self.my_set2 = set2.clone();
        self.my_is_done = false;
    }

    /// `Perform(theTolerance)` (the native default is 0): the overlapping faces, added to
    /// those of earlier tests; done when a pair of leaves was accepted. Standard_NullObject
    /// before the sets are loaded; `tool.perform(0.0)?`.
    pub fn perform(&mut self, tolerance: f64) -> Result<(), Failure> {
        self.my_tolerance = tolerance;
        let (set1, set2) = (self.set(true)?, self.set(false)?);
        let tree1: BvhTree<BvhVec3d> = set1.take_bvh()?;
        let same: bool = set1.object_address() == set2.object_address();
        let tree2: Option<BvhTree<BvhVec3d>> = match same {
            true => None,
            false => match set2.take_bvh() {
                Ok(tree) => Some(tree),
                Err(failure) => {
                    set1.restore_bvh(tree1);
                    return Err(failure);
                }
            },
        };
        let selected = self.select_trees(Some(&tree1), Some(tree2.as_ref().unwrap_or(&tree1)));
        if let Some(tree2) = tree2 {
            set2.restore_bvh(tree2);
        }
        set1.restore_bvh(tree1);
        self.my_is_done = selected? > 0;
        Ok(())
    }

    /// The first or the second set; Standard_NullObject for a null handle.
    fn set(&self, first: bool) -> Result<Handle<BRepExtremaTriangleSet>, Failure> {
        let set = if first { &self.my_set1 } else { &self.my_set2 };
        if set.is_null() {
            return Err(null_object(
                "BRepExtrema_OverlapTool: a triangle set is null; expected LoadTriangleSets",
            ));
        }
        Ok(set.clone())
    }

    /// `IsDone()`: true when the last test found overlapping elements; `tool.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `MarkDirty()`: the results are outdated; `tool.mark_dirty()`.
    pub fn mark_dirty(&mut self) {
        self.my_is_done = false;
    }

    /// `OverlapSubShapes1()`: the overlapped faces of the second set for each face of the
    /// first one; `tool.overlap_sub_shapes1().extent()`.
    pub fn overlap_sub_shapes1(&self) -> &BRepExtremaOverlapMap {
        &self.my_overlap_sub_shapes1
    }

    /// `OverlapSubShapes2()`: the overlapped faces of the first set for each face of the
    /// second one; `tool.overlap_sub_shapes2().extent()`.
    pub fn overlap_sub_shapes2(&self) -> &BRepExtremaOverlapMap {
        &self.my_overlap_sub_shapes2
    }

    /// `SetElementFilter(theFilter)`: the filter of element pairs (`None`: every pair is
    /// tested); `tool.set_element_filter(None)`.
    pub fn set_element_filter(&mut self, filter: Option<Box<dyn BRepExtremaElementFilter>>) {
        self.my_filter = filter;
    }

    /// Records the overlap of the faces of two triangles in both maps.
    fn add_overlap(&mut self, face_idx1: i32, face_idx2: i32) {
        set_of_faces(&mut self.my_overlap_sub_shapes1, face_idx1).add(face_idx2);
        set_of_faces(&mut self.my_overlap_sub_shapes2, face_idx2).add(face_idx1);
    }

    /// The common start of both narrow phases: the face indices, or `None` when the
    /// faces are already known to overlap; then the filter result.
    fn pre_check(
        &mut self,
        trg_idx1: i32,
        trg_idx2: i32,
    ) -> Result<Option<(i32, i32, BRepExtremaFilterResult)>, Failure> {
        let face_idx1: i32 = self.my_set1.get_face_id(trg_idx1)?;
        let is_in_set: bool = self.my_overlap_sub_shapes1.is_bound(&face_idx1);
        let face_idx2: i32 = self.my_set2.get_face_id(trg_idx2)?;
        if is_in_set
            && self
                .my_overlap_sub_shapes1
                .find(&face_idx1)?
                .contains(face_idx2)
        {
            return Ok(None);
        }
        let result = match self.my_filter.as_mut() {
            None => BRepExtremaFilterResult::DoCheck,
            Some(filter) => filter.pre_check_elements(trg_idx1, trg_idx2)?,
        };
        Ok(Some((face_idx1, face_idx2, result)))
    }

    /// `intersectTrianglesExact(theTrgIdx1, theTrgIdx2)`: the narrow phase without
    /// tolerance.
    fn intersect_triangles_exact(&mut self, trg_idx1: i32, trg_idx2: i32) -> Result<(), Failure> {
        let vertices1 = self.my_set1.get_vertices(trg_idx1)?;
        let Some((face_idx1, face_idx2, result)) = self.pre_check(trg_idx1, trg_idx2)? else {
            return Ok(());
        };
        if result == BRepExtremaFilterResult::Overlap {
            self.add_overlap(face_idx1, face_idx2);
        } else if result == BRepExtremaFilterResult::DoCheck {
            let vertices2 = self.my_set2.get_vertices(trg_idx2)?;
            let [a, b, c] = &vertices1;
            let [d, e, f] = &vertices2;
            if triangles_intersected([a, b, c], [d, e, f]) {
                self.add_overlap(face_idx1, face_idx2);
            }
        }
        Ok(())
    }

    /// `intersectTrianglesToler(theTrgIdx1, theTrgIdx2, theToler)`: the narrow phase with
    /// the bounding prisms of both triangles.
    fn intersect_triangles_toler(
        &mut self,
        trg_idx1: i32,
        trg_idx2: i32,
        toler: f64,
    ) -> Result<(), Failure> {
        let vertices1 = self.my_set1.get_vertices(trg_idx1)?;
        let mut prism1 = BoundingPrism::default(); // not initialized
        let Some((face_idx1, face_idx2, result)) = self.pre_check(trg_idx1, trg_idx2)? else {
            return Ok(());
        };
        if result == BRepExtremaFilterResult::Overlap {
            self.add_overlap(face_idx1, face_idx2);
        } else if result == BRepExtremaFilterResult::DoCheck {
            if !prism1.is_inited {
                prism1.init(vertices1, toler);
            }
            let prism2 = BoundingPrism::new(self.my_set2.get_vertices(trg_idx2)?, toler);
            if prisms_intersected(&prism1, &prism2) {
                self.add_overlap(face_idx1, face_idx2);
            }
        }
        Ok(())
    }
}

impl BvhBaseTraverse<f64> for BRepExtremaOverlapTool {}

impl BvhPairTraverse<BvhVec3d, f64> for BRepExtremaOverlapTool {
    /// `RejectNode`: the boxes do not overlap within the tolerance.
    fn reject_node(
        &self,
        min1: &BvhVec3d,
        max1: &BvhVec3d,
        min2: &BvhVec3d,
        max2: &BvhVec3d,
        _metric: &mut f64,
    ) -> bool {
        !overlap_boxes(min1, max1, min2, max2, self.my_tolerance)
    }

    /// `Accept`: the exact test for a zero tolerance, the prism test otherwise; always
    /// accepted.
    fn accept(&mut self, trg_idx1: i32, trg_idx2: i32) -> Result<bool, Failure> {
        if self.my_tolerance == 0.0 {
            self.intersect_triangles_exact(trg_idx1, trg_idx2)?;
        } else {
            self.intersect_triangles_toler(trg_idx1, trg_idx2, self.my_tolerance)?;
        }
        Ok(true)
    }
}
