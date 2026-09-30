//! Unit tests of the BRepGProp classes and package functions: closed-form properties of
//! simple shapes and the native behaviours docs/tktopalgo-progress.md lists (the probe
//! compares everything with OCCT bit for bit).
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
mod gprop_records;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;
#[allow(dead_code)]
#[path = "../examples/tp_probe/shape.rs"]
mod shape;
#[allow(dead_code)]
#[path = "../examples/tp_probe/shape_geometry.rs"]
mod shape_geometry;
mod solid_records;

use gprop_records::{
    BOX_1, CYLINDER_3_7, HOLED_PLATE, OPEN_BOX, RING, SHARED_MIRRORED, SHARED_MOVED,
    SHARED_REVERSED, SPLINE_PATCH,
};
use print::Reader;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeombase::gprop::{GProps, ValueType};
use rustycad_tkmath::geom_abs::{IsoType, Shape as Continuity};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tkmath::math::Function;
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_gprop::{
    BRepGProp, BRepGPropCinert, BRepGPropDomain, BRepGPropEdgeTool, BRepGPropFace, BRepGPropGauss,
    BRepGPropMeshCinert, BRepGPropMeshProps, BRepGPropSinert, BRepGPropTFunction,
    BRepGPropUFunction, BRepGPropVinert, BRepGPropVinertGK, GPropsSums, GaussProps, GaussType,
    MeshObjType, VinertGKOptions, VolumeGKFlags,
};
use shape::{TpShape, tp_shape};
use solid_records::{BOX10, HOLLOW10};
use std::f64::consts::PI;

fn read(record: &[f64]) -> TpShape {
    tp_shape(&mut Reader::new(record)).expect("shape record")
}

fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

fn volume(shape: &Shape) -> GProps {
    let mut props = GProps::new();
    BRepGProp::volume_properties(shape, &mut props, false, false, false).unwrap();
    props
}

#[test]
fn domain_visits_the_edges_then_fails_past_the_end() {
    let plate = read(&HOLED_PLATE);
    let mut domain = BRepGPropDomain::with_face(&plate.faces[0]).unwrap();
    let mut count = 0;
    while domain.more() {
        assert!(!domain.value().unwrap().is_null());
        count += 1;
        domain.next().unwrap();
    }
    assert_eq!(count, 5);
    let failure = domain.value().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_TypeMismatch");
    domain.init().unwrap();
    assert!(domain.more());
    assert!(BRepGPropDomain::new().value().unwrap().is_null());
}

#[test]
fn edge_tool_orders_follow_the_curve_type() {
    let solid = read(&CYLINDER_3_7);
    let circle = BRepAdaptorCurve::with_edge(&solid.edges[0]).unwrap();
    let line = BRepAdaptorCurve::with_edge(&solid.edges[2]).unwrap();
    assert_eq!(BRepGPropEdgeTool::integration_order(&circle).unwrap(), 10);
    assert_eq!(BRepGPropEdgeTool::integration_order(&line).unwrap(), 2);
    assert_eq!(
        BRepGPropEdgeTool::nb_intervals(&line, Continuity::CN).unwrap(),
        1
    );
    let cinert = BRepGPropCinert::with_curve(&circle, Point3::new(0.0, 0.0, 0.0)).unwrap();
    assert!(near(cinert.mass(), 6.0 * PI, 1e-12));
}

#[test]
fn face_orders_of_a_plane_and_a_cylinder() {
    let solid = read(&CYLINDER_3_7);
    let lateral = BRepGPropFace::with_face(&solid.faces[0], false).unwrap();
    assert_eq!(lateral.u_integration_order().unwrap(), 18);
    assert_eq!(lateral.su_int_subs().unwrap(), 3);
    assert_eq!(lateral.sv_int_subs().unwrap(), 1);
    let floor = BRepGPropFace::with_face(&solid.faces[1], false).unwrap();
    assert_eq!(floor.u_integration_order().unwrap(), 8);
    let mut short = Array1::with_bounds(1, 3).unwrap();
    let failure = lateral.u_knots(&mut short).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}

#[test]
fn plane_v_knots_are_its_u_bounds() {
    let plate = read(&HOLED_PLATE);
    let tool = BRepGPropFace::with_face(&plate.faces[0], false).unwrap();
    let mut knots = Array1::with_bounds(1, 2).unwrap();
    tool.v_knots(&mut knots).unwrap();
    let (u1, u2, _, _) = tool.bounds().unwrap();
    assert_eq!((knots[1], knots[2]), (u1, u2));
}

#[test]
fn spans_split_the_knots_only_with_the_span_flag() {
    let patch = read(&SPLINE_PATCH);
    let plain = BRepGPropFace::with_face(&patch.faces[0], false).unwrap();
    assert_eq!(plain.get_u_knots(0.0, 2.0).unwrap().length(), 2);
    let mut spans = BRepGPropFace::with_face(&patch.faces[0], true).unwrap();
    let knots = spans.get_u_knots(0.0, 2.0).unwrap();
    assert_eq!(
        knots.iter().copied().collect::<Vec<_>>(),
        vec![0.0, 1.0, 2.0]
    );
    spans.load_iso(true, IsoType::IsoV).unwrap();
    assert_eq!(spans.get_t_knots(0.0, 2.0).unwrap().length(), 2);
}

#[test]
fn gauss_checks_the_type_of_each_overload() {
    let plate = read(&HOLED_PLATE);
    let tool = BRepGPropFace::with_face(&plate.faces[0], false).unwrap();
    let mut out = GaussProps::default();
    let mut gauss = BRepGPropGauss::new(GaussType::Vinert);
    let failure = gauss
        .compute_natural_surface(&tool, Point3::new(0.0, 0.0, 0.0), &mut out)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert_eq!(out, GaussProps::default());
}

#[test]
fn sinert_on_the_domain_of_a_holed_plate() {
    let plate = read(&HOLED_PLATE);
    let mut tool = BRepGPropFace::with_face(&plate.faces[0], false).unwrap();
    let mut domain = BRepGPropDomain::with_face(&plate.faces[0]).unwrap();
    let mut props =
        BRepGPropSinert::with_domain(&mut tool, &mut domain, Point3::new(0.0, 0.0, 0.0)).unwrap();
    assert!(near(props.mass(), 16.0 - PI, 1e-9));
    assert_eq!(props.get_epsilon(), 1.0);
    let error = props.perform_eps(&mut tool, 1e-6).unwrap();
    // An empty domain on a face with wires integrates nothing (native Perform(S, Eps)).
    assert_eq!(props.mass(), 0.0);
    assert!(error >= 0.0);
}

#[test]
fn vinert_by_plane_keeps_no_mass() {
    let solid = read(&BOX_1);
    let mut tool = BRepGPropFace::with_face(&solid.faces[5], false).unwrap();
    let mut domain = BRepGPropDomain::with_face(&solid.faces[5]).unwrap();
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let props = BRepGPropVinert::with_domain_plane(
        &mut tool,
        &mut domain,
        plane,
        Point3::new(0.0, 0.0, 0.0),
    )
    .unwrap();
    assert_eq!(props.mass(), 0.0);
    let m = props.matrix_of_inertia();
    assert!(m.value(3, 3).unwrap() > 0.0);
}

#[test]
fn vinert_gk_reports_the_volume_under_a_face() {
    let solid = read(&BOX_1);
    let mut tool = BRepGPropFace::with_face(&solid.faces[5], false).unwrap();
    let mut domain = BRepGPropDomain::with_face(&solid.faces[5]).unwrap();
    let options = VinertGKOptions {
        tolerance: 1e-6,
        cg_flag: true,
        i_flag: true,
    };
    let props =
        BRepGPropVinertGK::with_domain(&mut tool, &mut domain, Point3::new(0.0, 0.0, 0.0), options)
            .unwrap();
    assert!(near(props.mass(), 1.0 / 3.0, 1e-9));
    assert!(props.get_error_reached() >= 0.0);
    let mut empty = BRepGPropVinertGK::new();
    assert_eq!(
        empty
            .perform_domain(&mut tool, &mut BRepGPropDomain::new(), options)
            .unwrap(),
        -1.0
    );
}

#[test]
fn integrand_functions_reject_the_unknown_type() {
    let solid = read(&BOX_1);
    let mut tool = BRepGPropFace::with_face(&solid.faces[5], false).unwrap();
    tool.load_iso(false, IsoType::IsoU).unwrap();
    let mut u = BRepGPropUFunction::new(&tool, Point3::new(0.0, 0.0, 0.0), true, &[0.0; 3]);
    let mut value = 0.0;
    assert!(!u.value(0.5, &mut value));
    u.set_value_type(ValueType::Mass);
    u.set_v_param(0.5);
    assert!(u.value(0.5, &mut value) && value > 0.0);
    let mut t = BRepGPropTFunction::new(
        &tool,
        Point3::new(0.0, 0.0, 0.0),
        true,
        &[0.0; 3],
        0.0,
        1e-9,
    );
    t.set_value_type(ValueType::Mass);
    assert!(t.value(0.5, &mut value));
    assert_eq!(t.state_number(), 0);
}

#[test]
fn mesh_cinert_and_calculate_props() {
    let mut nodes = Array1::with_bounds(1, 3).unwrap();
    nodes[1] = Point3::new(0.0, 0.0, 0.0);
    nodes[2] = Point3::new(3.0, 4.0, 0.0);
    nodes[3] = Point3::new(3.0, 4.0, 0.0);
    let mut polyline = BRepGPropMeshCinert::new();
    polyline.perform(&nodes).unwrap();
    assert!(near(polyline.mass(), 5.0, 1e-12));
    let shifted = Array1::with_bounds(0, 2).map(|mut a: Array1<'static, Point3>| {
        a.init(Point3::new(1.0, 0.0, 0.0));
        a
    });
    assert!(polyline.perform(&shifted.unwrap()).is_err());
    let mut sums: GPropsSums = [0.0; 10];
    let triangle = (
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(0.0, 3.0, 0.0),
    );
    BRepGPropMeshProps::calculate_props(
        triangle,
        Point3::new(0.0, 0.0, 0.0),
        false,
        &mut sums,
        1,
        &[1.0 / 3.0, 1.0 / 3.0, 0.5],
    )
    .unwrap();
    assert!(near(sums[0], 3.0, 1e-12));
}

/// A mesh of the unit square at z = 1, two triangles.
fn unit_square_mesh() -> Handle<Triangulation> {
    let mut nodes = Array1::with_bounds(1, 4).unwrap();
    for (i, (x, y)) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        nodes[i as i32 + 1] = Point3::new(x, y, 1.0);
    }
    let mut triangles = Array1::with_bounds(1, 2).unwrap();
    triangles[1] = Triangle::new(1, 2, 3);
    triangles[2] = Triangle::new(1, 3, 4);
    Handle::new(Triangulation::from_nodes(&nodes, &triangles).unwrap())
}

#[test]
fn mesh_props_with_rigid_and_scaled_locations() {
    let mesh = unit_square_mesh();
    let mut surface = BRepGPropMeshProps::new(MeshObjType::Sinert);
    surface
        .perform(&mesh, rustycad_tkg3d::top_abs::Orientation::Forward)
        .unwrap();
    assert!(near(surface.mass(), 1.0, 1e-12));
    let mut t = rustycad_tkmath::gp::Transform3::default();
    t.set_scale(Point3::new(0.0, 0.0, 0.0), 2.0).unwrap();
    let mut scaled = BRepGPropMeshProps::new(MeshObjType::Vinert);
    scaled
        .perform_located(
            &mesh,
            &Location::new(t),
            rustycad_tkg3d::top_abs::Orientation::Forward,
        )
        .unwrap();
    // The cone from the origin to the square scaled by 2 (area 4 at z = 2): 4 * 2 / 3.
    assert!(near(scaled.mass(), 8.0 / 3.0, 1e-12));
    let mut empty = BRepGPropMeshProps::new(MeshObjType::Vinert);
    empty
        .perform(
            &Handle::default(),
            rustycad_tkg3d::top_abs::Orientation::Forward,
        )
        .unwrap();
    assert_eq!(empty.mass(), 0.0);
}

#[test]
fn volume_of_multi_shell_and_holed_solids() {
    assert!(near(
        volume(&read(&HOLLOW10).shape).mass(),
        1000.0 - 64.0,
        1e-9
    ));
    let ring = volume(&read(&RING).shape);
    assert!(near(ring.mass(), PI * (9.0 - 2.25) * 2.0, 1e-9));
    assert!(near(ring.centre_of_mass().z(), 1.0, 1e-9));
}

#[test]
fn shared_solids_are_moved_reversed_or_recomputed() {
    let moved = volume(&read(&SHARED_MOVED).shape);
    assert!(near(moved.mass(), 72.0, 1e-9));
    let reversed = volume(&read(&SHARED_REVERSED).shape);
    assert!(near(reversed.mass(), 24.0, 1e-9));
    // The mirror is not rigid: that instance is integrated face by face, its faces
    // turned inside out by the symmetry (-24).
    let mirrored = volume(&read(&SHARED_MIRRORED).shape);
    assert!(near(mirrored.mass(), 24.0, 1e-9), "{}", mirrored.mass());
    let mut skipped = GProps::new();
    BRepGProp::volume_properties(&read(&SHARED_MOVED).shape, &mut skipped, false, true, false)
        .unwrap();
    assert!(near(skipped.mass(), 48.0, 1e-9));
}

#[test]
fn only_closed_skips_open_shells() {
    let open = read(&OPEN_BOX).shape;
    let mut props = GProps::new();
    BRepGProp::volume_properties(&open, &mut props, true, false, false).unwrap();
    assert_eq!(props.mass(), 0.0);
    let error =
        BRepGProp::volume_properties_gk(&open, &mut props, 1e-3, true, VolumeGKFlags::default())
            .unwrap();
    assert_eq!(error, -1.0);
    let solid = read(&BOX10).shape;
    let flags = VolumeGKFlags {
        cg_flag: true,
        ..VolumeGKFlags::default()
    };
    BRepGProp::volume_properties_gk(&solid, &mut props, 1e-6, true, flags).unwrap();
    assert!(near(props.mass(), 1000.0, 1e-6));
    assert!(near(props.centre_of_mass().x(), 5.0, 1e-6));
}

#[test]
fn surface_and_linear_properties_of_a_ring() {
    let ring = read(&RING).shape;
    let mut area = GProps::new();
    let error = BRepGProp::surface_properties_eps(&ring, &mut area, 1e-8, false).unwrap();
    let expected = 2.0 * PI * 3.0 * 2.0 + 2.0 * PI * 1.5 * 2.0 + 2.0 * PI * (9.0 - 2.25);
    assert!(near(area.mass(), expected, 1e-7));
    assert!(error <= 1e-8);
    let mut length = GProps::new();
    BRepGProp::linear_properties(&ring, &mut length, true, false).unwrap();
    assert!(near(
        length.mass(),
        2.0 * (2.0 * PI * 3.0 + 2.0 * PI * 1.5) + 4.0,
        1e-9
    ));
}

#[test]
fn a_face_without_wires_uses_its_natural_bounds() {
    let f: Face = read(&BOX_1).faces[0].clone();
    let tool = BRepGPropFace::with_face(&f, false).unwrap();
    assert!(!tool.natural_restriction().unwrap());
    let props = BRepGPropSinert::with_face(&tool, Point3::new(0.0, 0.0, 0.0)).unwrap();
    assert!(near(props.mass(), 1.0, 1e-12));
}
