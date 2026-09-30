// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_PolyData.hxx/.lxx/.cxx (the class and HideByPolyData) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_PolyData: the triangulation of one face and its hiding triangles.
use super::bi_point::{BiPointIndices, BiPointPoints};
use super::edge_status::EdgeStatus;
use super::failures::{array_failure, null_handle};
use super::poly_data_hide::hide_by_one_triangle;
use super::poly_records::{PolyHidingData, PolyMask, TriangleData};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Xy, Xyz};
use std::sync::RwLockWriteGuard;

/// HLRAlgo_PolyData::FaceIndices: the face index and its packed box.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct FaceIndices {
    pub index: i32,
    pub min: i32,
    pub max: i32,
}

/// HLRAlgo_PolyData::Triangle: the working triangle of the hiding test: its projected
/// vertices, the crossing parameter, the parameter and angular tolerances and the
/// distance tolerance. Natively the fields are uninitialized; the Rust default is zero.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct PolyTriangle {
    pub v1: Xy,
    pub v2: Xy,
    pub v3: Xy,
    pub param: f64,
    pub tol_param: f64,
    pub tol_ang: f64,
    pub tolerance: f64,
}

#[derive(Default)]
struct PolyDataArrays {
    h_nodes: Handle<HArray1<Xyz>>,
    h_t_data: Handle<HArray1<TriangleData>>,
    h_ph_dat: Handle<HArray1<PolyHidingData>>,
}

/// HLRAlgo_PolyData: a face of the polyhedral algorithm: its projected nodes, its
/// triangles and, when it hides, the planes of its hiding triangles. A transient
/// shared by handles; the arrays are handle-shared too (docs/hlr-design.md section 5).
/// A member that reads a null array crashes natively; the port returns
/// Standard_NullObject.
///
/// ```
/// use rustycad_tkhlr::hlr_algo::PolyData;
/// let face = PolyData::new();
/// face.set_face_index(3);
/// assert_eq!(face.face_index(), 3);
/// assert!(!face.hiding());
/// assert_eq!(face.nodes().err().unwrap().exception_type(), "Standard_NullObject");
/// ```
#[derive(Default)]
pub struct PolyData {
    base: StandardTransient,
    face_indices: SharedState<FaceIndices>,
    arrays: SharedState<PolyDataArrays>,
}
impl_transient!(PolyData, "HLRAlgo_PolyData", StandardTransient, base);

/// The array behind `handle`, or the null failure of `member`.
fn shared_array<T: Send + Sync + 'static>(
    handle: &Handle<HArray1<T>>,
    member: &str,
    field: &str,
) -> Result<Handle<HArray1<T>>, Failure> {
    if handle.is_null() {
        return Err(null_handle(member, field));
    }
    Ok(handle.clone())
}

impl PolyData {
    /// HLRAlgo_PolyData(): null arrays and a zero face index; `PolyData::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// HNodes: sets the projected nodes; `face.set_h_nodes(nodes)`.
    pub fn set_h_nodes(&self, h_nodes: Handle<HArray1<Xyz>>) {
        self.arrays.write().h_nodes = h_nodes;
    }

    /// HTData: sets the triangles; `face.set_h_t_data(triangles)`.
    pub fn set_h_t_data(&self, h_t_data: Handle<HArray1<TriangleData>>) {
        self.arrays.write().h_t_data = h_t_data;
    }

    /// HPHDat: sets the hiding triangles (a null handle makes the face non hiding);
    /// `face.set_h_ph_dat(hiding)`.
    pub fn set_h_ph_dat(&self, h_ph_dat: Handle<HArray1<PolyHidingData>>) {
        self.arrays.write().h_ph_dat = h_ph_dat;
    }

    /// FaceIndex(I); `face.set_face_index(1)`.
    pub fn set_face_index(&self, index: i32) {
        self.face_indices.write().index = index;
    }

    /// FaceIndex(); `face.face_index()`.
    pub fn face_index(&self) -> i32 {
        self.face_indices.read().index
    }

    /// Nodes(): the projected nodes (the shared array; Standard_NullObject when unset);
    /// `face.nodes()?.change_array1()`.
    pub fn nodes(&self) -> Result<Handle<HArray1<Xyz>>, Failure> {
        shared_array(
            &self.arrays.read().h_nodes,
            "HLRAlgo_PolyData::Nodes",
            "myHNodes",
        )
    }

    /// TData(): the triangles; `face.t_data()?`.
    pub fn t_data(&self) -> Result<Handle<HArray1<TriangleData>>, Failure> {
        shared_array(
            &self.arrays.read().h_t_data,
            "HLRAlgo_PolyData::TData",
            "myHTData",
        )
    }

    /// PHDat(): the hiding triangles; `face.ph_dat()?`.
    pub fn ph_dat(&self) -> Result<Handle<HArray1<PolyHidingData>>, Failure> {
        shared_array(
            &self.arrays.read().h_ph_dat,
            "HLRAlgo_PolyData::PHDat",
            "myHPHDat",
        )
    }

    /// Hiding: the face has hiding triangles (a set hiding array); `face.hiding()`.
    pub fn hiding(&self) -> bool {
        self.arrays.read().h_ph_dat.is_some()
    }

    /// Indices(): the face index and its packed box, writable through the guard;
    /// `face.indices().min = 0`.
    pub fn indices(&self) -> RwLockWriteGuard<'_, FaceIndices> {
        self.face_indices.write()
    }

    /// UpdateGlobalMinMax: adds the nodes of every hiding triangle (flag FMskHiding)
    /// of triangles 1..upper to `the_box`; `face.update_global_min_max(&mut bounds)?`.
    pub fn update_global_min_max(&self, the_box: &mut BoundingBox3) -> Result<(), Failure> {
        let (nodes, triangles) = (self.nodes()?, self.t_data()?);
        let (nodes, triangles) = (nodes_of(&nodes)?, triangles_of(&triangles)?);
        let (nodes, triangles) = (nodes.array1(), triangles.array1());
        for i in 1..=triangles.upper() {
            let td = *triangles.value(i).map_err(array_failure(GLOBAL_MIN_MAX))?;
            if td.flags & PolyMask::FMskHiding as i32 == 0 {
                continue;
            }
            for node in [td.node1, td.node2, td.node3] {
                let p = nodes.value(node).map_err(array_failure(GLOBAL_MIN_MAX))?;
                the_box.update_point(p.x(), p.y(), p.z());
            }
        }
        Ok(())
    }

    /// HideByPolyData: hides the parts of the segment `points` (with the packed box
    /// and the faces of `indices`) behind the hiding triangles of this face whose
    /// boxes meet it, into `status`. With `hiding_shell` (the segment belongs to the
    /// shell of this face) a triangle sharing the segment's edge is skipped. The
    /// working `triangle` keeps the vertices and the crossing parameter of the last
    /// triangle tested; `face.hide_by_poly_data(&p, &mut t, &i, false, &mut status)?`.
    pub fn hide_by_poly_data(
        &self,
        points: &BiPointPoints,
        triangle: &mut PolyTriangle,
        indices: &BiPointIndices,
        hiding_shell: bool,
        status: &mut EdgeStatus,
    ) -> Result<(), Failure> {
        let face = *self.face_indices.read();
        if !boxes_meet(face.max, face.min, indices) {
            return Ok(());
        }
        let hiding = self.ph_dat()?;
        let hiding = hiding.get().ok_or_else(|| null_handle(HIDE, "myHPHDat"))?;
        let hiding = hiding.array1();
        // The native loop starts from a pointer to entry 1, read even when the array
        // is empty (Standard_OutOfRange then).
        hiding.value(1).map_err(array_failure(HIDE))?;
        let context = HideContext {
            face,
            points,
            indices,
            hiding_shell,
        };
        for h in 1..=hiding.upper() {
            let data = hiding.value(h).map_err(array_failure(HIDE))?;
            self.hide_by_hiding_triangle(&context, data, triangle, status)?;
        }
        Ok(())
    }

    /// One hiding triangle of HideByPolyData: the box test, the connexity test, then
    /// the plane test that selects the crossing case of the segment.
    fn hide_by_hiding_triangle(
        &self,
        context: &HideContext<'_>,
        data: &PolyHidingData,
        triangle: &mut PolyTriangle,
        status: &mut EdgeStatus,
    ) -> Result<(), Failure> {
        let triangle_indices = data.indices();
        if !boxes_meet(triangle_indices.max, triangle_indices.min, context.indices) {
            return Ok(());
        }
        let td = self.triangle_data(triangle_indices.index)?;
        if !context.is_not_connex(&td) {
            return Ok(());
        }
        let plane = data.plane();
        let d1 = plane.normal.dot(context.points.pnt_p1) - plane.d;
        let d2 = plane.normal.dot(context.points.pnt_p2) - plane.d;
        let Some((crossing, hide_before)) = crossing_case(d1, d2, triangle.tolerance) else {
            return Ok(());
        };
        if crossing {
            triangle.param = d1 / (d1 - d2);
        }
        self.load_vertices(&td, triangle)?;
        hide_by_one_triangle(
            context.points,
            triangle,
            crossing,
            hide_before,
            td.flags,
            status,
        );
        Ok(())
    }

    fn triangle_data(&self, index: i32) -> Result<TriangleData, Failure> {
        let triangles = self.t_data()?;
        let triangles = triangles_of(&triangles)?;
        let triangles = triangles.array1();
        triangles.value(index).copied().map_err(array_failure(HIDE))
    }

    /// The projected vertices of `td` into the working triangle.
    fn load_vertices(&self, td: &TriangleData, triangle: &mut PolyTriangle) -> Result<(), Failure> {
        let nodes = self.nodes()?;
        let nodes = nodes_of(&nodes)?;
        let nodes = nodes.array1();
        let vertex = |node: i32| -> Result<Xy, Failure> {
            let p = nodes.value(node).map_err(array_failure(HIDE))?;
            Ok(Xy::new(p.x(), p.y()))
        };
        triangle.v1 = vertex(td.node1)?;
        triangle.v2 = vertex(td.node2)?;
        triangle.v3 = vertex(td.node3)?;
        Ok(())
    }
}

const HIDE: &str = "HLRAlgo_PolyData::HideByPolyData";
const GLOBAL_MIN_MAX: &str = "HLRAlgo_PolyData::UpdateGlobalMinMax";

fn nodes_of(handle: &Handle<HArray1<Xyz>>) -> Result<&HArray1<Xyz>, Failure> {
    handle
        .get()
        .ok_or_else(|| null_handle("HLRAlgo_PolyData::Nodes", "myHNodes"))
}

fn triangles_of(handle: &Handle<HArray1<TriangleData>>) -> Result<&HArray1<TriangleData>, Failure> {
    handle
        .get()
        .ok_or_else(|| null_handle("HLRAlgo_PolyData::TData", "myHTData"))
}

/// The packed box test of the native code: the box `[min, max]` (of a face or of a
/// triangle) meets the segment box of `indices` when no borrow reaches the guard
/// bits of the two differences (the `int` subtractions wrap on the pinned profile).
pub(super) fn boxes_meet(max: i32, min: i32, indices: &BiPointIndices) -> bool {
    (max.wrapping_sub(indices.min_seg) & (0x8010_0200_u32 as i32)) == 0
        && (indices.max_seg.wrapping_sub(min) & (0x8010_0000_u32 as i32)) == 0
}

/// The plane test of HideByPolyData on the signed distances of the segment ends:
/// `Some((crossing, hide_before))` when the triangle is tested (the segment crosses
/// the plane, or lies behind it at one end), `None` when it is skipped.
fn crossing_case(d1: f64, d2: f64, tolerance: f64) -> Option<(bool, bool)> {
    if d1 > tolerance {
        return (d2 < -tolerance).then_some((true, false));
    }
    if d1 < -tolerance {
        return Some(if d2 > tolerance {
            (true, true)
        } else {
            (false, false)
        });
    }
    (d2 < -tolerance).then_some((false, false))
}

/// The inputs of HideByPolyData shared by its hiding triangles.
struct HideContext<'a> {
    face: FaceIndices,
    points: &'a BiPointPoints,
    indices: &'a BiPointIndices,
    hiding_shell: bool,
}

impl HideContext<'_> {
    /// The connexity test: in the segment's own shell, a triangle of one of the two
    /// faces around the segment that holds the segment's edge does not hide it.
    fn is_not_connex(&self, td: &TriangleData) -> bool {
        if !self.hiding_shell {
            return true;
        }
        let i = self.indices;
        if self.face.index == i.face_conex1 {
            return not_on_edge(td, i.face1_pt1, i.face1_pt2);
        }
        if self.face.index == i.face_conex2 {
            return not_on_edge(td, i.face2_pt1, i.face2_pt2);
        }
        true
    }
}

/// The native test of one face: when `pt1` is a node of the triangle, the edge
/// `(pt1, pt2)` is not an edge of it; otherwise the triangle is not connex.
fn not_on_edge(td: &TriangleData, pt1: i32, pt2: i32) -> bool {
    if pt1 == td.node1 {
        pt2 != td.node2 && pt2 != td.node3
    } else if pt1 == td.node2 {
        pt2 != td.node3 && pt2 != td.node1
    } else if pt1 == td.node3 {
        pt2 != td.node1 && pt2 != td.node2
    } else {
        true
    }
}
