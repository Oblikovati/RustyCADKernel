use super::*;
use crate::imesh_data::{EdgeHandle, MEMORY_BLOCK_SIZE_HUGE};
use rustycad_tkernel::ncollection::{DefaultHashPolicy, IncrementalAllocatorObject, Map};
pub(super) fn clear_outdated_polygons(model: &Handle<dyn MeshModel>) -> Result<(), Failure> {
    let allocator = IncrementalAllocatorObject::new(MEMORY_BLOCK_SIZE_HUGE);
    let mut used_faces = Map::with_allocator(DefaultHashPolicy, 1, allocator.resource().clone());
    for index in 0..model.edges_nb() {
        let edge = model.get_edge(index)?;
        if edge.is_free() {
            clear_free_edge(&edge)?;
            continue;
        }
        clear_adjacent_faces(&edge, &mut used_faces)?;
    }
    Ok(())
}
fn clear_free_edge(edge: &EdgeHandle) -> Result<(), Failure> {
    if !edge.status().is_set(Status::Outdated) {
        return Ok(());
    }
    let topology = edge.get_edge()?;
    let (_, location) = BRepTool::polygon3d(&topology)?;
    ShapeTool::nullify_edge_polygon3d(&topology, &location)
}
fn clear_adjacent_faces(edge: &EdgeHandle, used_faces: &mut Map<usize>) -> Result<(), Failure> {
    for index in 0..edge.pcurves_nb() {
        let face = edge.get_pcurve(index)?.get_face().upgrade();
        if !used_faces.add(face.object_address() as usize) {
            continue;
        }
        if face.status().is_set(Status::Outdated) {
            clear_face_polygons(&face)?;
        }
    }
    Ok(())
}
fn clear_face_polygons(face: &FaceHandle) -> Result<(), Failure> {
    let topology = face.get_face()?;
    let (mesh, location) = BRepTool::triangulation(&topology, mesh_purpose::NONE)?;
    for wire_index in 0..face.wires_nb() {
        let wire = face.get_wire(wire_index)?;
        for edge_index in 0..wire.edges_nb() {
            let edge = wire.get_edge(edge_index)?.upgrade().get_edge()?;
            ShapeTool::nullify_edge_on_triangulation(&edge, &mesh, &location)?;
        }
    }
    ShapeTool::nullify_face(&topology)
}
