//! HLRBRep_CurveTool and HLRBRep_CInter on projected edges.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkhlr::hlr_algo::{Projector, init_min_max};
use rustycad_tkhlr::hlr_brep::{CInter, Curve, CurveTool, TheProjPCurOfCInter};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction3, Point2, Point3};
use std::sync::Arc;

fn line(origin: Point3, direction: Direction3) -> Curve {
    let geometry: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(origin, direction)).cast();
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder
        .make_edge_with_curve(&mut edge, &geometry, 1e-7)
        .unwrap();
    builder.range(&edge, -2., 2., false).unwrap();
    let mut c = Curve::new();
    c.set_projector(Some(Arc::new(Projector::new().unwrap())));
    c.set_curve(&edge).unwrap();
    let (mut mn, mut mx) = ([0.; 16], [0.; 16]);
    init_min_max(1e100, &mut mn, &mut mx);
    c.update(&mut mn, &mut mx).unwrap();
    c
}

#[test]
fn a_projected_line_samples_twice_and_has_no_spline_handles() {
    let c = line(
        Point3::new(0., 0., 0.),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    assert_eq!(CurveTool::the_type(&c), CurveType::Line);
    assert_eq!(CurveTool::nb_samples(&c).unwrap(), 2);
    assert_eq!(CurveTool::nb_samples_in(&c, -2., 2.).unwrap(), 2);
    assert_eq!(CurveTool::eps_x(&c), 1e-10);
    assert!(CurveTool::bezier(&c).is_null() && CurveTool::bspline(&c).is_null());
}

#[test]
fn get_interval_outside_the_table_raises_out_of_range() {
    let c = line(
        Point3::new(0., 0., 0.),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let tab = Array1::with_bounds(1, 2).unwrap();
    let error = CurveTool::get_interval(&c, 2, &tab).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

#[test]
fn two_crossing_projected_lines_meet_once() {
    let a = line(
        Point3::new(0., 0., 1.),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let b = line(
        Point3::new(0., 0., -1.),
        Direction3::new(0., 1., 0.).unwrap(),
    );
    let mut inter = CInter::new();
    inter.perform_between(&a, &b, 1e-9, 1e-7).unwrap();
    assert!(inter.is_done());
    assert_eq!(inter.nb_points().unwrap(), 1);
    let p = inter.point(1).unwrap();
    assert!(p.param_on_first().abs() < 1e-12 && p.param_on_second().abs() < 1e-12);
}

#[test]
fn a_point_projects_on_the_nearest_parameter() {
    let c = line(
        Point3::new(0., 0., 0.),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let u = TheProjPCurOfCInter::find_parameter(&c, Point2::new(0.5, 3.), 1e-7).unwrap();
    assert!((u - 0.5).abs() < 1e-9);
}

#[test]
fn compute_domain_reads_only_the_type_of_a_point_like_line() {
    // A line along the sight projects to a point: building its 2D line raises, but
    // the domain, as natively, reads only the type.
    let c = line(
        Point3::new(1., 1., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
    );
    assert!(c.line().is_err());
    let inter = CInter::new();
    assert!(inter.compute_domain(&c, 1e-7).unwrap().has_first_point());
}
