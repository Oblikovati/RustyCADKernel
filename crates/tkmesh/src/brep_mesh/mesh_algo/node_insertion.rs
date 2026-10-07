// Rust translation of BRepMesh_NodeInsertionMeshAlgo.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::base::{
    BaseMeshAlgoData, ConstrainedBaseMeshAlgoHooks, base_init_data_structure, base_perform,
};
use super::generator::MeshGenerator;
use crate::brep_mesh::{BRepMeshClassifier, BRepMeshVertex, DegreeOfFreedom, RangeSplitter};
use crate::imesh_data::{FaceHandle, Status, WireHandle};
use crate::imesh_tools::MeshParameters;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point2;
use std::marker::PhantomData;

/// The data `BRepMesh_NodeInsertionMeshAlgo` adds: the range splitter and the
/// classifier of the face wires (live during one Perform).
#[derive(Default)]
pub struct NodeInsertionData<R: RangeSplitter> {
    range_splitter: R,
    classifier: Handle<BRepMeshClassifier>,
}

impl<R: RangeSplitter> NodeInsertionData<R> {
    /// `getRangeSplitter()`; `data.get_range_splitter().get_delta()`.
    pub fn get_range_splitter(&self) -> &R {
        &self.range_splitter
    }
    /// The range splitter, mutable (GenerateSurfaceNodes fills NURBS parameters).
    pub fn range_splitter_mut(&mut self) -> &mut R {
        &mut self.range_splitter
    }
    /// `getClassifier()`; `data.get_classifier()`.
    pub fn get_classifier(&self) -> &Handle<BRepMeshClassifier> {
        &self.classifier
    }
}

/// Access to the node-insertion data of an algorithm built on
/// `BRepMesh_NodeInsertionMeshAlgo`.
pub trait NodeInsertionHooks: ConstrainedBaseMeshAlgoHooks {
    /// The range splitter type (`RangeSplitter` template argument).
    type Splitter: RangeSplitter;
    /// The node-insertion data; `algo.insertion().get_classifier()`.
    fn insertion(&self) -> &NodeInsertionData<Self::Splitter>;
    /// The node-insertion data, mutable.
    fn insertion_mut(&mut self) -> &mut NodeInsertionData<Self::Splitter>;
}

/// `BRepMesh_NodeInsertionMeshAlgo::Perform`: resets the splitter (a failure there
/// leaves the algorithm, as the native exception does), makes the classifier, then
/// the base Perform.
pub fn node_insertion_perform<A: NodeInsertionHooks + ?Sized>(
    algo: &mut A,
    face: &FaceHandle,
    parameters: &MeshParameters,
    range: &ProgressRange<'_>,
) -> Result<(), Failure> {
    algo.insertion_mut()
        .range_splitter
        .reset(face, parameters)?;
    algo.insertion_mut().classifier = Handle::new(BRepMeshClassifier::new());
    if !range.more() {
        return Ok(());
    }
    base_perform(algo, face, parameters, range);
    algo.insertion_mut().classifier = Handle::default();
    Ok(())
}

/// `BRepMesh_NodeInsertionMeshAlgo::initDataStructure`: the wire points (which also
/// bound the splitter range), the vertex tool grid, the classifier wires, the internal
/// vertices, then the base data structure. Failure status and false for an invalid
/// range.
pub fn node_insertion_init_data_structure<A: NodeInsertionHooks + ?Sized>(
    algo: &mut A,
) -> Result<bool, Failure> {
    let face = algo.base_data().get_dface().clone();
    let mut wires: Vec<Option<Vec<Point2>>> = Vec::new();
    for wire_index in 0..face.wires_nb() {
        let wire = face.get_wire(wire_index)?;
        let skipped = wire.status().is_set(Status::SelfIntersectingWire)
            || (wire.status().is_set(Status::OpenWire) && wire_index != 0);
        wires.push(if skipped {
            None
        } else {
            Some(collect_wire_points(algo, &face, &wire)?)
        });
    }
    algo.insertion_mut().range_splitter.adjust_range()?;
    if !algo.insertion().range_splitter.is_valid() {
        face.status().set_status(Status::Failure);
        return Ok(false);
    }
    configure_structure_grid(algo)?;
    register_classifier_wires(algo, &wires)?;
    if algo.base_data().get_parameters().internal_vertices_mode {
        insert_internal_vertices(algo)?;
    }
    base_init_data_structure(algo)
}

/// The vertex tool cell size (14 parametric tolerances) and tolerance, both in the
/// scaled face basis.
fn configure_structure_grid<A: NodeInsertionHooks + ?Sized>(algo: &A) -> Result<(), Failure> {
    let splitter = algo.insertion().range_splitter.range_state();
    let (delta, tol_uv) = (splitter.get_delta(), splitter.get_tolerance_uv());
    let tool = algo.base_data().get_structure().vertex_tool();
    tool.set_cell_sizes(14.0 * tol_uv.0 / delta.0, 14.0 * tol_uv.1 / delta.1)?;
    tool.set_tolerances(tol_uv.0 / delta.0, tol_uv.1 / delta.1);
    Ok(())
}

fn register_classifier_wires<A: NodeInsertionHooks + ?Sized>(
    algo: &A,
    wires: &[Option<Vec<Point2>>],
) -> Result<(), Failure> {
    let splitter = algo.insertion().range_splitter.range_state();
    let tol_uv = splitter.get_tolerance_uv();
    for wire in wires.iter().flatten().filter(|wire| !wire.is_empty()) {
        algo.insertion().classifier.register_wire(
            wire,
            tol_uv,
            splitter.get_range_u(),
            splitter.get_range_v(),
        )?;
    }
    Ok(())
}

/// `collectWirePoints`: the pcurve points of each wire edge in wire order, the last
/// point of each pcurve (in its traversal direction) excluded; each point also
/// extends the splitter range.
fn collect_wire_points<A: NodeInsertionHooks + ?Sized>(
    algo: &mut A,
    face: &FaceHandle,
    wire: &WireHandle,
) -> Result<Vec<Point2>, Failure> {
    let mut points = Vec::new();
    for edge_index in 0..wire.edges_nb() {
        let edge = wire.get_edge(edge_index)?.upgrade();
        let pcurve =
            edge.get_pcurve_for(&face.downgrade(), wire.get_edge_orientation(edge_index)?)?;
        let last = pcurve.parameters_nb() - 1;
        let (mut index, end, step) = if pcurve.is_forward() {
            (std::cmp::min(0, last), last, 1)
        } else {
            (last, std::cmp::min(0, last), -1)
        };
        while index != end {
            let point = pcurve.get_point(index)?;
            points.push(point);
            algo.insertion_mut().range_splitter.add_point(point);
            index += step;
        }
    }
    Ok(points)
}

/// `insertInternalVertices`: the INTERNAL vertices of the face that are not on an edge.
fn insert_internal_vertices<A: NodeInsertionHooks + ?Sized>(algo: &mut A) -> Result<(), Failure> {
    let face = algo.base_data().get_dface().get_face()?;
    for shape in Explorer::new(&face, ShapeEnum::Vertex, ShapeEnum::Edge)? {
        if shape.orientation() == Orientation::Internal {
            // Native try/catch around each vertex: a failure skips only that vertex.
            let _skipped: Result<(), Failure> = insert_internal_vertex(algo, &face, &shape);
        }
    }
    Ok(())
}

/// `insertInternalVertex`: a fixed node at the vertex parameters when they classify
/// inside the face.
fn insert_internal_vertex<A: NodeInsertionHooks + ?Sized>(
    algo: &mut A,
    face: &topods::Face,
    shape: &topods::Shape,
) -> Result<(), Failure> {
    let vertex = topods::vertex(shape)?;
    let point_2d = BRepTool::parameters(&vertex, face)?;
    if algo.insertion().classifier.perform(point_2d) != State::In {
        return Ok(());
    }
    algo.register_node(
        BRepTool::pnt(&vertex)?,
        point_2d,
        DegreeOfFreedom::Fixed,
        false,
    )?;
    Ok(())
}

/// `BRepMesh_NodeInsertionMeshAlgo::addNodeToStructure`: the point scaled to the
/// face basis, then the base version.
pub fn node_insertion_add_node_to_structure<A: NodeInsertionHooks + ?Sized>(
    algo: &mut A,
    point: Point2,
    location_3d: i32,
    movability: DegreeOfFreedom,
    force: bool,
) -> Result<i32, Failure> {
    let scaled = algo
        .insertion()
        .range_splitter
        .range_state()
        .scale(point, true);
    let vertex = BRepMeshVertex::new(scaled.xy(), location_3d, movability);
    algo.base_data().get_structure().add_node(&vertex, force)
}

/// `BRepMesh_NodeInsertionMeshAlgo::getNodePoint2d`: the node scaled back to the
/// surface parameters.
pub fn node_insertion_point2d<A: NodeInsertionHooks + ?Sized>(
    algo: &A,
    vertex: &BRepMeshVertex,
) -> Point2 {
    algo.insertion()
        .range_splitter
        .range_state()
        .scale(Point2::from(*vertex.coord()), false)
}

/// `BRepMesh_NodeInsertionMeshAlgo<RangeSplitter, BaseAlgo>`: the base algorithm `G`
/// over the scaled face basis of the splitter, with internal vertices;
/// `NodeInsertionMeshAlgo::<DefaultRangeSplitter, DelaunayGenerator>::default()`.
pub struct NodeInsertionMeshAlgo<R: RangeSplitter, G: MeshGenerator> {
    data: BaseMeshAlgoData,
    insertion: NodeInsertionData<R>,
    generator: PhantomData<G>,
}

impl<R: RangeSplitter + Default, G: MeshGenerator> Default for NodeInsertionMeshAlgo<R, G> {
    fn default() -> Self {
        Self {
            data: BaseMeshAlgoData::default(),
            insertion: NodeInsertionData {
                range_splitter: R::default(),
                classifier: Handle::default(),
            },
            generator: PhantomData,
        }
    }
}

/// The base and node-insertion hooks shared by the node-insertion family; the
/// generate step is `G::generate_mesh`.
macro_rules! node_insertion_hooks {
    ($algo:ident) => {
        impl<R: $crate::brep_mesh::RangeSplitter, G: $crate::brep_mesh::mesh_algo::MeshGenerator>
            $crate::brep_mesh::mesh_algo::BaseMeshAlgoHooks for $algo<R, G>
        {
            fn base_data(&self) -> &$crate::brep_mesh::mesh_algo::BaseMeshAlgoData {
                &self.data
            }
            fn base_data_mut(&mut self) -> &mut $crate::brep_mesh::mesh_algo::BaseMeshAlgoData {
                &mut self.data
            }
            fn add_node_to_structure(
                &mut self,
                point: rustycad_tkmath::gp::Point2,
                location_3d: i32,
                movability: $crate::brep_mesh::DegreeOfFreedom,
                force: bool,
            ) -> Result<i32, rustycad_tkernel::standard::Failure> {
                $crate::brep_mesh::mesh_algo::node_insertion_add_node_to_structure(
                    self,
                    point,
                    location_3d,
                    movability,
                    force,
                )
            }
            fn get_node_point2d(
                &self,
                vertex: &$crate::brep_mesh::BRepMeshVertex,
            ) -> rustycad_tkmath::gp::Point2 {
                $crate::brep_mesh::mesh_algo::node_insertion_point2d(self, vertex)
            }
            fn init_data_structure(&mut self) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.init_structure()
            }
            fn generate_mesh(
                &mut self,
                range: &rustycad_tkernel::message::ProgressRange<'_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                G::generate_mesh(self, range)
            }
        }
        impl<R: $crate::brep_mesh::RangeSplitter, G: $crate::brep_mesh::mesh_algo::MeshGenerator>
            $crate::brep_mesh::mesh_algo::NodeInsertionHooks for $algo<R, G>
        {
            type Splitter = R;
            fn insertion(&self) -> &$crate::brep_mesh::mesh_algo::NodeInsertionData<R> {
                &self.insertion
            }
            fn insertion_mut(&mut self) -> &mut $crate::brep_mesh::mesh_algo::NodeInsertionData<R> {
                &mut self.insertion
            }
        }
    };
}
pub(super) use node_insertion_hooks;

node_insertion_hooks!(NodeInsertionMeshAlgo);

impl<R: RangeSplitter, G: MeshGenerator> NodeInsertionMeshAlgo<R, G> {
    fn init_structure(&mut self) -> Result<bool, Failure> {
        node_insertion_init_data_structure(self)
    }
}

impl<R: RangeSplitter, G: MeshGenerator> ConstrainedBaseMeshAlgoHooks
    for NodeInsertionMeshAlgo<R, G>
{
    fn post_process_mesh(
        &mut self,
        mesher: &mut crate::brep_mesh::BRepMeshDelaun,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        G::post_process_mesh(self, mesher, range)
    }
}
