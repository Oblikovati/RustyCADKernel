//! Native differential coverage for `ShapeUpgrade_ClosedFaceDivide`.
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Axis3;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis, BRepPrimAPIMakeSphere,
};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{FaceDivider, ShapeUpgradeClosedFaceDivide},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

#[test]
fn cylinder_seam_uses_native_default_split_count() -> Result<(), rustycad_tkernel::standard::Failure>
{
    let face = BRepPrimAPIMakeCylinder::new(2.0, 5.0).face()?;
    let divider = ShapeUpgradeClosedFaceDivide::new(face);
    assert_eq!(divider.nb_split_points(), 1);
    assert!(divider.perform(0.0)?);
    assert_native_row("cylinder_seam_default", &divider, true, 1, 1);
    Ok(())
}

#[test]
fn cylinder_seam_honors_multiple_requested_cut_points()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let face = BRepPrimAPIMakeCylinder::new(2.0, 5.0).face()?;
    let divider = ShapeUpgradeClosedFaceDivide::new(face);
    divider.set_nb_split_points(2);
    assert_eq!(divider.nb_split_points(), 2);
    assert!(divider.perform(0.0)?);
    assert_native_row("cylinder_seam_two_points", &divider, true, 2, 2);
    Ok(())
}

#[test]
fn trait_dispatch_uses_the_closed_face_splitter() -> Result<(), rustycad_tkernel::standard::Failure>
{
    let face = BRepPrimAPIMakeCylinder::new(2.0, 5.0).face()?;
    let divider = ShapeUpgradeClosedFaceDivide::new(face);
    divider.set_nb_split_points(0);
    assert_eq!(divider.nb_split_points(), 1);
    let policy: &dyn FaceDivider = &divider;
    assert!(policy.perform(0.0)?);
    assert_native_row("cylinder_seam_default", &divider, true, 1, 1);
    Ok(())
}

#[test]
fn sphere_uses_geometric_closure_fallback() -> Result<(), rustycad_tkernel::standard::Failure> {
    let face = BRepPrimAPIMakeSphere::new(2.0)?.face()?;
    let divider = ShapeUpgradeClosedFaceDivide::new(face);
    assert!(divider.perform(0.0)?);
    assert_native_row("sphere_geometric_closure", &divider, true, 3, 1);
    Ok(())
}

#[test]
fn geometric_closure_uses_configured_precision() -> Result<(), rustycad_tkernel::standard::Failure>
{
    let face = near_closed_face()?;
    let default_precision = ShapeUpgradeClosedFaceDivide::new(face.clone());
    assert!(!default_precision.perform(0.0)?);
    assert_native_row(
        "near_closed_default_precision",
        &default_precision,
        false,
        4,
        1,
    );
    let divider = ShapeUpgradeClosedFaceDivide::new(face);
    divider.set_precision(0.1);
    assert!(divider.perform(0.0)?);
    assert_native_row("near_closed_precision", &divider, true, 5, 1);
    Ok(())
}

fn near_closed_face() -> Result<rustycad_tkbrep::topods::Face, rustycad_tkernel::standard::Failure>
{
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.0)?).cast();
    BRepBuilderAPIMakeFace::from_surface_bounds(
        &surface,
        (0.0, std::f64::consts::TAU - 0.01, -1.0, 1.0),
        1e-7,
    )
    .unwrap()
    .face()
}

#[test]
fn direct_split_on_uninitialized_result_sets_fail3() {
    let divider = ShapeUpgradeClosedFaceDivide::empty();
    assert!(!divider.split_surface(0.0).unwrap());
    assert!(divider.status_flag(ShapeExtendStatus::Fail3));
    let actual = format!(
        "empty_result_fail3 changed=0 faces=0 edges=0 seams=0 done2=0 fail2=0 fail3={}",
        usize::from(divider.status_flag(ShapeExtendStatus::Fail3)),
    );
    let expected = include_str!("../../../tools/oracle/shape_upgrade_closed_face_divide.expected");
    assert_eq!(actual, expected.lines().next().unwrap());
}

#[test]
fn closed_face_divider_keeps_native_parent_rtti() {
    let divider = Handle::new(ShapeUpgradeClosedFaceDivide::empty());
    assert_eq!(
        divider.dynamic_type().unwrap().name(),
        "ShapeUpgrade_ClosedFaceDivide"
    );
    assert!(divider.is_kind_name("ShapeUpgrade_FaceDivide"));
}

fn assert_native_row(
    name: &str,
    divider: &ShapeUpgradeClosedFaceDivide,
    changed: bool,
    row: usize,
    split_points: i32,
) {
    let result = divider.result();
    let actual = format!(
        "{name} changed={} faces={} edges={} seams=0 done2={} fail2={} fail3={}",
        usize::from(changed),
        count(&result, ShapeEnum::Face),
        count(&result, ShapeEnum::Edge),
        usize::from(divider.status_flag(ShapeExtendStatus::Done2)),
        usize::from(divider.status_flag(ShapeExtendStatus::Fail2)),
        usize::from(divider.status_flag(ShapeExtendStatus::Fail3)),
    );
    let expected = include_str!("../../../tools/oracle/shape_upgrade_closed_face_divide.expected");
    assert_eq!(actual, expected.lines().nth(row).unwrap());
    assert_eq!(split_points, divider.nb_split_points());
}

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count()
}
