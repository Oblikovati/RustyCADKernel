// Rust translation of BRepMesh_DelabellaBaseMeshAlgo.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::algo_object::{
    BaseMeshAlgo, ConstrainedBaseMeshAlgo, CustomBaseMeshAlgo, FaceMeshingCore, LockedFaceCore,
};
use super::base::{
    BaseMeshAlgoData, BaseMeshAlgoHooks, ConstrainedBaseMeshAlgoHooks, base_perform,
};
use super::generator::{CustomBaseTriangulation, CustomGenerator, MeshGenerator};
use crate::brep_mesh::delabella::DelaBella;
use crate::brep_mesh::{
    BRepMeshDataStructureOfDelaun, BRepMeshEdge, BRepMeshTriangle, BRepMeshVertex, DegreeOfFreedom,
};
use crate::imesh_data::FaceHandle;
use crate::imesh_tools::{MeshAlgo, MeshParameters};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{Gravity, Message, ProgressRange};
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::bnd::CompactBox2;
use rustycad_tkmath::gp::Xy;

/// `BRepMesh_DelabellaBaseMeshAlgo::buildBaseTriangulation`: the Delaunay triangulation
/// of the registered nodes (normalized into the unit square around the origin, with
/// the 4 corners of their enlarged bounding box added as auxiliary nodes) by DelaBella.
pub struct DelabellaTriangulation;

impl CustomBaseTriangulation for DelabellaTriangulation {
    fn build_base_triangulation<A: ConstrainedBaseMeshAlgoHooks + ?Sized>(
        algo: &mut A,
    ) -> Result<(), Failure> {
        let structure = algo.base_data().get_structure().clone();
        let structure = structure.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepMesh_DelabellaBaseMeshAlgo::buildBaseTriangulation: structure=null; expected a data structure",
            )
        })?;
        let points = delabella_points(structure)?;
        triangulate_into(structure, &points)
    }
}

/// The node coordinates followed by the 4 box corners (added to the structure as free
/// nodes), mapped to `[-0.5, 0.5]²`.
fn delabella_points(structure: &BRepMeshDataStructureOfDelaun) -> Result<Vec<f64>, Failure> {
    let nodes_nb = structure.nb_nodes() as usize;
    let mut points = vec![0.0; 2 * (nodes_nb + 4)];
    let mut bounds = CompactBox2::<f64>::default();
    for node in 0..nodes_nb {
        let coord = *structure.get_node(node as i32 + 1)?.coord();
        points[2 * node] = coord.x();
        points[2 * node + 1] = coord.y();
        bounds.add_xy(coord);
    }
    let extent = bounds.corner_max() - bounds.corner_min();
    bounds.enlarge(0.1 * extent.modulus());
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    let corners = [
        (min.x(), min.y()),
        (max.x(), min.y()),
        (max.x(), max.y()),
        (min.x(), max.y()),
    ];
    for (k, (x, y)) in corners.into_iter().enumerate() {
        points[2 * (nodes_nb + k)] = x;
        points[2 * (nodes_nb + k) + 1] = y;
        structure.add_node(&BRepMeshVertex::from_uv(x, y, DegreeOfFreedom::Free), false)?;
    }
    normalize(&mut points, min, max);
    Ok(points)
}

/// `(p - min) / (max - min) - 0.5` on each coordinate.
fn normalize(points: &mut [f64], min: Xy, max: Xy) {
    let (diff_x, diff_y) = (max.x() - min.x(), max.y() - min.y());
    for [x, y] in points.as_chunks_mut::<2>().0 {
        *x = (*x - min.x()) / diff_x - 0.5;
        *y = (*y - min.y()) / diff_y - 0.5;
    }
}

/// Triangulates the points and adds each Delaunay triangle, reoriented
/// (`v[0], v[2], v[1]`), with its links to the structure. A colinear cloud adds none.
fn triangulate_into(
    structure: &BRepMeshDataStructureOfDelaun,
    points: &[f64],
) -> Result<(), Failure> {
    let mut triangulator = DelaBella::create();
    triangulator.set_err_log(Some(Box::new(log_delabella_to_occ)));
    let count = (points.len() / 2) as i32;
    if triangulator.triangulate_f64(count, points, Some(&points[1..]), 2)? <= 0 {
        return Ok(());
    }
    let mut face = triangulator.get_first_delaunay_triangle();
    while let Some(index) = face {
        let (nodes, next) = triangle_nodes(&triangulator, index)?;
        add_triangle(structure, nodes)?;
        face = next;
    }
    Ok(())
}

/// The one-based structure nodes of a Delaunay triangle in the order `v[0], v[2],
/// v[1]`, and the next triangle of the list.
fn triangle_nodes(
    triangulator: &DelaBella,
    index: usize,
) -> Result<([i32; 3], Option<usize>), Failure> {
    let missing = |what: &str, at: usize| {
        Failure::with_message(
            "Standard_OutOfRange",
            &format!("BRepMesh_DelabellaBaseMeshAlgo: {what} {at} outside the DelaBella output"),
        )
    };
    let triangle = triangulator
        .triangle(index)
        .ok_or_else(|| missing("triangle", index))?;
    let mut nodes = [0; 3];
    for (node, slot) in nodes.iter_mut().zip([0, 2, 1]) {
        let vertex = triangle.v[slot];
        *node = triangulator
            .vertex(vertex)
            .ok_or_else(|| missing("vertex", vertex))?
            .i
            + 1;
    }
    Ok((nodes, triangle.next))
}

/// The links `nodes[k] -> nodes[k + 1]` and the triangle over them.
fn add_triangle(structure: &BRepMeshDataStructureOfDelaun, nodes: [i32; 3]) -> Result<(), Failure> {
    let mut edges = [0; 3];
    let mut orientations = [false; 3];
    for k in 0..3 {
        let link = BRepMeshEdge::new(nodes[k], nodes[(k + 1) % 3], DegreeOfFreedom::Free);
        let link_info = structure.add_link(&link)?;
        edges[k] = link_info.abs();
        orientations[k] = link_info > 0;
    }
    let triangle = BRepMeshTriangle::new(edges, orientations, DegreeOfFreedom::Free);
    structure.add_element(&triangle)?;
    Ok(())
}

/// `logDelabella2Occ`: a DelaBella message to the OCCT messenger, `[ERR]` as a fail,
/// `[WRN]`/`[NFO]` as a trace, anything else as a warning.
fn log_delabella_to_occ(text: &str) {
    let gravity = match text.as_bytes().get(1) {
        Some(b'E') => Gravity::Fail,
        Some(b'W' | b'N') => Gravity::Trace,
        _ => Gravity::Warning,
    };
    Message::send_text(&AsciiString::new(text), gravity);
}

/// The plain `BRepMesh_DelabellaBaseMeshAlgo` (planes without internal vertices);
/// `DelabellaBaseMeshAlgoCore::default()`.
#[derive(Default)]
pub struct DelabellaBaseMeshAlgoCore {
    data: BaseMeshAlgoData,
}

impl BaseMeshAlgoHooks for DelabellaBaseMeshAlgoCore {
    fn base_data(&self) -> &BaseMeshAlgoData {
        &self.data
    }
    fn base_data_mut(&mut self) -> &mut BaseMeshAlgoData {
        &mut self.data
    }
    fn generate_mesh(&mut self, range: &ProgressRange<'_>) -> Result<(), Failure> {
        CustomGenerator::<DelabellaTriangulation>::generate_mesh(self, range)
    }
}

impl ConstrainedBaseMeshAlgoHooks for DelabellaBaseMeshAlgoCore {}

impl FaceMeshingCore for DelabellaBaseMeshAlgoCore {
    fn perform_face(
        &mut self,
        face: &FaceHandle,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        base_perform(self, face, parameters, range);
        Ok(())
    }
}

/// `BRepMesh_DelabellaBaseMeshAlgo` as a handle: the plain algorithm, or one of the
/// node-insertion template instances over `BRepMesh_CustomDelaunayBaseMeshAlgo` of it
/// (whose dynamic type is this class, as the C++ templates declare no RTTI).
///
/// ```
/// use rustycad_tkmesh::brep_mesh::mesh_algo::BRepMeshDelabellaBaseMeshAlgo;
/// use rustycad_tkernel::standard::RuntimeTyped;
/// assert_eq!(BRepMeshDelabellaBaseMeshAlgo::TYPE_NAME, "BRepMesh_DelabellaBaseMeshAlgo");
/// let _algo = BRepMeshDelabellaBaseMeshAlgo::new();
/// ```
pub struct BRepMeshDelabellaBaseMeshAlgo {
    transient: StandardTransient,
    core: LockedFaceCore,
}
impl_transient!(
    BRepMeshDelabellaBaseMeshAlgo,
    "BRepMesh_DelabellaBaseMeshAlgo",
    dyn CustomBaseMeshAlgo,
    transient,
    interfaces: [dyn MeshAlgo, dyn BaseMeshAlgo, dyn ConstrainedBaseMeshAlgo, dyn CustomBaseMeshAlgo]
);

impl Default for BRepMeshDelabellaBaseMeshAlgo {
    fn default() -> Self {
        Self::from_core(DelabellaBaseMeshAlgoCore::default())
    }
}

impl BRepMeshDelabellaBaseMeshAlgo {
    /// `BRepMesh_DelabellaBaseMeshAlgo()`; `BRepMeshDelabellaBaseMeshAlgo::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// A template instance over this class (`new NodeInsertionMeshAlgo<...>::Type`);
    /// `BRepMeshDelabellaBaseMeshAlgo::from_core(algo)`.
    pub fn from_core(core: impl FaceMeshingCore + 'static) -> Self {
        Self {
            transient: StandardTransient::default(),
            core: LockedFaceCore::new(core),
        }
    }
}

impl MeshAlgo for BRepMeshDelabellaBaseMeshAlgo {
    fn perform(
        &self,
        face: &FaceHandle,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.core.perform(face, parameters, range)
    }
}
impl BaseMeshAlgo for BRepMeshDelabellaBaseMeshAlgo {}
impl ConstrainedBaseMeshAlgo for BRepMeshDelabellaBaseMeshAlgo {}
impl CustomBaseMeshAlgo for BRepMeshDelabellaBaseMeshAlgo {}
