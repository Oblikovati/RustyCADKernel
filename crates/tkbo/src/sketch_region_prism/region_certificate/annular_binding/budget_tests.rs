// SPDX-License-Identifier: GPL-2.0-only
//! Bounds precede native detail queries and remain effective after the last external callback.
use super::*;
use super::super::super::frozen::complete_history::{HistoryBoundCause, PrismHistoryCause};
use rustycad_tkbrep::topods::borrowed_shape_batch;
struct Continue;
impl AnnularBindingProgress for Continue {
    fn checkpoint(&mut self, _: AnnularBindingStage) -> Result<bool, rustycad_tkernel::standard::Failure> { Ok(false) }
}
#[test]
fn oversized_copied_tree_refuses_before_copy_shape_or_curve_queries() {
    tests::with_actual(|request, region, receipt| {
        let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
        receipt.copied.as_mut().unwrap().base = borrowed_shape_batch(&[&edge, &edge, &edge]).unwrap();
        let Err(failure) = bind_annular_prism_v2(request, region, receipt, &mut Continue) else { panic!("expected copy preflight refusal"); };
        assert_exhausted(&failure, 10, 9);
    });
}
#[test]
fn oversized_swept_tree_refuses_before_chart_shape_or_surface_queries() {
    tests::with_actual(|request, region, receipt| {
        let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
        receipt.diagnostic.as_mut().unwrap().body = borrowed_shape_batch(&[&edge; 16]).unwrap();
        let Err(failure) = bind_annular_prism_v2(request, region, receipt, &mut Continue) else { panic!("expected body preflight refusal"); };
        assert_exhausted(&failure, 49, 48);
    });
}
fn assert_exhausted(failure: &AnnularBindingFailure<'_>, measured: usize, expected: usize) {
    let AnnularBindingCause::History(cause) = &*failure.cause else { panic!("expected original typed hierarchy cause"); };
    assert!(matches!(&**cause, PrismHistoryCause::Bounds(HistoryBoundCause::Exhausted { quantity: "directed uses", actual, maximum }) if *actual == measured && *maximum == expected));
    assert_eq!(failure.classification(), TwoLoopRefusalClass::Incomplete);
}
#[test]
fn actual_hierarchy_census_reserves_all_ancestry_and_slot_rows_once() {
    tests::with_actual(|request, _, receipt| {
        let budget = bounds::inspect(receipt, request.requested.maximum_retained_units).unwrap_or_else(|_| panic!("actual hierarchy"));
        assert_eq!(budget.roots.map(|r| r.directed_uses), [9, 9, 48]);
        assert_eq!(budget.roots[0].ancestor_elements, 18);
        assert_eq!(budget.roots[1].ancestor_elements, 18);
        assert!(bounds::inspect(receipt, budget.retained_units).is_ok());
        assert!(bounds::inspect(receipt, budget.retained_units - 1).is_err());
    });
}
#[test]
fn nested_parent_budget_refuses_before_legacy_history_materializes_rows() {
    tests::with_actual(|request, region, receipt| {
        let rows = &mut receipt.copied.as_mut().unwrap().relations[0].generated;
        let parent = rows.first().unwrap().clone();
        rows.resize(request.requested.maximum_retained_units + 1, parent);
        let Err(failure) = bind_annular_prism_v2(request, region, receipt, &mut Continue) else { panic!("expected nested parent budget refusal"); };
        assert!(matches!(&*failure.cause, AnnularBindingCause::History(cause) if matches!(&**cause, PrismHistoryCause::Bounds(HistoryBoundCause::Exhausted { quantity: "annular retained structural units", .. }))));
    });
}
#[test]
fn structural_unit_overflow_preserves_operands_without_allocating() {
    let mut used = usize::MAX;
    let Err(cause) = metadata_budget::reserve(&mut used, "authored overflow fixture", 1, usize::MAX) else { panic!("expected checked overflow"); };
    assert!(matches!(*cause, AnnularBindingCause::BudgetOverflow { quantity: "authored overflow fixture", retained: usize::MAX, increment: 1 }));
    assert_eq!(used, usize::MAX);
}
