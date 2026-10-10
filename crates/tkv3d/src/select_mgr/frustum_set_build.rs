//! Native frontier insertion and Delaunay iteration define the child-frustum order.
use super::*;
use rustycad_tkernel::{
    ncollection::{Array1, IncrementalAllocatorObject},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDataStructureOfDelaun, BRepMeshDelaun, BRepMeshEdge, BRepMeshVertex, DegreeOfFreedom,
};
impl SelectMgrTriangularFrustumSet {
    pub(super) fn frustum_set_build(&self) -> Result<(), Failure> {
        let points = self.selection_polyline.read().points.clone();
        if self.get_selection_type() != SelectMgrSelectionType::Polyline && points.is_null() {
            self.frustum_set_initialized("Build")?;
        }
        self.frustums.write().clear();
        let points = points
            .get()
            .ok_or_else(frustum_set_missing_points)?
            .array1();
        let allocator = Handle::new(IncrementalAllocatorObject::new(512 * 7));
        let mesh = Handle::new(BRepMeshDataStructureOfDelaun::new(&allocator, 100));
        let mut indices = self.frustum_set_insert_nodes(&points, &mesh)?;
        frustum_set_frontier(&points, &indices, &mesh)?;
        let _triangulator = BRepMeshDelaun::with_indices(&mesh, &mut indices, (-1, -1))?;
        self.frustum_set_children(&mesh)
    }
    fn frustum_set_insert_nodes(
        &self,
        points: &Array1<'_, Point2>,
        mesh: &BRepMeshDataStructureOfDelaun,
    ) -> Result<Vec<i32>, Failure> {
        let builder = self.base.builder.read().clone();
        let builder = builder.get().ok_or_else(frustum_set_missing_builder)?;
        let mut boundary = self.boundary_points.write();
        boundary
            .resize_bounds(
                points.lower(),
                points.lower() + 2 * points.len() as i32 - 1,
                false,
            )
            .map_err(|e| e.into_failure("TriangularFrustumSet::Build boundary"))?;
        frustum_set_project_nodes(points, mesh, builder, &mut boundary)
    }
    fn frustum_set_children(&self, mesh: &BRepMeshDataStructureOfDelaun) -> Result<(), Failure> {
        for index in mesh.elements_of_domain().iter() {
            let triangle = mesh.get_element(index)?;
            if triangle.movability() == DegreeOfFreedom::Deleted {
                continue;
            }
            let nodes = mesh.element_nodes(&triangle)?;
            let points = [
                Point2::from(*mesh.get_node(nodes[0])?.coord()),
                Point2::from(*mesh.get_node(nodes[1])?.coord()),
                Point2::from(*mesh.get_node(nodes[2])?.coord()),
            ];
            let child = Handle::new(SelectMgrTriangularFrustum::new());
            child.init(points[0], points[1], points[2]);
            child.set_builder(&self.base.builder.read());
            child.build()?;
            self.frustums.write().push(child);
        }
        Ok(())
    }
}
fn frustum_set_project_boundary(
    boundary: &mut Array1<'_, Point3>,
    index: i32,
    count: i32,
    point: Point2,
    builder: &SelectMgrFrustumBuilder,
) -> Result<(), Failure> {
    boundary
        .set_value(
            index,
            builder.project_pnt_on_view_plane(point.x(), point.y(), 0.),
        )
        .map_err(|e| e.into_failure("frustum boundary near"))?;
    boundary
        .set_value(
            index + count,
            builder.project_pnt_on_view_plane(point.x(), point.y(), 1.),
        )
        .map_err(|e| e.into_failure("frustum boundary far"))
}
fn frustum_set_clockwise(points: &Array1<'_, Point2>) -> Result<bool, Failure> {
    let mut sum = 0.;
    for index in points.lower()..=points.upper() {
        // Native modulo assumes one-based indices; retain failure for incompatible bounds.
        let next = points
            .value(index % points.len() as i32 + 1)
            .map_err(|e| e.into_failure("TriangularFrustumSet winding next"))?;
        let point = points
            .value(index)
            .map_err(|e| e.into_failure("TriangularFrustumSet winding point"))?;
        sum += (next.x() - point.x()) * (next.y() + point.y());
    }
    Ok(sum < 0.)
}
fn frustum_set_frontier(
    points: &Array1<'_, Point2>,
    indices: &[i32],
    mesh: &BRepMeshDataStructureOfDelaun,
) -> Result<(), Failure> {
    let clockwise = frustum_set_clockwise(points)?;
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

fn frustum_set_missing_points() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "TriangularFrustumSet points=null; expected polyline pixels",
    )
}
fn frustum_set_project_nodes(
    points: &Array1<'_, Point2>,
    mesh: &BRepMeshDataStructureOfDelaun,
    builder: &SelectMgrFrustumBuilder,
    boundary: &mut Array1<'_, Point3>,
) -> Result<Vec<i32>, Failure> {
    let mut indices = Vec::with_capacity(points.len());
    for index in points.lower()..=points.upper() {
        let point = points
            .value(index)
            .map_err(|e| e.into_failure("TriangularFrustumSet::Build point"))?;
        indices.push(mesh.add_node(
            &BRepMeshVertex::new(point.xy(), index, DegreeOfFreedom::Frontier),
            false,
        )?);
        frustum_set_project_boundary(boundary, index, points.len() as i32, *point, builder)?;
    }
    Ok(indices)
}

fn frustum_set_missing_builder() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "TriangularFrustumSet builder=null; expected a projection builder",
    )
}
