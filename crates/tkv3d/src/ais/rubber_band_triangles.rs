//! Native frontier-constrained Delaunay filling, including array reuse and domain order.
use super::AisRubberBandSource;
use rustycad_tkernel::{
    ncollection::{IncrementalAllocatorObject, Vec2},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::Xy;
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDataStructureOfDelaun, BRepMeshDelaun, BRepMeshEdge, BRepMeshVertex, DegreeOfFreedom,
};
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
/// Fill the native cached triangle array; `ais_rubber_band_fill_triangles(&*band)?`.
pub fn ais_rubber_band_fill_triangles<O: AisRubberBandSource + ?Sized>(
    object: &O,
) -> Result<bool, Failure> {
    let points: Vec<Vec2<i32>> = object.points().read().iter().copied().collect();
    let mesh = rubber_band_frontier_mesh(&points)?;
    let domain = mesh.elements_of_domain();
    if domain.extent() < 1 {
        return Ok(false);
    }
    let (array, allocate) = rubber_band_triangle_cache(object, domain.extent() * 3)?;
    let mut vertex_index = 1;
    for id in domain.iter() {
        rubber_band_triangle_vertices(&mesh, id, &array, allocate, &mut vertex_index)?;
    }
    Ok(true)
}
fn rubber_band_triangle_cache<O: AisRubberBandSource + ?Sized>(
    object: &O,
    count: i32,
) -> Result<(Handle<Graphic3dArrayOfTriangles>, bool), Failure> {
    let mut cache = object.rubber_band_base().fields.triangles.write();
    let allocate = cache.is_null() || cache.vertex_number() != count;
    if allocate {
        *cache = Handle::new(Graphic3dArrayOfTriangles::with_options(
            count, 0, true, false, false,
        )?);
    }
    Ok((cache.clone(), allocate))
}
fn rubber_band_frontier_mesh(
    points: &[Vec2<i32>],
) -> Result<Handle<BRepMeshDataStructureOfDelaun>, Failure> {
    let allocator = Handle::new(IncrementalAllocatorObject::new(512 * 7));
    let mesh = Handle::new(BRepMeshDataStructureOfDelaun::new(&allocator, 100));
    let mut indices = Vec::with_capacity(points.len());
    for (index, point) in points.iter().enumerate() {
        let vertex = BRepMeshVertex::new(
            Xy::new(f64::from(point.x), f64::from(point.y)),
            index as i32 + 1,
            DegreeOfFreedom::Frontier,
        );
        indices.push(mesh.add_node(&vertex, false)?);
    }
    rubber_band_frontier_links(&mesh, points, &indices)?;
    BRepMeshDelaun::with_indices(&mesh, &mut indices, (-1, -1))?;
    Ok(mesh)
}
fn rubber_band_frontier_links(
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
    points: &[Vec2<i32>],
    indices: &[i32],
) -> Result<(), Failure> {
    let clockwise = rubber_band_orientation_sum(points) < 0.;
    for index in 0..indices.len() {
        let next = (index + 1) % indices.len();
        let (first, last) = if clockwise {
            (index, next)
        } else {
            (next, index)
        };
        mesh.add_link(&BRepMeshEdge::new(
            indices[first],
            indices[last],
            DegreeOfFreedom::Frontier,
        ))?;
    }
    Ok(())
}
fn rubber_band_orientation_sum(points: &[Vec2<i32>]) -> f64 {
    let mut sum = 0.;
    for (index, current) in points.iter().enumerate() {
        let next = points[(index + 1) % points.len()];
        // Preserve native integer arithmetic before conversion, including host two's-complement wrap.
        sum +=
            f64::from(next.x.wrapping_sub(current.x)) * f64::from(next.y.wrapping_add(current.y));
    }
    sum
}
fn rubber_band_triangle_vertices(
    mesh: &Handle<BRepMeshDataStructureOfDelaun>,
    id: i32,
    array: &Handle<Graphic3dArrayOfTriangles>,
    allocate: bool,
    vertex_index: &mut i32,
) -> Result<(), Failure> {
    let triangle = mesh.get_element(id)?;
    if triangle.movability() == DegreeOfFreedom::Deleted {
        return Ok(());
    }
    for node_id in mesh.element_nodes(&triangle)? {
        let vertex = mesh.get_node(node_id)?;
        let uv = vertex.coord();
        if allocate {
            array.add_vertex_normal_f64([uv.x(), uv.y(), 0.], [0., 0., 1.])?;
        } else {
            array.set_vertice(*vertex_index, uv.x() as f32, uv.y() as f32, 0.)?;
            *vertex_index += 1;
        }
    }
    Ok(())
}
