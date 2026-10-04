//! Supplemental copy protocol, separate from the twelve original legacy declarations.
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorTransformedSurface;
use rustycad_tkmath::gp::{Plane3, Point3, Transform3, Vector3};

type CopyAdaptor = GeomAdaptorTransformedSurface;

fn native_bezier_fixture() -> Handle<dyn GeomSurface> {
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles[(1, 1)] = Point3::new(0.0, 0.0, 0.0);
    poles[(1, 2)] = Point3::new(0.0, 1.0, 0.0);
    poles[(2, 1)] = Point3::new(1.0, 0.0, 0.0);
    poles[(2, 2)] = Point3::new(1.0, 1.0, 1.0);
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}

fn cached_fixture(spline: &Handle<dyn GeomSurface>) -> CopyAdaptor {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(4.0, 0.0, 0.0));
    let source = CopyAdaptor::with_surface(spline, transform).unwrap();
    source.init_transformed_cache().unwrap();
    source
        .adaptor_surface_transformed()
        .unwrap()
        .eval_d0(0.25, 0.5)
        .unwrap();
    source
}

fn cached_member_receipt(
    source: &CopyAdaptor,
    copy: &CopyAdaptor,
    plane: &Handle<dyn GeomSurface>,
) -> String {
    let original = source.adaptor_surface_transformed().unwrap();
    let copied = copy.adaptor_surface_transformed().unwrap();
    let distinct = original != copied;
    original.load(plane).unwrap();
    let replaced = source.geom_surface_transformed().unwrap() == *plane;
    let preserved = copy.geom_surface_transformed().unwrap() != *plane;
    format!(
        "distinct-adaptor {}\ncached-state-independent {} {}\n",
        i32::from(distinct),
        i32::from(replaced),
        i32::from(preserved)
    )
}

fn original_member_receipt(
    source: &CopyAdaptor,
    copy: &CopyAdaptor,
    spline: &Handle<dyn GeomSurface>,
    plane: &Handle<dyn GeomSurface>,
) -> String {
    source.adaptor_surface_original().load(plane).unwrap();
    let preserved = copy.geom_surface_original() == *spline;
    let replaced = source.geom_surface_original() == *plane;
    format!(
        "original-state-independent {} {}\n",
        i32::from(preserved),
        i32::from(replaced)
    )
}

#[test]
fn protected_shallow_copy_matches_native_member_isolation() {
    let spline = native_bezier_fixture();
    let source = cached_fixture(&spline);
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(Plane3::default())).cast();
    let copied: Handle<CopyAdaptor> = source.shallow_copy().unwrap().cast();
    let observed = format!(
        "{}{}",
        cached_member_receipt(&source, &copied, &plane),
        original_member_receipt(&source, &copied, &spline, &plane)
    );
    assert_eq!(
        observed,
        include_str!("../../../tools/oracle/transformed_surface_shallow_value_copy.expected")
    );
}
