//! Whole-shape continuity selection against pinned OCCT behavior.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{Shape, Wire},
};
use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    standard::Handle,
};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::{
    geom::{GeomBSplineCurve, GeomBSplineSurface, GeomPlane, GeomSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::{
    geom_abs::Shape as Continuity,
    gp::{Axis3, Point2, Point3},
};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{
        ShapeDivider, ShapeUpgradeShapeDivideContinuity, ShapeUpgradeSplitCurve2dContinuity,
        ShapeUpgradeSplitCurve3dContinuity, ShapeUpgradeSplitSurfaceContinuity,
    },
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

fn c0_wire() -> Shape {
    let edge = BRepBuilderAPIMakeEdge::from_curve(&c0_curve())
        .unwrap()
        .edge()
        .unwrap();
    let mut wire = Wire::default();
    let builder = BRepBuilder::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    wire.into_shape()
}

fn c0_curve() -> Handle<dyn rustycad_tkg3d::geom::GeomCurve> {
    let poles = [
        rustycad_tkmath::gp::Point3::new(0., 0., 0.),
        rustycad_tkmath::gp::Point3::new(0.5, 1., 0.),
        rustycad_tkmath::gp::Point3::new(1., 1., 0.),
        rustycad_tkmath::gp::Point3::new(1.5, 0., 0.),
        rustycad_tkmath::gp::Point3::new(2., 0., 0.),
    ];
    let poles = Array1::from_shared(&poles, 1);
    let knots = Array1::from_shared(&[0., 0.5, 1.], 1);
    let mults = Array1::from_shared(&[3, 2, 3], 1);
    let curve = GeomBSplineCurve::new(&poles, &knots, &mults, 2, false).unwrap();
    Handle::new(curve).cast()
}

fn c0_surface() -> Handle<dyn GeomSurface> {
    let poles = c0_surface_poles();
    let knots = Array1::from_shared(&[0., 0.5, 1.], 1);
    let mults = Array1::from_shared(&[3, 2, 3], 1);
    Handle::new(
        GeomBSplineSurface::new(
            &poles,
            (&knots, &mults),
            (&knots, &mults),
            (2, 2),
            (false, false),
        )
        .unwrap(),
    )
    .cast()
}

fn c0_surface_poles() -> Array2<'static, Point3> {
    let mut poles = Array2::with_bounds(1, 5, 1, 5).unwrap();
    for u in 1_i32..=5 {
        for v in 1_i32..=5 {
            let z = (u.saturating_sub(3).max(0) + v.saturating_sub(3).max(0)) as f64;
            poles[(u, v)] = Point3::new((u - 1) as f64 / 4., (v - 1) as f64 / 4., z);
        }
    }
    poles
}

fn c0_face() -> Shape {
    BRepBuilderAPIMakeFace::from_surface_bounds(&c0_surface(), (0., 1., 0., 1.), 1.0e-7)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn curve_2d(points: &[Point2]) -> Handle<dyn Geom2dCurve> {
    let poles = Array1::from_shared(points, 1);
    let knots = if points.len() == 3 {
        vec![0., 0.5, 1.]
    } else {
        vec![0., 1.]
    };
    let multiplicities = if points.len() == 3 {
        vec![2, 1, 2]
    } else {
        vec![2, 2]
    };
    let knots = Array1::from_shared(&knots, 1);
    let multiplicities = Array1::from_shared(&multiplicities, 1);
    Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &multiplicities, 1, false).unwrap()).cast()
}

fn c0_pcurve_face() -> Shape {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let wire = c0_pcurve_wire(&surface);
    BRepBuilderAPIMakeFace::from_surface_wire(&surface, &wire, true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn c0_pcurve_wire(surface: &Handle<dyn GeomSurface>) -> Wire {
    let curves = c0_boundary_pcurves();
    let mut wire = Wire::default();
    let builder = BRepBuilder::default();
    builder.make_wire(&mut wire);
    add_pcurve_edges(surface, &curves, &builder, &mut wire);
    wire
}

fn c0_boundary_pcurves() -> [Handle<dyn Geom2dCurve>; 4] {
    [
        curve_2d(&[
            Point2::new(0., 0.),
            Point2::new(0.5, 0.2),
            Point2::new(1., 0.),
        ]),
        curve_2d(&[Point2::new(1., 0.), Point2::new(1., 1.)]),
        curve_2d(&[Point2::new(1., 1.), Point2::new(0., 1.)]),
        curve_2d(&[Point2::new(0., 1.), Point2::new(0., 0.)]),
    ]
}

fn add_pcurve_edges(
    surface: &Handle<dyn GeomSurface>,
    curves: &[Handle<dyn Geom2dCurve>; 4],
    builder: &BRepBuilder,
    wire: &mut Wire,
) {
    for curve in curves {
        let edge = BRepBuilderAPIMakeEdge::from_pcurve_params(curve, surface, 0., 1.)
            .unwrap()
            .edge()
            .unwrap();
        builder.add(wire, &edge).unwrap();
    }
}

fn count(shape: &Shape, kind: ShapeEnum) -> i32 {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut result = 0;
    while explorer.more() {
        result += 1;
        explorer.next().unwrap();
    }
    result
}

fn observe(id: i32, criterion: Continuity, tolerance: f64) -> Vec<i32> {
    let divider = ShapeUpgradeShapeDivideContinuity::new(c0_wire());
    divider.set_boundary_criterion(criterion);
    divider.set_tolerance(tolerance);
    let changed = (&divider as &dyn ShapeDivider).perform(true).unwrap();
    let result = divider.result();
    let mut row = vec![id, i32::from(changed)];
    row.extend(
        [ShapeEnum::Wire, ShapeEnum::Edge, ShapeEnum::Vertex].map(|kind| count(&result, kind)),
    );
    row.extend(statuses(&divider));
    row
}

fn observe_surface(id: i32, criterion: Continuity) -> Vec<i32> {
    let divider = ShapeUpgradeShapeDivideContinuity::new(c0_face());
    divider.set_surface_criterion(criterion);
    let changed = (&divider as &dyn ShapeDivider).perform(true).unwrap();
    let result = divider.result();
    let mut row = vec![id, i32::from(changed)];
    row.extend([ShapeEnum::Face, ShapeEnum::Edge].map(|kind| count(&result, kind)));
    row.extend(statuses(&divider));
    row
}

fn observe_pcurve(id: i32, tolerance_2d: Option<f64>) -> Vec<i32> {
    let divider = ShapeUpgradeShapeDivideContinuity::new(c0_pcurve_face());
    configure_pcurve(&divider, tolerance_2d);
    let changed = (&divider as &dyn ShapeDivider).perform(true).unwrap();
    let result = divider.result();
    let mut row = vec![id, i32::from(changed)];
    row.extend(
        [
            ShapeEnum::Face,
            ShapeEnum::Wire,
            ShapeEnum::Edge,
            ShapeEnum::Vertex,
        ]
        .map(|kind| count(&result, kind)),
    );
    row.extend(statuses(&divider));
    row
}

fn configure_pcurve(divider: &ShapeUpgradeShapeDivideContinuity, tolerance_2d: Option<f64>) {
    divider.set_boundary_criterion(Continuity::C0);
    divider.set_pcurve_criterion(Continuity::C1);
    divider.set_surface_criterion(Continuity::C0);
    divider.set_tolerance(1.0e-7);
    if let Some(tolerance) = tolerance_2d {
        divider.set_tolerance_2d(tolerance);
    }
}

fn statuses(divider: &ShapeUpgradeShapeDivideContinuity) -> [i32; 5] {
    [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
        ShapeExtendStatus::Fail3,
    ]
    .map(|flag| i32::from(divider.status_flag(flag)))
}

fn expected_row(index: usize) -> Vec<i32> {
    include_str!("../../../tools/oracle/shape_upgrade_shape_divide_continuity.expected")
        .lines()
        .nth(index)
        .unwrap()
        .split_whitespace()
        .map(|word| word.parse().unwrap())
        .collect()
}

#[test]
fn boundary_criteria_match_native_end_to_end_rows() {
    for (id, criterion, tolerance) in [
        (0, Continuity::C0, 1.0e-7),
        (1, Continuity::C1, 1.0e-7),
        (2, Continuity::C2, 1.0e-7),
        (3, Continuity::C1, 2.0),
    ] {
        assert_eq!(observe(id, criterion, tolerance), expected_row(id as usize));
    }
}

#[test]
fn surface_criteria_match_native_end_to_end_rows() {
    for (id, criterion) in [
        (4, Continuity::C0),
        (5, Continuity::C1),
        (6, Continuity::C2),
    ] {
        assert_eq!(observe_surface(id, criterion), expected_row(id as usize));
    }
}

#[test]
fn pcurve_criteria_and_tolerance_match_native_end_to_end_rows() {
    for (index, tolerance) in [(7, None), (8, Some(2.0))] {
        assert_eq!(observe_pcurve(index as i32, tolerance), expected_row(index));
    }
}

#[test]
fn virtual_face_tool_is_fresh_and_contains_continuity_children() {
    let divider = ShapeUpgradeShapeDivideContinuity::new(Shape::default());
    let first = divider.split_face_tool();
    let second = divider.split_face_tool();
    assert_ne!(first.object_address(), second.object_address());
    let virtual_tool = (&divider as &dyn ShapeDivider).split_face_tool();
    assert_ne!(second.object_address(), virtual_tool.object_address());
    let (curve_2d, curve_3d, surface_tool) = child_tools(&first);
    assert!(!curve_2d.is_null());
    assert!(!curve_3d.is_null());
    assert!(!surface_tool.is_null());
}

fn child_tools(
    face: &rustycad_tkernel::standard::Handle<
        rustycad_tkshhealing::shape_upgrade::ShapeUpgradeFaceDivide,
    >,
) -> (
    Handle<ShapeUpgradeSplitCurve2dContinuity>,
    Handle<ShapeUpgradeSplitCurve3dContinuity>,
    Handle<ShapeUpgradeSplitSurfaceContinuity>,
) {
    use rustycad_tkshhealing::shape_upgrade::{FaceDivider, WireDivider};
    let face_policy = face.cast::<dyn FaceDivider>();
    let face_policy = face_policy.get().unwrap();
    let wire_policy = face_policy.wire_divide_tool().cast::<dyn WireDivider>();
    let wire_policy = wire_policy.get().unwrap();
    let curve_2d = wire_policy.split_curve_2d_tool().cast();
    let curve_3d = wire_policy.split_curve_3d_tool().cast();
    let surface = face_policy.split_surface_tool();
    (curve_2d, curve_3d, surface.cast())
}
