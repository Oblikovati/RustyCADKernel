//! FaceDivide's pinned no-context safety regressions from ShapeUpgrade_FaceDivide_Test.cxx.
use std::sync::atomic::{AtomicBool, Ordering};

use rustycad_tkbrep::{topexp::Explorer, topods};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeFaceDivide;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn first_box_face(x: f64) -> topods::Face {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(x, x + 5.0, x + 10.0).unwrap();
    let shape = maker.shape().unwrap();
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    assert!(faces.more(), "box contains a face");
    topods::face(faces.current()).unwrap()
}

#[test]
fn perform_without_context_does_not_crash() {
    for index in 0..10 {
        let face = first_box_face(10.0 + index as f64 * 0.5);
        let divider = ShapeUpgradeFaceDivide::new(face.clone());
        assert!(!divider.perform(0.0).unwrap());
        assert!(!divider.context().is_null());
        assert!(divider.result().is_same(&face.into()));
    }
}

#[test]
fn perform_parallel_does_not_crash() {
    let failed = AtomicBool::new(false);
    std::thread::scope(|scope| {
        for index in 0..100 {
            let failed = &failed;
            scope.spawn(move || {
                let face = first_box_face(10.0 + index as f64 * 0.1);
                if ShapeUpgradeFaceDivide::new(face).perform(0.0).is_err() {
                    failed.store(true, Ordering::Relaxed);
                }
            });
        }
    });
    assert!(!failed.load(Ordering::Relaxed));
}
