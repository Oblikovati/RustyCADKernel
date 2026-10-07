//! Regression for ShapeFix_FixSmallFace::Perform on a thin box: the strip faces are
//! replaced by shared edges, and `Shape()` returns the shape after FixShape (native
//! `myShape = FixShape()`), whose wires ShapeFix_Face has reordered.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixFixSmallFace};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// The orientation of each edge of each wire, in explorer order.
fn wire_orientations(shape: &Shape) -> Vec<Vec<Orientation>> {
    Explorer::new(shape, ShapeEnum::Wire, ShapeEnum::Shape)
        .unwrap()
        .map(|wire| {
            ShapeIterator::new(&wire, true, true)
                .map(|e| e.orientation())
                .collect()
        })
        .collect()
}

#[test]
fn thin_box_strips_are_removed_and_faces_fixed() {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1e-5).unwrap();
    maker.build(&ProgressRange::default()).unwrap();
    let thin = maker.shape().unwrap().clone();
    let fixer = ShapeFixFixSmallFace::default();
    fixer.init(thin);
    FixRoot::set_precision(&fixer, 1e-3);
    fixer.perform().unwrap();
    assert!(fixer.status(ShapeExtendStatus::Done2));
    let result = fixer.shape();
    let faces = Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .count();
    assert_eq!(faces, 2);
    // OCCT 3d097a0 (sh_fix_shape thin box, precision 1e-3, op 6).
    use Orientation::{Forward as F, Reversed as R};
    assert_eq!(
        wire_orientations(&result),
        vec![vec![R, R, F, F], vec![R, F, F, R]]
    );
}
