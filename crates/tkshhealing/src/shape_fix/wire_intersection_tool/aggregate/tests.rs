#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::ShapeFixIntersectionTool;
    use crate::shape_analysis::ShapeAnalysisEdge;
    use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
    use crate::shape_extend::ShapeExtendWireData;
    use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
    use rustycad_tkbrep::topods::{Edge, Face};
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine};
    use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomPlane};
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tkmath::gp::{
        Axis1, Axis2, Axis2d, Axis3, Axis22d, Direction2, Direction3, Point2, Point3,
    };
    use rustycad_tkmath::toploc::Location;

    const EXPECTED: &str = include_str!(
        "../../../../../../tools/oracle/tkshhealing_intersection_tool_aggregate.expected"
    );

    #[test]
    fn interior_nonadjacent_line_crossing_splits_both_edges() {
        let face = plane();
        let wire = Handle::new(ShapeExtendWireData::new());
        for edge in crossing_edges(&face) {
            wire.add_edge(&edge, 0).unwrap();
        }
        let context = Handle::new(ShapeBuildReShape::new());
        let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);
        let (mut split, mut cut, mut removed) = (91, 92, 93);
        let done = tool
            .fix_self_intersect_wire(&wire, &face, &mut split, &mut cut, &mut removed)
            .unwrap();
        let mut observed = vec![
            f64::from(done),
            f64::from(split),
            f64::from(cut),
            f64::from(removed),
            f64::from(wire.nb_edges()),
        ];
        append_ranges(&wire, &face, &mut observed);
        let expected = EXPECTED
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect::<Vec<f64>>();
        assert_eq!(observed, expected);
    }

    #[test]
    fn middle_to_endpoint_intersection_reuses_endpoint_vertex() {
        let mut face = plane();
        let wire = Handle::new(ShapeExtendWireData::new());
        for edge in middle_to_endpoint_edges(&face) {
            wire.add_edge(&edge, 0).unwrap();
        }
        BRepBuilder::default()
            .add(&mut face, &wire.wire().unwrap().into_shape())
            .unwrap();
        let context = Handle::new(ShapeBuildReShape::new());
        let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);
        let (mut split, mut cut, mut removed) = (91, 92, 93);
        let done = tool
            .fix_self_intersect_wire(&wire, &face, &mut split, &mut cut, &mut removed)
            .unwrap();
        let analysis = ShapeAnalysisEdge::new();
        let split_edge = wire.edge(1).unwrap();
        let endpoint_edge = wire.edge(4).unwrap();
        assert!(
            analysis
                .last_vertex(&split_edge)
                .is_same(&analysis.first_vertex(&endpoint_edge))
        );
        let mut observed = vec![
            f64::from(done),
            f64::from(split),
            f64::from(cut),
            f64::from(removed),
            f64::from(wire.nb_edges()),
        ];
        append_ranges(&wire, &face, &mut observed);
        let expected = EXPECTED
            .lines()
            .nth(1)
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect::<Vec<f64>>();
        assert_eq!(observed, expected);
    }

    #[test]
    fn endpoint_to_endpoint_intersection_unions_shared_vertex() {
        let mut face = plane();
        let wire = Handle::new(ShapeExtendWireData::new());
        for edge in endpoint_to_endpoint_edges(&face) {
            wire.add_edge(&edge, 0).unwrap();
        }
        BRepBuilder::default()
            .add(&mut face, &wire.wire().unwrap().into_shape())
            .unwrap();
        let context = Handle::new(ShapeBuildReShape::new());
        let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);
        let (mut split, mut cut, mut removed) = (91, 92, 93);
        let done = tool
            .fix_self_intersect_wire(&wire, &face, &mut split, &mut cut, &mut removed)
            .unwrap();
        let analysis = ShapeAnalysisEdge::new();
        let first_edge = wire.edge(1).unwrap();
        let third_edge = wire.edge(3).unwrap();
        assert!(
            analysis
                .last_vertex(&first_edge)
                .is_same(&analysis.first_vertex(&third_edge))
        );
        let mut observed = vec![
            f64::from(done),
            f64::from(split),
            f64::from(cut),
            f64::from(removed),
            f64::from(wire.nb_edges()),
        ];
        append_ranges(&wire, &face, &mut observed);
        let expected = EXPECTED
            .lines()
            .nth(2)
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect::<Vec<f64>>();
        assert_eq!(observed, expected);
    }

    #[test]
    fn reversed_nonadjacent_edge_crossing_preserves_native_ranges() {
        let mut face = plane();
        let wire = Handle::new(ShapeExtendWireData::new());
        let mut edges = crossing_edges(&face);
        edges[2].set_orientation(Orientation::Reversed);
        for edge in edges {
            wire.add_edge(&edge, 0).unwrap();
        }
        BRepBuilder::default()
            .add(&mut face, &wire.wire().unwrap().into_shape())
            .unwrap();
        let context = Handle::new(ShapeBuildReShape::new());
        let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);
        let (mut split, mut cut, mut removed) = (91, 92, 93);
        let done = tool
            .fix_self_intersect_wire(&wire, &face, &mut split, &mut cut, &mut removed)
            .unwrap();
        let mut observed = vec![
            f64::from(done),
            f64::from(split),
            f64::from(cut),
            f64::from(removed),
            f64::from(wire.nb_edges()),
        ];
        append_ranges(&wire, &face, &mut observed);
        let expected = EXPECTED
            .lines()
            .nth(3)
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect::<Vec<f64>>();
        assert_eq!(observed, expected);
    }

    #[test]
    fn two_point_intersections_are_repaired_in_native_selection_order() {
        let mut face = plane();
        let wire = Handle::new(ShapeExtendWireData::new());
        wire.add_edge(&circle_edge(&face), 0).unwrap();
        wire.add_edge(
            &line_edge(
                &face,
                Point3::new(3.0, 3.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(3.0, 3.0),
                Direction2::new(1.0, 0.0).unwrap(),
            ),
            0,
        )
        .unwrap();
        wire.add_edge(
            &line_edge_with_range(
                &face,
                Point3::new(-2.0, 0.5, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(-2.0, 0.5),
                Direction2::new(1.0, 0.0).unwrap(),
                4.0,
                1.0e-7,
            ),
            0,
        )
        .unwrap();
        wire.add_edge(
            &line_edge(
                &face,
                Point3::new(-4.0, -4.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
                Point2::new(-4.0, -4.0),
                Direction2::new(0.0, 1.0).unwrap(),
            ),
            0,
        )
        .unwrap();
        BRepBuilder::default()
            .add(&mut face, &wire.wire().unwrap().into_shape())
            .unwrap();
        let context = Handle::new(ShapeBuildReShape::new());
        let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);
        let (mut split, mut cut, mut removed) = (91, 92, 93);
        let done = tool
            .fix_self_intersect_wire(&wire, &face, &mut split, &mut cut, &mut removed)
            .unwrap();
        let mut observed = vec![
            f64::from(done),
            f64::from(split),
            f64::from(cut),
            f64::from(removed),
            f64::from(wire.nb_edges()),
        ];
        append_ranges(&wire, &face, &mut observed);
        let expected = EXPECTED
            .lines()
            .nth(4)
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect::<Vec<f64>>();
        assert_eq!(observed, expected);
    }

    #[test]
    fn overlap_starting_at_existing_endpoints_matches_native_replacement() {
        let mut face = plane();
        let wire = Handle::new(ShapeExtendWireData::new());
        for edge in endpoint_overlap_edges(&face) {
            wire.add_edge(&edge, 0).unwrap();
        }
        BRepBuilder::default()
            .add(&mut face, &wire.wire().unwrap().into_shape())
            .unwrap();
        let context = Handle::new(ShapeBuildReShape::new());
        let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);
        let (mut split, mut cut, mut removed) = (91, 92, 0);
        let done = tool
            .fix_self_intersect_wire(&wire, &face, &mut split, &mut cut, &mut removed)
            .unwrap();
        let mut observed = vec![
            f64::from(done),
            f64::from(split),
            f64::from(cut),
            f64::from(removed),
            f64::from(wire.nb_edges()),
        ];
        append_ranges(&wire, &face, &mut observed);
        let expected = EXPECTED
            .lines()
            .nth(5)
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect::<Vec<f64>>();
        assert_eq!(observed, expected);
    }

    #[test]
    fn long_overlap_triple_split_removes_overlapped_pieces() {
        let mut face = plane();
        let wire = Handle::new(ShapeExtendWireData::new());
        let edges = overlap_edges(&face, 0.4, 0.8);
        for edge in edges {
            wire.add_edge(&edge, 0).unwrap();
        }
        BRepBuilder::default()
            .add(&mut face, &wire.wire().unwrap().into_shape())
            .unwrap();
        let context = Handle::new(ShapeBuildReShape::new());
        let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);
        let (mut split, mut cut, mut removed) = (91, 92, 0);
        let done = tool
            .fix_self_intersect_wire(&wire, &face, &mut split, &mut cut, &mut removed)
            .unwrap();
        let mut observed = vec![
            f64::from(done),
            f64::from(split),
            f64::from(cut),
            f64::from(removed),
            f64::from(wire.nb_edges()),
        ];
        append_ranges(&wire, &face, &mut observed);
        let expected = EXPECTED
            .lines()
            .nth(6)
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect::<Vec<f64>>();
        assert_eq!(observed, expected);
    }

    #[test]
    fn short_overlap_splits_both_edges_without_removing_them() {
        let mut face = plane();
        let wire = Handle::new(ShapeExtendWireData::new());
        for edge in overlap_edges(&face, 0.1, 0.95) {
            wire.add_edge(&edge, 0).unwrap();
        }
        BRepBuilder::default()
            .add(&mut face, &wire.wire().unwrap().into_shape())
            .unwrap();
        let context = Handle::new(ShapeBuildReShape::new());
        let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);
        let (mut split, mut cut, mut removed) = (91, 92, 0);
        let done = tool
            .fix_self_intersect_wire(&wire, &face, &mut split, &mut cut, &mut removed)
            .unwrap();
        let mut observed = vec![
            f64::from(done),
            f64::from(split),
            f64::from(cut),
            f64::from(removed),
            f64::from(wire.nb_edges()),
        ];
        append_ranges(&wire, &face, &mut observed);
        let expected = EXPECTED
            .lines()
            .nth(7)
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect::<Vec<f64>>();
        assert_eq!(observed, expected);
    }

    fn plane() -> Face {
        let surface = Handle::new(GeomPlane::new(Axis3::default()));
        let mut face = Face::default();
        BRepBuilder::default()
            .make_face_on_surface(&mut face, &surface.cast(), 1.0e-7)
            .unwrap();
        face
    }

    fn crossing_edges(face: &Face) -> [Edge; 4] {
        [
            line_edge(
                face,
                Point3::new(-1.0, 0.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(-1.0, 0.0),
                Direction2::new(1.0, 0.0).unwrap(),
            ),
            line_edge(
                face,
                Point3::new(3.0, 3.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(3.0, 3.0),
                Direction2::new(1.0, 0.0).unwrap(),
            ),
            line_edge(
                face,
                Point3::new(0.0, -1.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
                Point2::new(0.0, -1.0),
                Direction2::new(0.0, 1.0).unwrap(),
            ),
            line_edge(
                face,
                Point3::new(-4.0, -4.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
                Point2::new(-4.0, -4.0),
                Direction2::new(0.0, 1.0).unwrap(),
            ),
        ]
    }

    fn middle_to_endpoint_edges(face: &Face) -> [Edge; 4] {
        [
            line_edge(
                face,
                Point3::new(-1.0, 0.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(-1.0, 0.0),
                Direction2::new(1.0, 0.0).unwrap(),
            ),
            line_edge(
                face,
                Point3::new(3.0, 3.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(3.0, 3.0),
                Direction2::new(1.0, 0.0).unwrap(),
            ),
            line_edge(
                face,
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
                Point2::new(0.0, 0.0),
                Direction2::new(0.0, 1.0).unwrap(),
            ),
            line_edge(
                face,
                Point3::new(-4.0, -4.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
                Point2::new(-4.0, -4.0),
                Direction2::new(0.0, 1.0).unwrap(),
            ),
        ]
    }

    fn endpoint_to_endpoint_edges(face: &Face) -> [Edge; 4] {
        [
            line_edge_with_range(
                face,
                Point3::new(-1.0, 0.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(-1.0, 0.0),
                Direction2::new(1.0, 0.0).unwrap(),
                1.0,
                1.0e-7,
            ),
            line_edge(
                face,
                Point3::new(3.0, 3.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(3.0, 3.0),
                Direction2::new(1.0, 0.0).unwrap(),
            ),
            line_edge_with_range(
                face,
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
                Point2::new(0.0, 0.0),
                Direction2::new(0.0, 1.0).unwrap(),
                1.0,
                1.0e-7,
            ),
            line_edge(
                face,
                Point3::new(-4.0, -4.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
                Point2::new(-4.0, -4.0),
                Direction2::new(0.0, 1.0).unwrap(),
            ),
        ]
    }

    fn line_edge(
        face: &Face,
        start: Point3,
        direction: Direction3,
        uv_start: Point2,
        uv_direction: Direction2,
    ) -> Edge {
        line_edge_with_range(face, start, direction, uv_start, uv_direction, 2.0, 1.0e-7)
    }

    fn overlap_edges(face: &Face, length: f64, start: f64) -> [Edge; 4] {
        [
            line_edge_with_range(
                face,
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(0.0, 0.0),
                Direction2::new(1.0, 0.0).unwrap(),
                2.0,
                0.1,
            ),
            line_edge(
                face,
                Point3::new(3.0, 3.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(3.0, 3.0),
                Direction2::new(1.0, 0.0).unwrap(),
            ),
            offset_overlap_edge(face, length, start),
            line_edge(
                face,
                Point3::new(-4.0, -4.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
                Point2::new(-4.0, -4.0),
                Direction2::new(0.0, 1.0).unwrap(),
            ),
        ]
    }

    fn endpoint_overlap_edges(face: &Face) -> [Edge; 4] {
        [
            line_edge_with_range(
                face,
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(0.0, 0.0),
                Direction2::new(1.0, 0.0).unwrap(),
                2.0,
                0.1,
            ),
            line_edge(
                face,
                Point3::new(3.0, 3.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(3.0, 3.0),
                Direction2::new(1.0, 0.0).unwrap(),
            ),
            line_edge_with_range(
                face,
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
                Point2::new(0.0, 0.0),
                Direction2::new(1.0, 0.0).unwrap(),
                1.5,
                0.1,
            ),
            line_edge(
                face,
                Point3::new(-4.0, -4.0, 0.0),
                Direction3::new(0.0, 1.0, 0.0).unwrap(),
                Point2::new(-4.0, -4.0),
                Direction2::new(0.0, 1.0).unwrap(),
            ),
        ]
    }

    fn line_edge_with_range(
        face: &Face,
        start: Point3,
        direction: Direction3,
        uv_start: Point2,
        uv_direction: Direction2,
        last: f64,
        vertex_tolerance: f64,
    ) -> Edge {
        let axis = Axis1::new(start, direction);
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
        let mut edge = Edge::default();
        ShapeBuildEdge
            .make_edge_on_curve_range(&mut edge, &curve, &Location::default(), 0.0, last)
            .unwrap();
        let analysis = ShapeAnalysisEdge::new();
        BRepBuilder::default()
            .update_vertex_tolerance(&analysis.first_vertex(&edge), vertex_tolerance)
            .unwrap();
        BRepBuilder::default()
            .update_vertex_tolerance(&analysis.last_vertex(&edge), vertex_tolerance)
            .unwrap();
        let uv_axis = Axis2d::new(uv_start, uv_direction);
        let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(uv_axis)).cast();
        let builder = BRepBuilder::default();
        builder
            .update_edge_pcurve_on_face(&edge, &pcurve, face, 1.0e-7)
            .unwrap();
        builder.range_on_face(&edge, face, 0.0, last).unwrap();
        builder.same_range(&edge, true).unwrap();
        builder.same_parameter(&edge, true).unwrap();
        edge
    }

    fn circle_edge(face: &Face) -> Edge {
        let curve: Handle<dyn GeomCurve> =
            Handle::new(GeomCircle::new(Axis2::default(), 1.0).unwrap()).cast();
        let mut edge = Edge::default();
        ShapeBuildEdge
            .make_edge_on_curve_range(
                &mut edge,
                &curve,
                &Location::default(),
                0.0,
                std::f64::consts::TAU,
            )
            .unwrap();
        let pcurve: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dCircle::new(Axis22d::default(), 1.0).unwrap()).cast();
        let builder = BRepBuilder::default();
        builder
            .update_edge_pcurve_on_face(&edge, &pcurve, face, 1.0e-7)
            .unwrap();
        builder
            .range_on_face(&edge, face, 0.0, std::f64::consts::TAU)
            .unwrap();
        builder.same_range(&edge, true).unwrap();
        builder.same_parameter(&edge, true).unwrap();
        edge
    }

    fn offset_overlap_edge(face: &Face, length: f64, start: f64) -> Edge {
        let edge = line_edge_with_range(
            face,
            Point3::new(start, 0.0, 0.0),
            Direction3::new(1.0, 0.0, 0.0).unwrap(),
            Point2::new(start, 0.0),
            Direction2::new(1.0, 0.0).unwrap(),
            length,
            0.1,
        );
        let analysis = ShapeAnalysisEdge::new();
        BRepBuilder::default()
            .update_vertex_point(
                &analysis.first_vertex(&edge),
                Point3::new(0.2, 0.0, 0.0),
                0.1,
            )
            .unwrap();
        BRepBuilder::default()
            .update_vertex_point(
                &analysis.last_vertex(&edge),
                Point3::new(1.8, 0.0, 0.0),
                0.1,
            )
            .unwrap();
        edge
    }

    fn append_ranges(wire: &ShapeExtendWireData, face: &Face, values: &mut Vec<f64>) {
        for index in 1..=wire.nb_edges() {
            let (first, last) = BRepTool::range_on_face(&wire.edge(index).unwrap(), face).unwrap();
            values.extend([first, last]);
        }
    }
}
