//! Behavior tests of BRepGProp after the cases of the upstream suite
//! `BRepGProp_Test.cxx`, which is not mapped: its shapes come from BRepBuilderAPI
//! (MakeEdge, MakeWire, MakeFace), BRepPrimAPI (MakeBox, MakeSphere, MakeCylinder) and
//! BRepAlgoAPI_Cut (TKBO), which the port does not provide, and TKBO is not in the shared
//! native build (the unchanged suite does not link there). Here the same shapes are
//! built with BRep_Builder (the solids by the probe's shape reader, records in
//! `tests/gprop_records`; edges and faces as MakeEdge and MakeFace give them), and each
//! case checks the same values with the same tolerances. `OCC49_CutShapeHasNoSymmetryAxis`
//! needs the boolean cut and has no counterpart.
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
mod gprop_builders;
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

use gprop_builders::{edge_on, plane_rectangle, quarter_disk, segment, wide_spline_surface};
use gprop_records::{BOX_1, BOX_10_20_30, CYLINDER_3_7, CYLINDER_10_20, SPHERE_5};
use print::Reader;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Direction3, Plane3, Point2, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_gprop::{
    BRepGProp, BRepGPropDomain, BRepGPropFace, BRepGPropSinert, BRepGPropVinert,
};
use shape::tp_shape;
use solid_records::BOX10;
use std::f64::consts::PI;

fn shape(record: &[f64]) -> Shape {
    tp_shape(&mut Reader::new(record))
        .expect("shape record")
        .shape
}

fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

/// `expectGPropsNear`: mass, centre and matrix within `tol`.
fn assert_gprops_near(actual: &GProps, expected: &GProps, tol: f64) {
    assert!(near(actual.mass(), expected.mass(), tol));
    let (a, e) = (actual.centre_of_mass(), expected.centre_of_mass());
    assert!(near(a.x(), e.x(), tol) && near(a.y(), e.y(), tol) && near(a.z(), e.z(), tol));
    let (ma, me) = (actual.matrix_of_inertia(), expected.matrix_of_inertia());
    for row in 1..=3 {
        for column in 1..=3 {
            let (x, y) = (
                ma.value(row, column).unwrap(),
                me.value(row, column).unwrap(),
            );
            assert!(near(x, y, tol), "inertia ({row}, {column}): {x} vs {y}");
        }
    }
}

// BRepGPropTest.LinearProperties_EdgeLength
#[test]
fn linear_properties_edge_length() {
    let e = segment(Point3::new(0.0, 0.0, 0.0), Point3::new(3.0, 4.0, 0.0));
    let mut props = GProps::new();
    BRepGProp::linear_properties(&e, &mut props, false, false).unwrap();
    assert!(near(props.mass(), 5.0, precision::confusion()));
}

// BRepGPropTest.SurfaceProperties_BoxFaceArea
#[test]
fn surface_properties_box_face_area() {
    let mut props = GProps::new();
    BRepGProp::surface_properties(&shape(&BOX_10_20_30), &mut props, false, false).unwrap();
    assert!(near(props.mass(), 2200.0, precision::confusion()));
}

// BRepGPropTest.VolumeProperties_UnitBox
#[test]
fn volume_properties_unit_box() {
    let mut props = GProps::new();
    BRepGProp::volume_properties(&shape(&BOX_1), &mut props, false, false, false).unwrap();
    assert!(near(props.mass(), 1.0, precision::confusion()));
}

// BRepGPropTest.VolumeProperties_Sphere
#[test]
fn volume_properties_sphere() {
    let mut props = GProps::new();
    BRepGProp::volume_properties(&shape(&SPHERE_5), &mut props, false, false, false).unwrap();
    assert!(near(props.mass(), (4.0 / 3.0) * PI * 125.0, 0.01));
}

// BRepGPropTest.VolumeProperties_BoxCenterOfMass
#[test]
fn volume_properties_box_center_of_mass() {
    let mut props = GProps::new();
    BRepGProp::volume_properties(&shape(&BOX10), &mut props, false, false, false).unwrap();
    let com = props.centre_of_mass();
    for c in [com.x(), com.y(), com.z()] {
        assert!(near(c, 5.0, precision::confusion()));
    }
}

// BRepGPropTest.AdaptiveSurfaceProperties_Box
#[test]
fn adaptive_surface_properties_box() {
    let mut props = GProps::new();
    BRepGProp::surface_properties_eps(&shape(&BOX_10_20_30), &mut props, 1.0e-6, false).unwrap();
    assert!(near(props.mass(), 2200.0, 1.0e-9));
}

// BRepGPropTest.AdaptiveVolumeProperties_Box
#[test]
fn adaptive_volume_properties_box() {
    let mut props = GProps::new();
    BRepGProp::volume_properties_eps(&shape(&BOX_10_20_30), &mut props, 1.0e-6, false, false)
        .unwrap();
    assert!(near(props.mass(), 6000.0, 1.0e-9));
}

// BRepGPropTest.AdaptiveNaturalBSplineSurface_ResizesIntervalArrays
#[test]
fn adaptive_natural_bspline_surface_resizes_interval_arrays() {
    let intervals = rustycad_tkmath::math::gauss_points_max() + 1;
    let mut f = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(
            &mut f,
            &wide_spline_surface(intervals),
            precision::confusion(),
        )
        .unwrap();
    assert_eq!(f.nb_children(), 0);
    let fixed_face = BRepGPropFace::with_face(&f, false).unwrap();
    assert_eq!(fixed_face.su_int_subs().unwrap(), intervals);
    let fixed = BRepGPropSinert::with_face(&fixed_face, Point3::new(0.0, 0.0, 0.0)).unwrap();
    let mut adaptive_face = BRepGPropFace::with_face(&f, false).unwrap();
    let adaptive =
        BRepGPropSinert::with_face_eps(&mut adaptive_face, Point3::new(0.0, 0.0, 0.0), 1.0e-8)
            .unwrap();
    let n = f64::from(intervals);
    assert!(near(adaptive.mass(), 2.0 * n, 1.0e-10));
    assert!(near(adaptive.centre_of_mass().x(), 0.5 * n, 1.0e-10));
    assert!(near(adaptive.centre_of_mass().y(), 1.0, 1.0e-12));
    assert!(near(adaptive.centre_of_mass().z(), 0.0, 1.0e-12));
    assert_gprops_near(&adaptive, &fixed, 1.0e-9);
}

// BRepGPropTest.AdaptiveTrimmedQuarterDisk_ReusesBoundaryBuffers
#[test]
fn adaptive_trimmed_quarter_disk_reuses_boundary_buffers() {
    let radius = 4.0;
    let origin = Point3::new(0.0, 0.0, 0.0);
    let f = quarter_disk(radius);
    let mut order_face = BRepGPropFace::with_face(&f, false).unwrap();
    let mut domain = BRepGPropDomain::with_face(&f).unwrap();
    let mut orders = Vec::new();
    while domain.more() {
        assert!(order_face.load_edge(&domain.value().unwrap()).unwrap());
        orders.push(order_face.integration_order().unwrap());
        domain.next().unwrap();
    }
    assert_eq!(orders.len(), 3);
    assert!(orders.iter().any(|order| *order != orders[0]));
    let sinert = |eps: Option<f64>| {
        let mut face = BRepGPropFace::with_face(&f, false).unwrap();
        let mut domain = BRepGPropDomain::with_face(&f).unwrap();
        match eps {
            None => BRepGPropSinert::with_domain(&mut face, &mut domain, origin).unwrap(),
            Some(eps) => {
                BRepGPropSinert::with_domain_eps(&mut face, &mut domain, origin, eps).unwrap()
            }
        }
    };
    let fixed = sinert(None);
    let mut adaptive = sinert(Some(1.0e-8));
    assert!(adaptive.get_epsilon() >= 0.0 && adaptive.get_epsilon() <= 1.0e-8);
    assert!(near(adaptive.mass(), PI * radius * radius / 4.0, 1.0e-10));
    let expected = 4.0 * radius / (3.0 * PI);
    assert!(near(adaptive.centre_of_mass().x(), expected, 1.0e-7));
    assert!(near(adaptive.centre_of_mass().y(), expected, 1.0e-7));
    assert!(near(adaptive.centre_of_mass().z(), 0.0, 1.0e-12));
    assert_gprops_near(&adaptive, &fixed, 5.0e-5);
    let mut fallback = sinert(Some(1.0e-2));
    assert_eq!(fallback.get_epsilon(), 1.0e-2);
    assert_gprops_near(&fallback, &fixed, 5.0e-5);
}

// BRepGPropTest.AdaptiveVolumeProperties_Cylinder
#[test]
fn adaptive_volume_properties_cylinder() {
    let (radius, height) = (3.0, 7.0);
    let cylinder = shape(&CYLINDER_3_7);
    let mut fixed = GProps::new();
    BRepGProp::volume_properties(&cylinder, &mut fixed, false, false, false).unwrap();
    let mut adaptive = GProps::new();
    let error =
        BRepGProp::volume_properties_eps(&cylinder, &mut adaptive, 1.0e-8, false, false).unwrap();
    assert!((0.0..=1.0e-8).contains(&error));
    let mass = PI * radius * radius * height;
    assert!(near(adaptive.mass(), mass, 1.0e-10));
    let com = adaptive.centre_of_mass();
    assert!(near(com.x(), 0.0, 1.0e-12) && near(com.y(), 0.0, 1.0e-12));
    assert!(near(com.z(), height * 0.5, 1.0e-12));
    let m = adaptive.matrix_of_inertia();
    let side = mass * (3.0 * radius * radius + height * height) / 12.0;
    assert!(near(m.value(1, 1).unwrap(), side, 2.0e-4));
    assert!(near(m.value(2, 2).unwrap(), side, 2.0e-4));
    assert!(near(
        m.value(3, 3).unwrap(),
        mass * radius * radius / 2.0,
        2.0e-4
    ));
    assert_gprops_near(&adaptive, &fixed, 2.0e-4);
}

// BRepGPropTest.AdaptivePlaneLimitedInertia
#[test]
fn adaptive_plane_limited_inertia() {
    let normal = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let f = plane_rectangle(
        Plane3::new(Point3::new(0.0, 0.0, 2.0), normal).unwrap(),
        (0.0, 3.0, 0.0, 4.0),
    );
    let reference = Plane3::new(Point3::new(0.0, 0.0, 0.0), normal).unwrap();
    let location = Point3::new(0.0, 0.0, 0.0);
    let vinert = |eps: Option<f64>| {
        let mut face = BRepGPropFace::with_face(&f, false).unwrap();
        let mut domain = BRepGPropDomain::with_face(&f).unwrap();
        match eps {
            None => BRepGPropVinert::with_domain_plane(&mut face, &mut domain, reference, location)
                .unwrap(),
            Some(eps) => BRepGPropVinert::with_domain_plane_eps(
                &mut face,
                &mut domain,
                reference,
                location,
                eps,
            )
            .unwrap(),
        }
    };
    let (fixed, adaptive) = (vinert(None), vinert(Some(1.0e-8)));
    let m = adaptive.matrix_of_inertia();
    let expected = [
        ((1, 1), 160.0),
        ((2, 2), 104.0),
        ((3, 3), 200.0),
        ((1, 2), -72.0),
        ((1, 3), -36.0),
        ((2, 3), -48.0),
    ];
    for ((row, column), value) in expected {
        assert!(near(m.value(row, column).unwrap(), value, 1.0e-10));
    }
    assert_gprops_near(&adaptive, &fixed, 1.0e-10);
}

// BRepGPropTest.NaturalInfinitePlane_UsesInfiniteArithmetic
#[test]
fn natural_infinite_plane_uses_infinite_arithmetic() {
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
    let mut f = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut f, &surface, precision::confusion())
        .unwrap();
    let tool = BRepGPropFace::with_face(&f, false).unwrap();
    let props = BRepGPropSinert::with_face(&tool, Point3::new(0.0, 0.0, 0.0)).unwrap();
    assert!(precision::is_positive_infinite(props.mass()));
}

// BRepGPropTest.LinearProperties_SkipShared
#[test]
fn linear_properties_skip_shared() {
    let solid = shape(&BOX10);
    let (mut not_skipped, mut skipped) = (GProps::new(), GProps::new());
    BRepGProp::linear_properties(&solid, &mut not_skipped, false, false).unwrap();
    BRepGProp::linear_properties(&solid, &mut skipped, true, false).unwrap();
    assert!(near(skipped.mass(), 120.0, precision::confusion()));
    assert!(near(not_skipped.mass(), 240.0, precision::confusion()));
}

// BRepGPropTest.OCC49_CylinderHasSymmetryAxis
#[test]
fn occ49_cylinder_has_symmetry_axis() {
    let mut props = GProps::new();
    BRepGProp::volume_properties(&shape(&CYLINDER_10_20), &mut props, false, false, false).unwrap();
    assert!(props.principal_properties().unwrap().has_symmetry_axis());
}

// Native reference of BRepAlgoAPI_Cut(MakeCylinder(10, 20), MakeBox(10, 10, 10)).
#[test]
fn occ49_cut_shape_has_no_symmetry_axis_from_native_brep() {
    let source = include_bytes!("data/brep_gprop/occ49-cylinder-minus-box.brep");
    let mut stream = IoStream::from_bytes(source.to_vec());
    let mut cut_result = Shape::default();
    brep_tools::read(
        &mut cut_result,
        &mut stream,
        &BRepBuilder::default(),
        &ProgressRange::default(),
    )
    .unwrap();
    assert!(!cut_result.is_null());
    assert_eq!(cut_result.shape_type().unwrap(), ShapeEnum::Compound);
    assert_eq!(
        Explorer::new(&cut_result, ShapeEnum::Vertex, ShapeEnum::Shape)
            .unwrap()
            .count(),
        44
    );
    assert_eq!(
        Explorer::new(&cut_result, ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap()
            .count(),
        22
    );
    assert_eq!(
        Explorer::new(&cut_result, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .count(),
        6
    );
    let mut props = GProps::new();
    BRepGProp::volume_properties(&cut_result, &mut props, false, false, false).unwrap();
    assert!(near(props.mass(), 5497.7871437821368, 1.0e-9));
    let center = props.centre_of_mass();
    assert!(near(center.x(), -0.60630454511198362, 1.0e-12));
    assert!(near(center.y(), -0.60630454511198295, 1.0e-12));
    assert!(near(center.z(), 10.714285714285715, 1.0e-12));
    let inertia = props.matrix_of_inertia();
    let expected = [
        315878.24146288086,
        14521.015150373298,
        -19047.619047619028,
        14521.015150373298,
        315878.24146287912,
        -19047.619047619057,
        -19047.619047619028,
        -19047.619047619057,
        270847.32688836136,
    ];
    for (index, value) in expected.into_iter().enumerate() {
        let row = (index / 3 + 1) as i32;
        let column = (index % 3 + 1) as i32;
        assert!(near(inertia.value(row, column).unwrap(), value, 1.0e-7));
    }
    assert!(!props.principal_properties().unwrap().has_symmetry_axis());
}

// BRepGPropTest.OCC8797_BSplineLengthConsistencyAbscissaVsLinearProperties
#[test]
fn occ8797_bspline_length_consistency() {
    let points = [
        (0.0, 0.0),
        (1.0, 1.0),
        (2.0, 1.0),
        (3.0, 0.0),
        (4.0, 1.0),
        (5.0, 1.0),
        (6.0, 0.0),
    ];
    let mut poles = Array1::with_bounds(0, 6).unwrap();
    for (i, (x, y)) in points.into_iter().enumerate() {
        poles[i as i32] = Point3::new(x, y, 0.0);
    }
    let mut knots = Array1::with_bounds(0, 2).unwrap();
    let mut mults = Array1::with_bounds(0, 2).unwrap();
    for (i, (knot, mult)) in [(0.0, 4), (0.5, 3), (1.0, 4)].into_iter().enumerate() {
        knots[i as i32] = knot;
        mults[i as i32] = mult;
    }
    let spline = GeomBSplineCurve::new(&poles, &knots, &mults, 3, false).unwrap();
    assert_eq!((spline.nb_poles(), spline.nb_knots()), (7, 3));
    let curve: Handle<dyn GeomCurve> = Handle::new(spline).cast();
    let length_abscissa =
        AbscissaPoint::length(&GeomAdaptorCurve::with_curve(&curve).unwrap()).unwrap();
    assert!(length_abscissa > 0.0);
    let ends = (Point3::new(0.0, 0.0, 0.0), Point3::new(6.0, 0.0, 0.0));
    let e = edge_on(curve, (0.0, 1.0), ends);
    let mut props = GProps::new();
    BRepGProp::linear_properties(&e, &mut props, false, false).unwrap();
    assert!(props.mass() > 0.0);
    assert!(near(length_abscissa, props.mass(), props.mass() * 1e-3));
}

// BRepGPropTest.LinearProperties_DegenerateEdgeWithBSplinePCurveNoCrash
#[test]
fn linear_properties_degenerate_edge_with_bspline_pcurve_no_crash() {
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
    let mut poles = Array1::with_bounds(1, 4).unwrap();
    for (i, (u, v)) in [(0.0, 0.0), (0.1, 0.05), (0.2, -0.05), (0.3, 0.0)]
        .into_iter()
        .enumerate()
    {
        poles[i as i32 + 1] = Point2::new(u, v);
    }
    let mut knots = Array1::with_bounds(1, 2).unwrap();
    knots[2] = 1.0;
    let mut mults = Array1::with_bounds(1, 2).unwrap();
    mults.init(4);
    let pcurve = Geom2dBSplineCurve::new(&poles, &knots, &mults, 3, false).unwrap();
    let (first, last) = (pcurve.first_parameter(), pcurve.last_parameter());
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(pcurve).cast();
    let b = BRepBuilder::default();
    let mut e = Edge::default();
    b.make_edge(&mut e).unwrap();
    b.update_edge_pcurve(
        &e,
        &pcurve,
        &surface,
        &Location::default(),
        precision::confusion(),
        None,
    )
    .unwrap();
    b.range_on_surface(&e, &surface, &Location::default(), first, last)
        .unwrap();
    b.degenerated(&e, true).unwrap();
    let mut props = GProps::new();
    BRepGProp::linear_properties(&e, &mut props, false, false).unwrap();
    assert!(props.mass().is_finite());
    assert!(props.mass() >= 0.0);
}
