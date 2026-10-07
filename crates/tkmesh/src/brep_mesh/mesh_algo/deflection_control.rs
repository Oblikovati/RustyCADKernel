// Rust translation of BRepMesh_DelaunayDeflectionControlMeshAlgo.hxx (the algorithm
// and optimizeMesh) at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::base::{BaseMeshAlgoData, ConstrainedBaseMeshAlgoHooks};
use super::deflection_split::DeflectionPass;
use super::delaunay_insertion::{
    DelaunayInsertionHooks, delaunay_insertion_cells_count, delaunay_insertion_init_data_structure,
    delaunay_insertion_post_process, insert_nodes,
};
use super::generator::MeshGenerator;
use super::node_insertion::{NodeInsertionData, node_insertion_hooks};
use crate::brep_mesh::{BRepMeshDelaun, RangeSplitter};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{Map, Shared};
use rustycad_tkernel::standard::{Failure, Handle};
use std::marker::PhantomData;

/// `BRepMesh_DelaunayDeflectionControlMeshAlgo<RangeSplitter, BaseAlgo>`: Delaunay
/// node insertion followed by up to 11 passes inserting the triangle centers and link
/// middles that deviate from the surface by more than the face deflection;
/// `DelaunayDeflectionControlMeshAlgo::<NURBSRangeSplitter, DelaunayGenerator>::default()`.
pub struct DelaunayDeflectionControlMeshAlgo<R: RangeSplitter, G: MeshGenerator> {
    data: BaseMeshAlgoData,
    insertion: NodeInsertionData<R>,
    pre_process_surface_nodes: bool,
    generator: PhantomData<G>,
}

impl<R: RangeSplitter + Default, G: MeshGenerator> Default
    for DelaunayDeflectionControlMeshAlgo<R, G>
{
    fn default() -> Self {
        Self {
            data: BaseMeshAlgoData::default(),
            insertion: NodeInsertionData::default(),
            pre_process_surface_nodes: false,
            generator: PhantomData,
        }
    }
}

node_insertion_hooks!(DelaunayDeflectionControlMeshAlgo);

impl<R: RangeSplitter, G: MeshGenerator> DelaunayInsertionHooks
    for DelaunayDeflectionControlMeshAlgo<R, G>
{
    fn is_pre_process_surface_nodes(&self) -> bool {
        self.pre_process_surface_nodes
    }
}

impl<R: RangeSplitter, G: MeshGenerator> ConstrainedBaseMeshAlgoHooks
    for DelaunayDeflectionControlMeshAlgo<R, G>
{
    fn get_cells_count(&mut self, vertices_nb: i32) -> Result<(i32, i32), Failure> {
        delaunay_insertion_cells_count(self, vertices_nb)
    }
    /// `postProcessMesh`: the surface nodes, then the deflection control when it is
    /// enabled and the domain has triangles.
    fn post_process_mesh(
        &mut self,
        mesher: &mut BRepMeshDelaun,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let scope = ProgressScope::with_literal(range, "Post process mesh", 2.0, false);
        delaunay_insertion_post_process::<Self, G>(self, mesher, &scope.next(1.0))?;
        if !scope.more() {
            return Ok(());
        }
        let has_triangles = self.data.get_structure().elements_of_domain().extent() > 0;
        if self.data.get_parameters().control_surface_deflection && has_triangles {
            return self.optimize_mesh(mesher, &scope.next(1.0));
        }
        drop(scope.next(1.0));
        Ok(())
    }
}

impl<R: RangeSplitter, G: MeshGenerator> DelaunayDeflectionControlMeshAlgo<R, G> {
    /// `SetPreProcessSurfaceNodes`; `algo.set_pre_process_surface_nodes(true)`.
    pub fn set_pre_process_surface_nodes(&mut self, value: bool) {
        self.pre_process_surface_nodes = value;
    }

    fn init_structure(&mut self) -> Result<bool, Failure> {
        delaunay_insertion_init_data_structure(self)
    }

    /// `optimizeMesh`: the deflection passes; the face deflection becomes the largest
    /// deviation measured in the last pass.
    fn optimize_mesh(
        &mut self,
        mesher: &mut BRepMeshDelaun,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let min_size = self.data.get_parameters().min_size;
        let extent = self.data.get_structure().elements_of_domain().extent();
        let mut pass = DeflectionPass::new(min_size * min_size, Map::new(3 * extent as usize));
        let scope = ProgressScope::with_literal(range, "Iteration", 11.0, false);
        let mut is_inserted = true;
        let mut pass_index = 1;
        while pass_index <= 11 && is_inserted && !pass.all_degenerated {
            if !scope.more() {
                return Ok(());
            }
            if !pass.start(&self.data) {
                break;
            }
            pass.split_domain(self, mesher)?;
            let nodes = Handle::new(Shared::new(std::mem::take(&mut pass.control_nodes)));
            is_inserted = insert_nodes(self, &nodes, mesher, &scope.next(1.0))?;
            pass_index += 1;
        }
        if pass.max_sq_deflection.partial_cmp(&0.0) != Some(std::cmp::Ordering::Less) {
            self.data
                .get_dface()
                .set_deflection(pass.max_sq_deflection.sqrt());
        }
        Ok(())
    }
}
