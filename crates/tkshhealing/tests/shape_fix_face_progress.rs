//! Port-only check: ShapeFix_Face::Perform with a progress range keeps a removed face removed.
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixFace};
#[path = "support/shape_fix_face.rs"]
mod shape_fix_face;
use shape_fix_face::unit_square_plane_face as planar_face;

#[test]
fn perform_with_progress_preserves_removed_face() {
    let face = planar_face();
    let context = Handle::new(ShapeBuildReShape::new());
    context.remove(&face.clone().into_shape()).unwrap();
    let fixer = ShapeFixFace::new(face.into_shape());
    fixer.set_context(&context);
    let progress = rustycad_tkernel::message::ProgressRange::default();

    assert!(!fixer.perform_with_progress(&progress).unwrap());
    assert!(fixer.status(ShapeExtendStatus::Ok));
    assert!(fixer.result().is_null());
}
