//! Block and cache ownership regression coverage for Context.
#[path = "../examples/bo_probe/fclass_faces.rs"]
mod faces;
use rustycad_tkbo::int_tools::{Context, Curve};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine},
    top_abs::State,
};
use rustycad_tkmath::gp::{Axis1, Axis2d, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use std::sync::Arc;
fn displaced_curve() -> (Curve, Handle<dyn Geom2dCurve>) {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
        Point3::new(0., 0., 10.),
        Direction3::new(1., 0., 0.).unwrap(),
    )))
    .into_cast();
    let pc: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).into_cast();
    let mut curve = Curve::default();
    curve.set_curve(&line);
    (curve, pc)
}
#[test]
fn pcurve_blocks_bypass_spatial_distance_when_both_are_present() {
    let c = Context::default();
    let f = faces::face(0, 2.).unwrap();
    let (mut curve, pc) = displaced_curve();
    assert!(!c.is_valid_block_for_face(0., 1., &curve, &f, 1e-7).unwrap());
    assert!(
        !c.is_valid_block_for_faces(0., 1., &curve, &f, &f, 1e-7)
            .unwrap()
    );
    curve.set_first_curve2d(&pc);
    assert!(
        !c.is_valid_block_for_faces(0., 1., &curve, &f, &f, 1e-7)
            .unwrap()
    );
    curve.set_second_curve2d(&pc);
    assert!(
        c.is_valid_block_for_faces(0., 1., &curve, &f, &f, 1e-7)
            .unwrap()
    );
}
#[test]
fn solid_and_bounding_boxes_keep_first_cached_configuration() {
    let c = Context::default();
    let solid = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)
        .unwrap()
        .solid()
        .unwrap();
    let classifier = c.solid_classifier(&solid).unwrap();
    classifier
        .write()
        .perform(&Point3::new(1., 1., 1.), 1e-7)
        .unwrap();
    assert_eq!(classifier.read().state(), State::In);
    assert!(Arc::ptr_eq(
        &classifier,
        &c.solid_classifier(&solid).unwrap()
    ));
}
#[test]
fn bounding_boxes_keep_first_cached_configuration() {
    let c = Context::default();
    let solid = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)
        .unwrap()
        .solid()
        .unwrap();
    let bbox = c.bnd_box(&solid).unwrap();
    assert!(Arc::ptr_eq(&bbox, &c.bnd_box(&solid).unwrap()));
    let obb = c.obb(&solid, 0.1).unwrap();
    let half = obb.read().x_h_size();
    assert!(Arc::ptr_eq(&obb, &c.obb(&solid, 5.).unwrap()));
    assert_eq!(half, obb.read().x_h_size());
}
#[test]
fn hatcher_settings_and_native_transient_identity() {
    let c = Handle::new(Context::default());
    assert!(!c.cast::<StandardTransient>().is_null());
    let service = c.hatcher(&faces::face(0, 2.).unwrap()).unwrap();
    let h = service.read();
    assert_eq!(h.confusion2d(), 1e-8);
    assert_eq!(h.confusion3d(), 1e-8);
    assert_eq!(h.intersector().confusion_tolerance(), 1e-10);
    assert_eq!(h.intersector().tangency_tolerance(), 1e-10);
    assert!(h.keep_points());
    assert!(!h.keep_segments());
}
