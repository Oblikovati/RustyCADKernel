// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_SelfIntersection.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_SelfIntersection`: the overlapping faces of one shape, found on its
//! triangulation by the overlap tool with a filter of the correct shared edges and
//! vertices of adjacent triangles.
use super::overlap_tool::{BRepExtremaOverlapMap, projection_axes};
use super::{
    BRepExtremaElementFilter, BRepExtremaFilterResult, BRepExtremaOverlapTool,
    BRepExtremaTriangleSet,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bvh::BvhVec3d;

/// `ZERO_VEC`.
const ZERO_VEC: BvhVec3d = BvhVec3d {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

/// `ccw(v0, v1, v2, theX, theY)`: the triangle is counterclockwise in the plane of the
/// two axes (the shoelace sum is negative).
fn ccw(v0: &BvhVec3d, v1: &BvhVec3d, v2: &BvhVec3d, x: usize, y: usize) -> bool {
    let sum: f64 = (v1[x] - v0[x]) * (v1[y] + v0[y])
        + (v2[x] - v1[x]) * (v2[y] + v1[y])
        + (v0[x] - v2[x]) * (v0[y] + v2[y]);
    sum < 0.0
}

/// `rayInsideAngle(theDirec, theEdge0, theEdge1, theX, theY)`: the ray lies inside the
/// angle of the two edges (both sub-angles turn as the angle).
fn ray_inside_angle(
    direc: &BvhVec3d,
    edge0: &BvhVec3d,
    edge1: &BvhVec3d,
    (x, y): (usize, usize),
) -> bool {
    let is_ccw: bool = ccw(&ZERO_VEC, edge0, edge1, x, y);
    ccw(&ZERO_VEC, edge0, direc, x, y) == is_ccw && ccw(&ZERO_VEC, direc, edge1, x, y) == is_ccw
}

/// `Standard_ASSERT_RAISE`: Standard_ProgramError when `condition` is false.
fn assert_raise(condition: bool, description: &str) -> Result<(), Failure> {
    if condition {
        return Ok(());
    }
    Err(program_error(&format!(
        "*** ERROR: ASSERT in file 'BRepExtrema_SelfIntersection.cxx': \n{description} (aPosOutTrgn || aNegOutTrgn)"
    )))
}

/// Whether the shared line and its opposite are outside the angle of a triangle at the
/// shared vertex; one of them must be (the native assertion).
fn line_outside(
    cross_line: &BvhVec3d,
    edges: &[BvhVec3d; 2],
    normal: &BvhVec3d,
) -> Result<(bool, bool), Failure> {
    let axes = projection_axes(normal);
    let pos_out: bool = !ray_inside_angle(cross_line, &edges[0], &edges[1], axes);
    let neg_out: bool = !ray_inside_angle(&-*cross_line, &edges[0], &edges[1], axes);
    assert_raise(
        pos_out || neg_out,
        "Failed to detect if shared vertex is regular or not",
    )?;
    Ok((pos_out, neg_out))
}

/// `isRegularSharedVertex(theSharedVert, theTrng1Vtxs1, theTrng1Vtxs2, theTrng2Vtxs1,
/// theTrng2Vtxs2)`: `NoCheck` when two triangles sharing one vertex do not overlap
/// near it, `Overlap` otherwise.
fn is_regular_shared_vertex(
    shared_vert: &BvhVec3d,
    [trng0_vtxs1, trng0_vtxs2]: [&BvhVec3d; 2],
    [trng1_vtxs1, trng1_vtxs2]: [&BvhVec3d; 2],
) -> Result<BRepExtremaFilterResult, Failure> {
    let trng0_edges = [
        (*trng0_vtxs1 - *shared_vert).normalized(),
        (*trng0_vtxs2 - *shared_vert).normalized(),
    ];
    let trng1_edges = [
        (*trng1_vtxs1 - *shared_vert).normalized(),
        (*trng1_vtxs2 - *shared_vert).normalized(),
    ];
    let trng0_normal = BvhVec3d::cross(&trng0_edges[0], &trng0_edges[1]);
    let trng1_normal = BvhVec3d::cross(&trng1_edges[0], &trng1_edges[1]);
    let cross_line = BvhVec3d::cross(&trng0_normal, &trng1_normal);
    if cross_line.square_modulus() < precision::square_confusion() {
        // Coplanar case.
        let axes = projection_axes(&trng0_normal);
        let overlap: bool =
            ray_inside_angle(&trng1_edges[0], &trng0_edges[0], &trng0_edges[1], axes)
                || ray_inside_angle(&trng1_edges[1], &trng0_edges[0], &trng0_edges[1], axes)
                || ray_inside_angle(&trng0_edges[0], &trng1_edges[0], &trng1_edges[1], axes)
                || ray_inside_angle(&trng0_edges[1], &trng1_edges[0], &trng1_edges[1], axes);
        return Ok(if overlap {
            BRepExtremaFilterResult::Overlap
        } else {
            BRepExtremaFilterResult::NoCheck
        });
    }
    // The shared line should lie outside at least one triangle.
    let (pos_out0, neg_out0) = line_outside(&cross_line, &trng0_edges, &trng0_normal)?;
    if pos_out0 && neg_out0 {
        return Ok(BRepExtremaFilterResult::NoCheck);
    }
    let (pos_out1, neg_out1) = line_outside(&cross_line, &trng1_edges, &trng1_normal)?;
    if pos_out1 && neg_out1 {
        return Ok(BRepExtremaFilterResult::NoCheck);
    }
    Ok(match (pos_out0 || pos_out1) && (neg_out0 || neg_out1) {
        true => BRepExtremaFilterResult::NoCheck,
        false => BRepExtremaFilterResult::Overlap,
    })
}

/// `isRegularSharedEdge(theTrng1Vtxs0, theTrng1Vtxs1, theTrng1Vtxs2, theTrng2Vtxs2)`:
/// `NoCheck` for non-coplanar triangles or coplanar ones on both sides of the shared
/// edge, `Overlap` for coplanar ones on the same side.
fn is_regular_shared_edge(
    trng0_vtxs0: &BvhVec3d,
    trng0_vtxs1: &BvhVec3d,
    trng0_vtxs2: &BvhVec3d,
    trng1_vtxs2: &BvhVec3d,
) -> BRepExtremaFilterResult {
    let shared_edge = (*trng0_vtxs1 - *trng0_vtxs0).normalized();
    let unique_edges = [
        (*trng0_vtxs2 - *trng0_vtxs0).normalized(),
        (*trng1_vtxs2 - *trng0_vtxs0).normalized(),
    ];
    let trng0_normal = BvhVec3d::cross(&shared_edge, &unique_edges[0]);
    let trng1_normal = BvhVec3d::cross(&shared_edge, &unique_edges[1]);
    let cross_line = BvhVec3d::cross(&trng0_normal, &trng1_normal);
    if cross_line.square_modulus() > precision::square_confusion() {
        return BRepExtremaFilterResult::NoCheck; // non-coplanar case
    }
    let (x, y) = projection_axes(&trng0_normal);
    if ccw(&ZERO_VEC, &shared_edge, &unique_edges[0], x, y)
        != ccw(&ZERO_VEC, &shared_edge, &unique_edges[1], x, y)
    {
        return BRepExtremaFilterResult::NoCheck;
    }
    BRepExtremaFilterResult::Overlap
}

/// `PreCheckElements(theIndex1, theIndex2)` on `set`: `NoCheck` for triangles of one
/// face; the shared edge or shared vertex tests for triangles with two or one common
/// vertices (within Confusion); `DoCheck` otherwise.
fn pre_check_elements(
    set: &BRepExtremaTriangleSet,
    index1: i32,
    index2: i32,
) -> Result<BRepExtremaFilterResult, Failure> {
    if set.get_face_id(index1)? == set.get_face_id(index2)? {
        return Ok(BRepExtremaFilterResult::NoCheck); // triangles are from the same face
    }
    let mut trng0_vtxs: [BvhVec3d; 3] = set.get_vertices(index1)?;
    let mut trng1_vtxs: [BvhVec3d; 3] = set.get_vertices(index2)?;
    let mut shared_vtxs: Vec<(usize, usize)> = Vec::new();
    for (vert_idx1, vertex1) in trng0_vtxs.iter().enumerate() {
        let shared = trng1_vtxs.iter().position(|vertex2| {
            (*vertex1 - *vertex2).square_modulus() < precision::square_confusion()
        });
        if let Some(vert_idx2) = shared {
            shared_vtxs.push((vert_idx1, vert_idx2)); // then the next vertex of the 1st triangle
        }
    }
    match shared_vtxs.as_slice() {
        [(first0, first1), (second0, second1)] => Ok(is_regular_shared_edge(
            &trng0_vtxs[*first0],
            &trng0_vtxs[*second0],
            &trng0_vtxs[3 - first0 - second0],
            &trng1_vtxs[3 - first1 - second1],
        )),
        [(first0, first1)] => {
            trng0_vtxs.swap(0, *first0);
            trng1_vtxs.swap(0, *first1);
            is_regular_shared_vertex(
                &trng0_vtxs[0],
                [&trng0_vtxs[1], &trng0_vtxs[2]],
                [&trng1_vtxs[1], &trng1_vtxs[2]],
            )
        }
        _ => Ok(BRepExtremaFilterResult::DoCheck),
    }
}

/// The filter the self-intersection gives to its overlap tool: its own
/// `PreCheckElements` on its element set (natively the tool points to the
/// self-intersection object itself).
struct SelfIntersectionFilter {
    element_set: Handle<BRepExtremaTriangleSet>,
}

impl BRepExtremaElementFilter for SelfIntersectionFilter {
    fn pre_check_elements(
        &mut self,
        index1: i32,
        index2: i32,
    ) -> Result<BRepExtremaFilterResult, Failure> {
        pre_check_elements(&self.element_set, index1, index2)
    }
}

/// `BRepExtrema_SelfIntersection` (docs/topalgo-design.md section 13): the pairs of
/// faces of a triangulated shape whose triangles overlap (within a tolerance when it
/// is not 0), except the correct contacts of adjacent triangles.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaSelfIntersection;
/// let mut tool = BRepExtremaSelfIntersection::new(0.0);
/// assert!(tool.load_shape(&Shape::default())?);
/// tool.perform()?;
/// assert!(!tool.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepExtremaSelfIntersection {
    my_tolerance: f64,
    my_is_init: bool,
    my_face_list: Vec<Shape>,
    my_element_set: Handle<BRepExtremaTriangleSet>,
    my_overlap_tool: BRepExtremaOverlapTool,
}

impl BRepExtremaSelfIntersection {
    /// `BRepExtrema_SelfIntersection(theTolerance)` (the native default is 0): no shape;
    /// `BRepExtremaSelfIntersection::new(0.0)`.
    pub fn new(tolerance: f64) -> Self {
        Self {
            my_tolerance: tolerance,
            my_is_init: false,
            my_face_list: Vec::new(),
            my_element_set: Handle::default(),
            my_overlap_tool: BRepExtremaOverlapTool::new(),
        }
    }

    /// `BRepExtrema_SelfIntersection(theShape, theTolerance)`: loads the shape;
    /// `BRepExtremaSelfIntersection::with_shape(&shape, 0.0)?`.
    pub fn with_shape(shape: &Shape, tolerance: f64) -> Result<Self, Failure> {
        let mut tool: Self = Self::new(tolerance);
        tool.load_shape(shape)?;
        Ok(tool)
    }

    /// `Tolerance()`: the tolerance of the test; `tool.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.my_tolerance
    }

    /// `SetTolerance(theTolerance)`: the tolerance of the next test;
    /// `tool.set_tolerance(1e-3)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.my_tolerance = tolerance;
    }

    /// `LoadShape(theShape)`: the faces of the shape, in exploration order, and their
    /// triangles (the element set is created once and reinitialized); the overlap tool
    /// gets the set as both of its sets. Returns the result of `Init` (always true);
    /// `tool.load_shape(&shape)?`.
    pub fn load_shape(&mut self, shape: &Shape) -> Result<bool, Failure> {
        self.my_face_list.clear();
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            self.my_face_list.push(face);
        }
        if self.my_element_set.is_null() {
            self.my_element_set = Handle::new(BRepExtremaTriangleSet::new());
        }
        self.my_is_init = self.my_element_set.init(&self.my_face_list)?;
        if self.my_is_init {
            self.my_overlap_tool
                .load_triangle_sets(&self.my_element_set, &self.my_element_set);
        }
        Ok(self.my_is_init)
    }

    /// `Perform()`: the overlap test with the filter of adjacent triangles, unless the
    /// shape is not loaded or the test is done; `tool.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        if !self.my_is_init || self.my_overlap_tool.is_done() {
            return Ok(());
        }
        let filter = SelfIntersectionFilter {
            element_set: self.my_element_set.clone(),
        };
        self.my_overlap_tool
            .set_element_filter(Some(Box::new(filter)));
        self.my_overlap_tool.perform(self.my_tolerance)
    }

    /// `IsDone()`: the state of the overlap tool; `tool.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_overlap_tool.is_done()
    }

    /// `OverlapElements()`: the overlapped faces (indices from 0) of each face;
    /// `tool.overlap_elements().extent()`.
    pub fn overlap_elements(&self) -> &BRepExtremaOverlapMap {
        self.my_overlap_tool.overlap_sub_shapes1()
    }

    /// `GetSubShape(theID)`: the face of index `id` (from 0); Standard_OutOfRange outside
    /// the list (an unchecked read natively); `tool.get_sub_shape(0)?`.
    pub fn get_sub_shape(&self, id: i32) -> Result<Face, Failure> {
        let face = usize::try_from(id)
            .ok()
            .and_then(|i| self.my_face_list.get(i))
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "BRepExtrema_SelfIntersection::GetSubShape: index {id} outside [0, {})",
                    self.my_face_list.len()
                ))
            })?;
        topods::face(face)
    }

    /// `ElementSet()`: the triangles of the faces (a null handle before any load);
    /// `tool.element_set()`.
    pub fn element_set(&self) -> &Handle<BRepExtremaTriangleSet> {
        &self.my_element_set
    }

    /// `PreCheckElements(theIndex1, theIndex2)`: the filter of the overlap test on the
    /// element set; Standard_NullObject before any load; `tool.pre_check_elements(0, 1)?`.
    pub fn pre_check_elements(
        &self,
        index1: i32,
        index2: i32,
    ) -> Result<BRepExtremaFilterResult, Failure> {
        let set = self.my_element_set.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "BRepExtrema_SelfIntersection: no element set",
            )
        })?;
        pre_check_elements(set, index1, index2)
    }

    /// `isRegularSharedVertex`: `NoCheck` when two triangles sharing one vertex do not overlap near it, `Overlap` otherwise;
    /// `tool.is_regular_shared_vertex(&v, [&a, &b], [&c, &d])?`.
    pub fn is_regular_shared_vertex(
        &self,
        shared_vert: &BvhVec3d,
        trng0: [&BvhVec3d; 2],
        trng1: [&BvhVec3d; 2],
    ) -> Result<BRepExtremaFilterResult, Failure> {
        is_regular_shared_vertex(shared_vert, trng0, trng1)
    }

    /// `isRegularSharedEdge`: `NoCheck` for non-coplanar triangles or coplanar ones on both sides of the shared edge, `Overlap` otherwise;
    /// `tool.is_regular_shared_edge(&a, &b, &c, &d)`.
    pub fn is_regular_shared_edge(
        &self,
        v0: &BvhVec3d,
        v1: &BvhVec3d,
        v2: &BvhVec3d,
        w2: &BvhVec3d,
    ) -> BRepExtremaFilterResult {
        is_regular_shared_edge(v0, v1, v2, w2)
    }
}
