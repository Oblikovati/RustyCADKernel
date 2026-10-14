// SPDX-License-Identifier: GPL-2.0-only
//! Only opt-in observations read actual vertex coordinates while their shape still exists.
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;
pub(crate) type ObservedVertexPoint = Option<Result<[f64; 3], Arc<Failure>>>;

pub(crate) fn snapshot_vertex_point(shape: &Shape, known_kind: ShapeEnum) -> ObservedVertexPoint {
    if known_kind != ShapeEnum::Vertex {
        return None;
    }
    Some(actual_vertex_point(shape).map_err(Arc::new))
}
fn actual_vertex_point(shape: &Shape) -> Result<[f64; 3], Failure> {
    let actual = vertex(shape)?;
    let point = BRepTool::pnt(&actual)?;
    Ok([point.x(), point.y(), point.z()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::gp::Point3;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex;

    #[test]
    fn actual_vertex_values_are_read_while_nonvertex_kind_never_queries_null_shape() {
        let actual = BRepBuilderAPIMakeVertex::new(Point3::new(1., 2., 3.))
            .unwrap()
            .vertex()
            .unwrap();
        let shape = actual.into_shape();
        assert_eq!(
            snapshot_vertex_point(&shape, ShapeEnum::Vertex)
                .unwrap()
                .unwrap(),
            [1., 2., 3.]
        );
        assert!(snapshot_vertex_point(&Shape::default(), ShapeEnum::Edge).is_none());
    }

    #[test]
    fn typed_vertex_failure_remains_inside_the_observation_result() {
        let raw = actual_vertex_point(&Shape::default()).unwrap_err();
        let retained = snapshot_vertex_point(&Shape::default(), ShapeEnum::Vertex)
            .unwrap()
            .unwrap_err();
        assert_eq!(retained.exception_type(), raw.exception_type());
        assert_eq!(retained.message(), raw.message());
    }
}
