//! Native five-iteration refinement and pointer-map traversal.
use super::super::BRepMeshEdgeDiscret;
use super::*;
use crate::imesh_data::{EdgePtr, MEMORY_BLOCK_SIZE_HUGE};
use rustycad_tkernel::ncollection::{DefaultHashPolicy, IncrementalAllocatorObject, Map, map_algo};
impl BRepMeshModelHealer {
    pub(super) fn amplify_edges(&self) -> Result<(), Failure> {
        let allocator = IncrementalAllocatorObject::new(MEMORY_BLOCK_SIZE_HUGE);
        let mut edges: Map<EdgePtr> =
            Map::with_allocator(DefaultHashPolicy, 1, allocator.resource().clone());
        for _ in 0..5 {
            if !self.pop_edges(&mut edges) {
                break;
            }
            self.amplify_iteration(&edges, allocator.resource().clone())?;
            // Rust allocation guards must be released before the native arena rewind.
            edges.clear(false);
            allocator.resource().reset(false)?;
        }
        Ok(())
    }
    fn amplify_iteration(
        &self,
        edges: &Map<EdgePtr>,
        allocator: rustycad_tkernel::ncollection::BaseAllocator,
    ) -> Result<(), Failure> {
        let parameters = *self.parameters.read();
        crate::model_parallel::model_items(
            edges.iter().cloned().collect(),
            parameters.in_parallel,
            |edge| amplify(edge, &parameters),
        )?;
        let faces = affected_faces(edges, allocator)?;
        crate::model_parallel::model_items(
            faces.iter().cloned().collect(),
            parameters.in_parallel,
            |face| self.process_pointer(face),
        )
    }
    fn process_pointer(&self, face: FacePtr) -> Result<(), Failure> {
        self.process_face(&face.upgrade());
        Ok(())
    }
    fn pop_edges(&self, edges: &mut Map<EdgePtr>) -> bool {
        for (_, intersections) in self.intersections.read().write().items_mut() {
            if intersections.is_null() {
                continue;
            }
            map_algo::unite(edges, &intersections.read());
            *intersections = Handle::default();
        }
        !edges.is_empty()
    }
}
fn affected_faces(
    edges: &Map<EdgePtr>,
    allocator: rustycad_tkernel::ncollection::BaseAllocator,
) -> Result<Map<FacePtr>, Failure> {
    let mut faces = Map::with_allocator(DefaultHashPolicy, 1, allocator);
    for edge in edges.iter() {
        let edge = edge.upgrade();
        for index in 0..edge.pcurves_nb() {
            faces.add(edge.get_pcurve(index)?.get_face());
        }
    }
    Ok(faces)
}
fn amplify(pointer: EdgePtr, parameters: &MeshParameters) -> Result<(), Failure> {
    let edge = pointer.upgrade();
    let mut points = edge.get_curve().parameters_nb();
    edge.clear(true)?;
    edge.set_deflection(rustycad_tkg2d::geom2d::native::std_max(
        edge.get_deflection() / 3.,
        rustycad_tkernel::precision::confusion(),
    ));
    if has_small_outer_wire(&edge)? {
        points += 1;
    }
    tessellate_amplified(&edge, parameters, points)
}
fn tessellate_amplified(
    edge: &crate::imesh_data::EdgeHandle,
    parameters: &MeshParameters,
    points: i32,
) -> Result<(), Failure> {
    let curve = edge.get_pcurve(0)?;
    let face = curve.get_face().upgrade();
    let tessellator = BRepMeshEdgeDiscret::create_face_tessellator(
        edge,
        curve.get_orientation(),
        &face,
        parameters,
        points,
    )?;
    BRepMeshEdgeDiscret::tessellate_3d(edge, &tessellator, false)?;
    BRepMeshEdgeDiscret::tessellate_2d(edge, false)
}

fn has_small_outer_wire(edge: &crate::imesh_data::EdgeHandle) -> Result<bool, Failure> {
    for index in 0..edge.pcurves_nb() {
        if edge
            .get_pcurve(index)?
            .get_face()
            .upgrade()
            .get_wire(0)?
            .edges_nb()
            <= 2
        {
            return Ok(true);
        }
    }
    Ok(false)
}
