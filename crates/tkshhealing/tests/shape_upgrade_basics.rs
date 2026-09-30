//! Behavior tests of the ShapeUpgrade package functions, ShapeUpgrade_Tool,
//! ShapeUpgrade_SplitCurve and ShapeUpgrade_RemoveLocations.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::{Array1, HSequence};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis3, Point2, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_upgrade::{
    ShapeUpgrade, ShapeUpgradeRemoveLocations, ShapeUpgradeSplitCurve, ShapeUpgradeTool, SplitCurve,
};

fn c0_cubic(periodic: bool) -> GeomBSplineCurve {
    let poles: Vec<Point3> = (0..6)
        .map(|i| Point3::new(f64::from(i), f64::from(i % 2), 0.0))
        .collect();
    let (knots, mults): (Vec<f64>, Vec<i32>) = if periodic {
        (vec![0.0, 1.0, 2.0, 3.0, 4.0], vec![1, 3, 1, 1, 1])
    } else {
        (vec![0.0, 1.0, 2.0], vec![4, 2, 4])
    };
    let poles = if periodic { poles } else { poles[..6].to_vec() };
    GeomBSplineCurve::new(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(&knots, 1),
        &Array1::from_shared(&mults, 1),
        3,
        periodic,
    )
    .unwrap()
}

#[test]
fn c0_bspline_splits_at_the_c0_knots() {
    let smooth = Handle::new(c0_cubic(false));
    // Multiplicity 2 on a cubic is C1: nothing to split.
    assert!(
        ShapeUpgrade::c0_bspline_to_sequence_of_c1_bspline_curve(&smooth)
            .unwrap()
            .is_none()
    );
    assert!(
        ShapeUpgrade::c0_bspline_to_sequence_of_c1_bspline_curve(&Handle::default())
            .unwrap()
            .is_none()
    );
    let periodic = Handle::new(c0_cubic(true));
    let pieces = ShapeUpgrade::c0_bspline_to_sequence_of_c1_bspline_curve(&periodic)
        .unwrap()
        .expect("a C0 knot");
    assert!(pieces.sequence(|s| s.len()) >= 1);
    // The input is made non-periodic in place, as natively.
    assert!(!periodic.is_periodic());
}

#[test]
fn c0_bspline2d_keeps_the_input_periodic() {
    let poles: Vec<Point2> = (0..6)
        .map(|i| Point2::new(f64::from(i), f64::from(i % 3)))
        .collect();
    let bs = Handle::new(
        Geom2dBSplineCurve::new(
            &Array1::from_shared(&poles, 1),
            &Array1::from_shared(&[0.0, 1.0, 2.0, 3.0, 4.0], 1),
            &Array1::from_shared(&[1, 3, 1, 1, 1], 1),
            3,
            true,
        )
        .unwrap(),
    );
    let pieces = ShapeUpgrade::c0_bspline2d_to_sequence_of_c1_bspline_curve(&bs).unwrap();
    assert!(pieces.is_some());
    assert!(rustycad_tkg2d::geom2d::Geom2dCurve::is_periodic(&*bs));
}

#[test]
fn tool_keeps_the_tolerances_around_the_precision() {
    let tool = Handle::new(ShapeUpgradeTool::new());
    tool.set_precision(1e-9);
    assert_eq!(tool.min_tolerance(), 1e-9);
    tool.set_precision(3.0);
    assert_eq!(
        (tool.max_tolerance(), tool.limit_tolerance(10.0)),
        (3.0, 3.0)
    );
    let other = Handle::new(ShapeUpgradeTool::new());
    other.set_context(&Handle::new(ShapeBuildReShape::new()));
    tool.set(&other).unwrap();
    assert!(tool.context() == other.context());
    assert_eq!(tool.max_tolerance(), 1.0);
    assert!(tool.set(&Handle::default()).is_err());
}

#[test]
fn split_values_are_inserted_per_original_interval() {
    let splitter = ShapeUpgradeSplitCurve::new();
    let values = Handle::new(HSequence::default());
    values.append(5.0);
    assert!(splitter.set_split_values(&values).is_err());
    splitter.init(0.0, 10.0);
    [5.0, 2.0, 8.0, -1.0]
        .into_iter()
        .for_each(|v| values.append(v));
    splitter.set_split_values(&values).unwrap();
    let got: Vec<f64> = splitter
        .split_values()
        .sequence(|s| s.iter().copied().collect());
    // Quirk kept: the values are compared with the bounds of the original interval only.
    assert_eq!(got, vec![0.0, 5.0, 5.0, 2.0, 8.0, 10.0]);
    splitter.set_split_values(&splitter.split_values()).unwrap();
    splitter.perform(true).unwrap();
    assert!(splitter.status_flag(ShapeExtendStatus::Ok));
}

#[test]
fn remove_locations_moves_the_face_location_into_the_surface() {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut trsf = Transform3::default();
    trsf.set_translation(Vector3::new(1.0, 2.0, 3.0));
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_located_surface(&mut face, &plane, &Location::new(trsf), 1e-7)
        .unwrap();
    let remover = ShapeUpgradeRemoveLocations::new();
    remover.set_remove_level(ShapeEnum::Edge);
    assert_eq!(remover.remove_level(), ShapeEnum::Face);
    assert!(remover.remove(&face).unwrap());
    let result: Shape = remover.get_result();
    let (_, location) =
        BRepTool::surface_with_location(&rustycad_tkbrep::topods::face(&result).unwrap()).unwrap();
    assert!(location.is_identity());
    assert!(remover.modified_shape(&face).is_same(&result));
    assert!(remover.remove(&Shape::default()).is_err());
}
