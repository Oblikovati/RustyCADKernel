// Rust translation of BRepMesh_BaseMeshAlgo.hxx/.cxx and BRepMesh_ConstrainedBaseMeshAlgo.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::brep_mesh::{
    BRepMeshDataStructureOfDelaun, BRepMeshDelaun, BRepMeshEdge, BRepMeshVertex, DegreeOfFreedom,
};
use crate::brep_mesh_data::MeshAllocator;
use crate::imesh_data::{EdgeHandle, FaceHandle, MEMORY_BLOCK_SIZE_HUGE, PCurveHandle, Status};
use crate::imesh_tools::MeshParameters;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, IncrementalAllocatorObject};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Point3};

/// The protected and private data of `BRepMesh_BaseMeshAlgo`, live during one Perform.
#[derive(Default)]
pub struct BaseMeshAlgoData {
    face: FaceHandle,
    parameters: MeshParameters,
    allocator: MeshAllocator,
    structure: Handle<BRepMeshDataStructureOfDelaun>,
    /// `VectorOfPnt`: the 3d points, indexed from 0 by `Location3d`.
    nodes_map: Vec<Point3>,
    /// `DMapOfIntegerInteger myUsedNodes`: structure node to triangulation node.
    used_nodes: DataMap<i32, i32>,
}

impl BaseMeshAlgoData {
    /// `getDFace()`; `data.get_dface()`.
    pub fn get_dface(&self) -> &FaceHandle {
        &self.face
    }
    /// `getParameters()`; `data.get_parameters().deflection`.
    pub fn get_parameters(&self) -> &MeshParameters {
        &self.parameters
    }
    /// `getAllocator()`; `data.get_allocator()`.
    pub fn get_allocator(&self) -> &MeshAllocator {
        &self.allocator
    }
    /// `getStructure()`; `data.get_structure().nb_nodes()`.
    pub fn get_structure(&self) -> &Handle<BRepMeshDataStructureOfDelaun> {
        &self.structure
    }
    /// `getNodesMap()`: the registered 3d points; `data.get_nodes_map().len()`.
    pub fn get_nodes_map(&self) -> &[Point3] {
        &self.nodes_map
    }

    /// The Perform prologue: a fresh allocator, structure, nodes map and used nodes.
    fn start(&mut self, face: &FaceHandle, parameters: &MeshParameters) {
        self.face = face.clone();
        self.parameters = *parameters;
        self.allocator = Handle::new(IncrementalAllocatorObject::new(MEMORY_BLOCK_SIZE_HUGE));
        self.structure = Handle::new(BRepMeshDataStructureOfDelaun::new(&self.allocator, 100));
        self.nodes_map = Vec::with_capacity(256);
        self.used_nodes = DataMap::with_hasher(DefaultHashPolicy, 1);
    }

    /// The Perform epilogue: no link to the face or the structure is kept.
    fn finish(&mut self) {
        *self = Self::default();
    }
}

/// The virtual protected members of `BRepMesh_BaseMeshAlgo` over its data. The default
/// bodies are the native ones; an override reaches the base version through the
/// `base_*` functions, as the C++ calls `BaseAlgo::member`.
pub trait BaseMeshAlgoHooks {
    /// The base data; `algo.base_data().get_structure()`.
    fn base_data(&self) -> &BaseMeshAlgoData;
    /// The base data, mutable; `algo.base_data_mut()`.
    fn base_data_mut(&mut self) -> &mut BaseMeshAlgoData;

    /// `registerNode`; `algo.register_node(point, uv, DegreeOfFreedom::Free, false)?`.
    fn register_node(
        &mut self,
        point: Point3,
        point_2d: Point2,
        movability: DegreeOfFreedom,
        force: bool,
    ) -> Result<i32, Failure> {
        base_register_node(self, point, point_2d, movability, force)
    }
    /// `addNodeToStructure`; `algo.add_node_to_structure(uv, 0, DegreeOfFreedom::Free, false)?`.
    fn add_node_to_structure(
        &mut self,
        point: Point2,
        location_3d: i32,
        movability: DegreeOfFreedom,
        force: bool,
    ) -> Result<i32, Failure> {
        let vertex = BRepMeshVertex::new(point.xy(), location_3d, movability);
        self.base_data().structure.add_node(&vertex, force)
    }
    /// `getNodePoint2d`: the node coordinates; `algo.get_node_point2d(&vertex)`.
    fn get_node_point2d(&self, vertex: &BRepMeshVertex) -> Point2 {
        Point2::from(*vertex.coord())
    }
    /// `initDataStructure`; `algo.init_data_structure()?`.
    fn init_data_structure(&mut self) -> Result<bool, Failure> {
        base_init_data_structure(self)
    }
    /// `generateMesh` (pure virtual); `algo.generate_mesh(&range)?`.
    fn generate_mesh(&mut self, range: &ProgressRange<'_>) -> Result<(), Failure>;
}

/// The two virtual members `BRepMesh_ConstrainedBaseMeshAlgo` adds, with their native
/// defaults.
pub trait ConstrainedBaseMeshAlgoHooks: BaseMeshAlgoHooks {
    /// `getCellsCount`: `(-1, -1)`, the default circle grid; `algo.get_cells_count(10)?`.
    fn get_cells_count(&mut self, _vertices_nb: i32) -> Result<(i32, i32), Failure> {
        Ok((-1, -1))
    }
    /// `postProcessMesh`: nothing; `algo.post_process_mesh(&mut mesher, &range)?`.
    fn post_process_mesh(
        &mut self,
        _mesher: &mut BRepMeshDelaun,
        _range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        Ok(())
    }
}

/// `BRepMesh_BaseMeshAlgo::Perform`: any failure inside is caught (the native
/// `catch (Standard_Failure)`), and no link to the face is kept afterwards;
/// `base_perform(&mut algo, &face, &parameters, &range)`.
pub fn base_perform<A: BaseMeshAlgoHooks + ?Sized>(
    algo: &mut A,
    face: &FaceHandle,
    parameters: &MeshParameters,
    range: &ProgressRange<'_>,
) {
    algo.base_data_mut().start(face, parameters);
    // The native try block swallows every Standard_Failure: the face keeps whatever
    // status the steps set and no triangulation is committed.
    let _caught: Result<(), Failure> = perform_steps(algo, range);
    algo.base_data_mut().finish();
}

fn perform_steps<A: BaseMeshAlgoHooks + ?Sized>(
    algo: &mut A,
    range: &ProgressRange<'_>,
) -> Result<(), Failure> {
    if !algo.init_data_structure()? || !range.more() {
        return Ok(());
    }
    algo.generate_mesh(range)?;
    commit_surface_triangulation(algo)
}

/// `BRepMesh_BaseMeshAlgo::registerNode`: frontier and fixed nodes are pre-bound to
/// their own index so pcurve indices stay valid in the compact triangulation.
pub fn base_register_node<A: BaseMeshAlgoHooks + ?Sized>(
    algo: &mut A,
    point: Point3,
    point_2d: Point2,
    movability: DegreeOfFreedom,
    force: bool,
) -> Result<i32, Failure> {
    let length = algo.base_data().nodes_map.len() as i32;
    let node_index = algo.add_node_to_structure(point_2d, length, movability, force)?;
    if node_index > length {
        let data = algo.base_data_mut();
        data.nodes_map.push(point);
        if movability != DegreeOfFreedom::Free {
            data.used_nodes.bind(node_index, node_index);
        }
    }
    Ok(node_index)
}

/// `BRepMesh_BaseMeshAlgo::initDataStructure`: the frontier nodes and links of every
/// wire that does not self-intersect.
pub fn base_init_data_structure<A: BaseMeshAlgoHooks + ?Sized>(
    algo: &mut A,
) -> Result<bool, Failure> {
    let face = algo.base_data().face.clone();
    for wire_index in 0..face.wires_nb() {
        let wire = face.get_wire(wire_index)?;
        if wire.status().is_set(Status::SelfIntersectingWire) {
            continue;
        }
        for edge_index in 0..wire.edges_nb() {
            let edge = wire.get_edge(edge_index)?.upgrade();
            let orientation = wire.get_edge_orientation(edge_index)?;
            let pcurve = edge.get_pcurve_for(&face.downgrade(), orientation)?;
            register_wire_pcurve(algo, &edge, &pcurve, wire_index)?;
            register_other_pcurves(algo, &edge, &pcurve)?;
        }
    }
    Ok(true)
}

/// The nodes and constraint links of the pcurve of one wire occurrence of an edge.
fn register_wire_pcurve<A: BaseMeshAlgoHooks + ?Sized>(
    algo: &mut A,
    edge: &EdgeHandle,
    pcurve: &PCurveHandle,
    wire_index: i32,
) -> Result<(), Failure> {
    let orientation = fix_seam_edge_orientation(algo.base_data(), edge, pcurve)?;
    let curve = edge.get_curve();
    let mut previous = -1;
    for point_index in 0..pcurve.parameters_nb() {
        let node = algo.register_node(
            curve.get_point(point_index)?,
            pcurve.get_point(point_index)?,
            DegreeOfFreedom::Frontier,
            false,
        )?;
        pcurve.set_index(point_index, node)?;
        if previous != -1 && previous != node {
            add_wire_link(algo.base_data(), previous, node, orientation, wire_index)?;
        }
        previous = node;
    }
    Ok(())
}

/// A link of a wire; a link that already existed in an inner wire becomes fixed
/// ("prevent holes around wire of zero area").
fn add_wire_link(
    data: &BaseMeshAlgoData,
    first: i32,
    last: i32,
    orientation: Orientation,
    wire_index: i32,
) -> Result<(), Failure> {
    let links_nb = data.structure.nb_links();
    let link = add_link_to_mesh(&data.structure, first, last, orientation)?;
    if wire_index != 0 && link <= links_nb {
        data.structure
            .set_link_movability(link, DegreeOfFreedom::Fixed)?;
    }
    Ok(())
}

/// The other pcurves of the edge on this face get node indices (for their polygons
/// on triangulation) but no constraint.
fn register_other_pcurves<A: BaseMeshAlgoHooks + ?Sized>(
    algo: &mut A,
    edge: &EdgeHandle,
    pcurve: &PCurveHandle,
) -> Result<(), Failure> {
    let face = algo.base_data().face.downgrade();
    let curve = edge.get_curve();
    for other_index in edge.get_pcurves(&face)? {
        let other = edge.get_pcurve(other_index)?;
        if other.object_address() == pcurve.object_address() {
            continue;
        }
        for point_index in 0..other.parameters_nb() {
            let node = algo.register_node(
                curve.get_point(point_index)?,
                other.get_point(point_index)?,
                DegreeOfFreedom::Frontier,
                false,
            )?;
            other.set_index(point_index, node)?;
        }
    }
    Ok(())
}

/// `addLinkToMesh`: reversed frontier, internal fixed or forward frontier link; the
/// absolute link index.
fn add_link_to_mesh(
    structure: &Handle<BRepMeshDataStructureOfDelaun>,
    first: i32,
    last: i32,
    orientation: Orientation,
) -> Result<i32, Failure> {
    let edge = match orientation {
        Orientation::Reversed => BRepMeshEdge::new(last, first, DegreeOfFreedom::Frontier),
        Orientation::Internal => BRepMeshEdge::new(first, last, DegreeOfFreedom::Fixed),
        _ => BRepMeshEdge::new(first, last, DegreeOfFreedom::Frontier),
    };
    Ok(structure.add_link(&edge)?.abs())
}

/// `fixSeamEdgeOrientation`: INTERNAL when another pcurve of the edge on this face
/// has both ends within Precision::Confusion() of the ends of this one.
fn fix_seam_edge_orientation(
    data: &BaseMeshAlgoData,
    edge: &EdgeHandle,
    pcurve: &PCurveHandle,
) -> Result<Orientation, Failure> {
    for index in 0..edge.pcurves_nb() {
        let other = edge.get_pcurve(index)?;
        if other.get_face().object_address() != data.face.object_address()
            || other.object_address() == pcurve.object_address()
        {
            continue;
        }
        if ends_coincide(pcurve, &other)? {
            return Ok(Orientation::Internal);
        }
    }
    Ok(pcurve.get_orientation())
}

/// Both ends of `first` within the square confusion of an end of `second`.
fn ends_coincide(first: &PCurveHandle, second: &PCurveHandle) -> Result<bool, Failure> {
    let (a1, a2) = (
        first.get_point(0)?,
        first.get_point(first.parameters_nb() - 1)?,
    );
    let (b1, b2) = (
        second.get_point(0)?,
        second.get_point(second.parameters_nb() - 1)?,
    );
    let dist1 = a1.square_distance(b1).min(a1.square_distance(b2));
    let dist2 = a2.square_distance(b1).min(a2.square_distance(b2));
    let tolerance = precision::square_confusion();
    Ok(dist1 < tolerance && dist2 < tolerance)
}

/// `commitSurfaceTriangulation`: Failure status without domain triangle, else the
/// triangulation stored in the face.
fn commit_surface_triangulation<A: BaseMeshAlgoHooks + ?Sized>(
    algo: &mut A,
) -> Result<(), Failure> {
    let Some(triangulation) = super::collect::collect_triangles(algo.base_data_mut())? else {
        algo.base_data().face.status().set_status(Status::Failure);
        return Ok(());
    };
    super::collect::collect_nodes(algo, &triangulation)?;
    let face = algo.base_data().face.get_face()?;
    crate::brep_mesh::ShapeTool::add_in_face(&face, &triangulation)
}

/// Mutable access to the used-node map for the collection step.
pub(super) fn used_nodes_mut(data: &mut BaseMeshAlgoData) -> &mut DataMap<i32, i32> {
    &mut data.used_nodes
}

/// Read access to the used-node map for the collection step.
pub(super) fn used_nodes(data: &BaseMeshAlgoData) -> &DataMap<i32, i32> {
    &data.used_nodes
}
