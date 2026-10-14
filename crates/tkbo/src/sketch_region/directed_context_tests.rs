// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkbrep::topods::{Builder, Compound, borrowed_shape_batch};
use rustycad_tkg3d::top_abs::ShapeEnum;

#[test]
fn shared_context_traversal_retains_repeated_directed_child_paths() {
    let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let root = borrowed_shape_batch(&[&edge, &edge]).unwrap();
    let rows = strict_region_contexts(&root).unwrap_or_else(|_| panic!("expected actual contexts"));
    assert_eq!(rows.len(), 7);
    let edges: Vec<_> = rows
        .iter()
        .filter(|row| row.occurrence.shape_type().unwrap() == ShapeEnum::Edge)
        .collect();
    assert_eq!(edges.len(), 2);
    assert!(edges[0].occurrence.is_equal(&edges[1].occurrence));
    assert_eq!(edges[0].child_slots, [0]);
    assert_eq!(edges[1].child_slots, [1]);
    assert!(
        rows.iter()
            .all(|row| row.ancestors.len() == row.child_slots.len())
    );
}
#[test]
fn neutral_traversal_preserves_native_iterator_failure() {
    let root = crate::changing_child_count::changing_shape(ShapeEnum::Face);
    let Err(RegionContextFailure::Native(cause)) = strict_region_contexts(&root) else {
        panic!("expected actual iterator read failure");
    };
    assert_eq!(cause.exception_type(), "Standard_NoSuchObject");
    assert!(cause.message().contains("TopoDS_Iterator::Value"));
}
#[test]
fn neutral_traversal_preserves_same_placement_ancestor_cycle_cause() {
    let mut first = Compound::default();
    let mut second = Compound::default();
    Builder.make_compound(&mut first);
    Builder.make_compound(&mut second);
    Builder.add(&mut first, &second).unwrap();
    second.set_free(true).unwrap();
    Builder.add(&mut second, &first).unwrap();
    let result = strict_region_contexts(&first);
    Builder.remove(&mut second, &first).unwrap(); // Break the intentional ownership cycle.
    assert!(
        matches!(result, Err(RegionContextFailure::RepeatedAncestor { member }) if member.is_same(&first))
    );
}
