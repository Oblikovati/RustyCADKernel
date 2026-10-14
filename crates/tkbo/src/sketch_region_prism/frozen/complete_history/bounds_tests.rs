// SPDX-License-Identifier: GPL-2.0-only
use super::bounds::preflight_history_root;
use super::*;
use rustycad_tkbrep::topods::{Builder, Compound, Shape, borrowed_shape_batch};
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Location,
};

#[test]
fn bounded_history_counts_actual_repeated_uses_without_truncation() {
    let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let root = borrowed_shape_batch(&[&edge, &edge]).unwrap();
    assert_eq!(
        preflight_history_root(&root, PrismHistoryLimits::bounded_v1()).unwrap(),
        7
    );
    let mut limits = PrismHistoryLimits::bounded_v1();
    limits.maximum_uses = 6;
    assert!(matches!(
        preflight_history_root(&root, limits),
        Err(HistoryBoundCause::Exhausted {
            actual: 7,
            maximum: 6,
            ..
        })
    ));
    limits.maximum_uses = 7;
    limits.maximum_depth = 1;
    assert!(matches!(
        preflight_history_root(&root, limits),
        Err(HistoryBoundCause::Exhausted {
            quantity: "ancestor depth",
            actual: 2,
            maximum: 1
        })
    ));
}
#[test]
fn bounded_history_rejects_changing_placement_tshape_cycle() {
    let mut first = compound();
    let mut second = compound();
    Builder.add(&mut first, &second).unwrap();
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(1., 0., 0.));
    let moved = first.located(&Location::new(transform), true).unwrap();
    assert!(first.is_partner(&moved) && !first.is_same(&moved));
    second.set_free(true).unwrap();
    Builder.add(&mut second, &moved).unwrap();
    let result = preflight_history_root(&first, PrismHistoryLimits::bounded_v1());
    Builder.remove(&mut second, &moved).unwrap(); // Break intentional ownership cycle before assertions.
    assert!(matches!(
        result,
        Err(HistoryBoundCause::AncestorCycle {
            ancestor_depth: 0,
            repeated_depth: 2
        })
    ));
}
#[test]
fn bounded_history_retains_actual_iterator_failure() {
    let root =
        crate::changing_child_count::changing_shape(rustycad_tkg3d::top_abs::ShapeEnum::Face);
    let Err(HistoryBoundCause::Native(cause)) =
        preflight_history_root(&root, PrismHistoryLimits::bounded_v1())
    else {
        panic!("expected actual iterator failure");
    };
    assert_eq!(cause.exception_type(), "Standard_NoSuchObject");
}
#[test]
fn bounded_history_settings_refuse_zero_and_overflow_without_allocating() {
    for value in [0, usize::MAX] {
        let mut limits = PrismHistoryLimits::bounded_v1();
        limits.maximum_uses = value;
        assert!(
            matches!(preflight_history_root(&Shape::default(), limits), Err(HistoryBoundCause::Setting { actual, .. }) if actual == value)
        );
    }
    fn require<T: Send + Sync>() {}
    require::<HistoryBoundCause>();
    require::<PrismHistoryLimits>();
}
fn compound() -> Compound {
    let mut result = Compound::default();
    Builder.make_compound(&mut result);
    result
}
