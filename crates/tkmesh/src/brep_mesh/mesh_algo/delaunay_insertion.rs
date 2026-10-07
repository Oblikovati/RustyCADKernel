// Rust translation of BRepMesh_DelaunayNodeInsertionMeshAlgo.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::base::{BaseMeshAlgoData, ConstrainedBaseMeshAlgoHooks};
use super::generator::MeshGenerator;
use super::node_insertion::{
    NodeInsertionData, NodeInsertionHooks, node_insertion_hooks, node_insertion_init_data_structure,
};
use crate::brep_mesh::{BRepMeshDelaun, DegreeOfFreedom, GeomTool, RangeSplitter, SurfaceNodes};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point2;
use std::marker::PhantomData;

/// The node-insertion algorithms that add surface nodes to the Delaunay mesh.
pub trait DelaunayInsertionHooks: NodeInsertionHooks {
    /// `IsPreProcessSurfaceNodes()`; `algo.is_pre_process_surface_nodes()`.
    fn is_pre_process_surface_nodes(&self) -> bool;
}

/// `BRepMesh_DelaunayNodeInsertionMeshAlgo::initDataStructure`: the node-insertion
/// structure, then (when pre-processing) the surface nodes registered before meshing.
pub fn delaunay_insertion_init_data_structure<A: DelaunayInsertionHooks + ?Sized>(
    algo: &mut A,
) -> Result<bool, Failure> {
    if !node_insertion_init_data_structure(algo)? {
        return Ok(false);
    }
    if algo.is_pre_process_surface_nodes() {
        let nodes = generate_surface_nodes(algo)?;
        register_surface_nodes(algo, &nodes)?;
    }
    Ok(true)
}

/// `getCellsCount`: BRepMesh_GeomTool::CellsCount of the face surface, deflection and
/// splitter.
pub fn delaunay_insertion_cells_count<A: DelaunayInsertionHooks + ?Sized>(
    algo: &A,
    vertices_nb: i32,
) -> Result<(i32, i32), Failure> {
    let face = algo.base_data().get_dface();
    GeomTool::cells_count(
        &face.get_surface().cast(),
        vertices_nb,
        face.get_deflection(),
        Some(algo.insertion().get_range_splitter().range_state()),
    )
}

/// `BRepMesh_DelaunayNodeInsertionMeshAlgo::postProcessMesh`: the base post-process
/// (with a default range, as natively), then the surface nodes inserted into the mesh
/// unless they were registered beforehand.
pub fn delaunay_insertion_post_process<A: DelaunayInsertionHooks + ?Sized, G: MeshGenerator>(
    algo: &mut A,
    mesher: &mut BRepMeshDelaun,
    range: &ProgressRange<'_>,
) -> Result<(), Failure> {
    if !range.more() {
        return Ok(());
    }
    G::post_process_mesh(algo, mesher, &ProgressRange::default())?;
    if !algo.is_pre_process_surface_nodes() {
        let nodes = generate_surface_nodes(algo)?;
        insert_nodes(algo, &nodes, mesher, range)?;
    }
    Ok(())
}

fn generate_surface_nodes<A: DelaunayInsertionHooks + ?Sized>(
    algo: &mut A,
) -> Result<SurfaceNodes, Failure> {
    let parameters = *algo.base_data().get_parameters();
    algo.insertion_mut()
        .range_splitter_mut()
        .generate_surface_nodes(&parameters)
}

/// `insertNodes`: the nodes classified inside registered as free nodes and inserted
/// into the mesh; whether any was (false on a user break).
pub fn insert_nodes<A: DelaunayInsertionHooks + ?Sized>(
    algo: &mut A,
    nodes: &SurfaceNodes,
    mesher: &mut BRepMeshDelaun,
    range: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let Some(list) = nodes.get().filter(|list| !list.is_empty()) else {
        return Ok(false);
    };
    let mut indices = Vec::with_capacity(list.len());
    for point in list.iter() {
        if let Some(index) = register_inside(algo, *point)? {
            indices.push(index);
        }
    }
    mesher.add_vertices(&mut indices, range)?;
    if !range.more() {
        return Ok(false);
    }
    Ok(!indices.is_empty())
}

/// `registerSurfaceNodes`: the nodes classified inside registered as free nodes.
fn register_surface_nodes<A: DelaunayInsertionHooks + ?Sized>(
    algo: &mut A,
    nodes: &SurfaceNodes,
) -> Result<bool, Failure> {
    let Some(list) = nodes.get().filter(|list| !list.is_empty()) else {
        return Ok(false);
    };
    let mut added = false;
    for point in list.iter() {
        added |= register_inside(algo, *point)?.is_some();
    }
    Ok(added)
}

/// A free node at `point` (and its surface point) when the classifier keeps it.
fn register_inside<A: DelaunayInsertionHooks + ?Sized>(
    algo: &mut A,
    point: Point2,
) -> Result<Option<i32>, Failure> {
    if algo.insertion().get_classifier().perform(point) != State::In {
        return Ok(None);
    }
    let point_3d = algo
        .insertion()
        .get_range_splitter()
        .range_state()
        .point(point)?;
    algo.register_node(point_3d, point, DegreeOfFreedom::Free, false)
        .map(Some)
}

/// `BRepMesh_DelaunayNodeInsertionMeshAlgo<RangeSplitter, BaseAlgo>`: node insertion
/// with the surface nodes of the splitter added after (or before) the base mesh;
/// `DelaunayNodeInsertionMeshAlgo::<SphereRangeSplitter, DelaunayGenerator>::default()`.
pub struct DelaunayNodeInsertionMeshAlgo<R: RangeSplitter, G: MeshGenerator> {
    data: BaseMeshAlgoData,
    insertion: NodeInsertionData<R>,
    pre_process_surface_nodes: bool,
    generator: PhantomData<G>,
}

impl<R: RangeSplitter + Default, G: MeshGenerator> Default for DelaunayNodeInsertionMeshAlgo<R, G> {
    fn default() -> Self {
        Self {
            data: BaseMeshAlgoData::default(),
            insertion: NodeInsertionData::default(),
            pre_process_surface_nodes: false,
            generator: PhantomData,
        }
    }
}

impl<R: RangeSplitter, G: MeshGenerator> DelaunayNodeInsertionMeshAlgo<R, G> {
    /// `SetPreProcessSurfaceNodes`: register the surface nodes before the base mesh
    /// instead of inserting them after it; `algo.set_pre_process_surface_nodes(true)`.
    pub fn set_pre_process_surface_nodes(&mut self, value: bool) {
        self.pre_process_surface_nodes = value;
    }
    fn init_structure(&mut self) -> Result<bool, Failure> {
        delaunay_insertion_init_data_structure(self)
    }
}

node_insertion_hooks!(DelaunayNodeInsertionMeshAlgo);

impl<R: RangeSplitter, G: MeshGenerator> DelaunayInsertionHooks
    for DelaunayNodeInsertionMeshAlgo<R, G>
{
    fn is_pre_process_surface_nodes(&self) -> bool {
        self.pre_process_surface_nodes
    }
}

impl<R: RangeSplitter, G: MeshGenerator> ConstrainedBaseMeshAlgoHooks
    for DelaunayNodeInsertionMeshAlgo<R, G>
{
    fn get_cells_count(&mut self, vertices_nb: i32) -> Result<(i32, i32), Failure> {
        delaunay_insertion_cells_count(self, vertices_nb)
    }
    fn post_process_mesh(
        &mut self,
        mesher: &mut BRepMeshDelaun,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        delaunay_insertion_post_process::<Self, G>(self, mesher, range)
    }
}
