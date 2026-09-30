//! Focused native output-state and threshold regressions supplement OCC25547.
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkmath::{
    geom_abs::IsoType,
    gp::{Direction3, Point2, Point3, Xy},
};
use rustycad_tkmesh::brep_mesh::{
    ConeRangeSplitter, DefaultRangeSplitter, GeomTool, IntersectionFlag,
};
#[path = "support/geom_cells_fixture.rs"]
mod cells;
#[path = "support/curve_tessellator_fixture.rs"]
mod curves;
#[path = "support/range_splitter_fixture.rs"]
mod ranges;
#[path = "support/geom_intersection_fixture.rs"]
mod segments;
#[test]
fn parallel_lines_and_endpoint_touch_leave_outputs_untouched() {
    let (a, b) = segments::segment(2);
    let (c, d) = segments::segment(4);
    let mut xy = Xy::new(17.0, 19.0);
    let mut parameters = [23.0, 29.0];
    assert_eq!(
        GeomTool::int_lin_lin(a, b, c, d, &mut xy, &mut parameters),
        IntersectionFlag::Same
    );
    assert_eq!(xy, Xy::new(17.0, 19.0));
    assert_eq!(parameters, [23.0, 29.0]);
    let mut point = Point2::new(17.0, 19.0);
    assert_eq!(
        GeomTool::int_seg_seg(a, b, c, d, true, true, &mut point),
        IntersectionFlag::EndPointTouch
    );
    assert_eq!(point, Point2::new(17.0, 19.0));
}
#[test]
fn native_point_on_segment_reports_opposite_endpoint() {
    let (a, b) = segments::segment(2);
    let (c, d) = segments::segment(7);
    let mut point = Point2::default();
    assert_eq!(
        GeomTool::int_seg_seg(a, b, c, d, false, true, &mut point),
        IntersectionFlag::PointOnSegment
    );
    // OCCT 3d097a03 selects a rather than the geometrically incident c.
    assert_eq!(point, Point2::new(a.x(), a.y()));
}
#[test]
fn short_chord_deflection_uses_distance_to_first_endpoint() {
    let first = Point3::default();
    let last = Point3::new(precision::confusion(), 0.0, 0.0);
    let middle = Point3::new(2.0, 3.0, 0.0);
    assert_eq!(
        GeomTool::square_deflection_of_segment(first, last, middle).unwrap(),
        13.0
    );
    assert_eq!(
        GeomTool::square_deflection_of_segment(first, Point3::new(1.0, 0.0, 0.0), middle).unwrap(),
        9.0
    );
}
#[test]
fn invalid_index_and_iso_surface_query_do_not_write_outputs() {
    let face = ranges::range_face(1, 1e-7);
    let surface = face.get_surface();
    let mut tool = GeomTool::on_surface(
        &surface,
        IsoType::IsoU,
        0.5,
        (0.0, 1.0),
        (0.1, 0.5),
        2,
        1e-7,
    )
    .unwrap();
    assert_eq!(tool.nb_points(), 2);
    assert_eq!(
        tool.add_point(Point3::new(7.0, 8.0, 9.0), 0.5, true)
            .unwrap(),
        2
    );
    let mut parameter = 23.0;
    let mut point = Point3::new(11.0, 13.0, 17.0);
    let mut uv = Point2::new(17.0, 19.0);
    assert!(
        !tool
            .value_iso(0, 0.37, &mut parameter, &mut point, &mut uv)
            .unwrap()
    );
    assert!(
        !tool
            .value_surface(1, &surface, &mut parameter, &mut point, &mut uv)
            .unwrap()
    );
    assert_eq!(
        (parameter, point, uv),
        (23.0, Point3::new(11.0, 13.0, 17.0), Point2::new(17.0, 19.0))
    );
    assert!(
        tool.value_iso(2, 0.37, &mut parameter, &mut point, &mut uv)
            .unwrap()
    );
    assert_eq!(uv, Point2::new(0.37, 0.5));
    assert_eq!(point, Point3::new(7.0, 8.0, 9.0));
}
#[test]
fn edge_sample_reads_face_pcurve_and_normal_orientation() {
    let (edge, face) = curves::refinement_fixture(0, false, false);
    let adaptor = BRepAdaptorCurve::with_edge(&edge.get_edge().unwrap()).unwrap();
    let tool = GeomTool::new(&adaptor, (0.0, 1.0), 0.1, 0.5, 2, 1e-7).unwrap();
    let mut parameter = 23.0;
    let mut point = Point3::default();
    let mut uv = Point2::default();
    assert!(
        tool.value_surface(2, &face.get_surface(), &mut parameter, &mut point, &mut uv)
            .unwrap()
    );
    assert_eq!(uv, Point2::new(1.0, 0.5));
    let mut normal = Direction3::X;
    assert!(GeomTool::normal(&face.get_surface(), 0.0, 0.5, &mut point, &mut normal).unwrap());
    assert_eq!(normal, Direction3::X);
}
#[test]
fn grid_count_zero_deflection_uses_native_integer_overflow() {
    let face = ranges::range_face(0, 1e-7);
    let mut splitter = DefaultRangeSplitter::default();
    splitter.reset(&face, &Default::default());
    let b = ranges::range_bounds(0);
    splitter.add_point(Point2::new(b[0], b[2]));
    splitter.add_point(Point2::new(b[1], b[3]));
    splitter.adjust_range().unwrap();
    assert_eq!(
        GeomTool::cells_count(&face.get_surface().cast(), 0, 0.0, Some(&splitter)).unwrap(),
        (2, 2)
    );
    assert_eq!(
        GeomTool::cells_count(&Handle::default(), 0, 0.0, None).unwrap(),
        (-1, -1)
    );
}
#[test]
fn cone_split_counts_use_native_overflow_not_saturating_conversion() {
    let face = ranges::range_face(0, 1e-7);
    face.set_deflection(0.1);
    let mut splitter = ConeRangeSplitter::default();
    splitter.reset(&face, &Default::default());
    splitter.add_point(Point2::new(0.0, 0.0));
    splitter.add_point(Point2::new(1e100, 1.0));
    let mut counts = (0, 0);
    splitter
        .get_split_steps(
            &rustycad_tkmesh::imesh_tools::MeshParameters {
                min_size: 0.001,
                ..Default::default()
            },
            &mut counts,
        )
        .unwrap();
    assert_eq!(counts.0, i32::MIN);
}
#[test]
fn spline_and_swept_grid_estimates_keep_native_minimum() {
    let splitter = DefaultRangeSplitter {
        range_u: (0.0, 3.0),
        range_v: (-1.0, 2.0),
        delta: (0.25, 0.75),
        ..Default::default()
    };
    for kind in 0..12 {
        let surface = Handle::new(
            rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface::with_surface(&cells::cells_surface(
                kind,
            ))
            .unwrap(),
        )
        .cast();
        let counts = GeomTool::cells_count(&surface, 31, 0.01, Some(&splitter)).unwrap();
        assert!(counts.0 >= 2 && counts.1 >= 2);
    }
}
