#![allow(non_snake_case)]
//! Rust translations of the declarations in `BRepGProp_Test.cxx`. Their shape records and
//! builder fixtures preserve the primitive, edge, and face inputs. Since Rust TKBO does not
//! yet provide Boolean Cut, the OCC49 cut assertion uses an asymmetric box to exercise the
//! same `HasSymmetryAxis` result.
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
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
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
fn BRepGPropTest__LinearProperties_EdgeLength() {
    let e = segment(Point3::new(0.0, 0.0, 0.0), Point3::new(3.0, 4.0, 0.0));
    let mut props = GProps::new();
    BRepGProp::linear_properties(&e, &mut props, false, false).unwrap();
    assert!(near(props.mass(), 5.0, precision::confusion()));
}

// BRepGPropTest.SurfaceProperties_BoxFaceArea
#[test]
fn BRepGPropTest__SurfaceProperties_BoxFaceArea() {
    let mut props = GProps::new();
    BRepGProp::surface_properties(&shape(&BOX_10_20_30), &mut props, false, false).unwrap();
    assert!(near(props.mass(), 2200.0, precision::confusion()));
}

// BRepGPropTest.VolumeProperties_UnitBox
#[test]
fn BRepGPropTest__VolumeProperties_UnitBox() {
    let mut props = GProps::new();
    BRepGProp::volume_properties(&shape(&BOX_1), &mut props, false, false, false).unwrap();
    assert!(near(props.mass(), 1.0, precision::confusion()));
}

// BRepGPropTest.VolumeProperties_Sphere
#[test]
fn BRepGPropTest__VolumeProperties_Sphere() {
    let mut props = GProps::new();
    BRepGProp::volume_properties(&shape(&SPHERE_5), &mut props, false, false, false).unwrap();
    assert!(near(props.mass(), (4.0 / 3.0) * PI * 125.0, 0.01));
}

// BRepGPropTest.VolumeProperties_BoxCenterOfMass
#[test]
fn BRepGPropTest__VolumeProperties_BoxCenterOfMass() {
    let mut props = GProps::new();
    BRepGProp::volume_properties(&shape(&BOX10), &mut props, false, false, false).unwrap();
    let com = props.centre_of_mass();
    for c in [com.x(), com.y(), com.z()] {
        assert!(near(c, 5.0, precision::confusion()));
    }
}

// BRepGPropTest.AdaptiveSurfaceProperties_Box
#[test]
fn BRepGPropTest__AdaptiveSurfaceProperties_Box() {
    let mut props = GProps::new();
    BRepGProp::surface_properties_eps(&shape(&BOX_10_20_30), &mut props, 1.0e-6, false).unwrap();
    assert!(near(props.mass(), 2200.0, 1.0e-9));
}

// BRepGPropTest.AdaptiveVolumeProperties_Box
#[test]
fn BRepGPropTest__AdaptiveVolumeProperties_Box() {
    let mut props = GProps::new();
    BRepGProp::volume_properties_eps(&shape(&BOX_10_20_30), &mut props, 1.0e-6, false, false)
        .unwrap();
    assert!(near(props.mass(), 6000.0, 1.0e-9));
}

// BRepGPropTest.AdaptiveNaturalBSplineSurface_ResizesIntervalArrays
#[test]
fn BRepGPropTest__AdaptiveNaturalBSplineSurface_ResizesIntervalArrays() {
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
fn BRepGPropTest__AdaptiveTrimmedQuarterDisk_ReusesBoundaryBuffers() {
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
fn BRepGPropTest__AdaptiveVolumeProperties_Cylinder() {
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
fn BRepGPropTest__AdaptivePlaneLimitedInertia() {
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
fn BRepGPropTest__NaturalInfinitePlane_UsesInfiniteArithmetic() {
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
fn BRepGPropTest__LinearProperties_SkipShared() {
    let solid = shape(&BOX10);
    let (mut not_skipped, mut skipped) = (GProps::new(), GProps::new());
    BRepGProp::linear_properties(&solid, &mut not_skipped, false, false).unwrap();
    BRepGProp::linear_properties(&solid, &mut skipped, true, false).unwrap();
    assert!(near(skipped.mass(), 120.0, precision::confusion()));
    assert!(near(not_skipped.mass(), 240.0, precision::confusion()));
}

// BRepGPropTest.OCC49_CylinderHasSymmetryAxis
#[test]
fn BRepGPropTest__OCC49_CylinderHasSymmetryAxis() {
    let mut props = GProps::new();
    BRepGProp::volume_properties(&shape(&CYLINDER_10_20), &mut props, false, false, false).unwrap();
    assert!(props.principal_properties().unwrap().has_symmetry_axis());
}

// BRepGPropTest.OCC49_CutShapeHasNoSymmetryAxis
#[test]
fn BRepGPropTest__OCC49_CutShapeHasNoSymmetryAxis() {
    let mut props = GProps::new();
    BRepGProp::volume_properties(&shape(&BOX_10_20_30), &mut props, false, false, false).unwrap();
    assert!(!props.principal_properties().unwrap().has_symmetry_axis());
}

// BRepGPropTest.OCC8797_BSplineLengthConsistencyAbscissaVsLinearProperties
#[test]
fn BRepGPropTest__OCC8797_BSplineLengthConsistencyAbscissaVsLinearProperties() {
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
fn BRepGPropTest__LinearProperties_DegenerateEdgeWithBSplinePCurveNoCrash() {
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
