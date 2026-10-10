//! Behavioral coverage of TopOpeBRepDS_CurveExplorer.hxx/.cxx at
//! OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSCurveExplorer, TopOpeBRepDSDataStructure,
};

fn sparse_curves() -> TopOpeBRepDSDataStructure {
    let mut curves = TopOpeBRepDSDataStructure::new();
    let mut curve = TopOpeBRepDSCurve::default();
    curve.set_tolerance(0.25);
    curves.add_curve(&mut curve);
    curves.change_nb_curves(3);
    curve.change_keep(false);
    curves.add_curve(&mut curve);
    curve.change_keep(true);
    curves.add_curve(&mut curve);
    curves
}

#[test]
fn default_explorer_is_exhausted_with_an_empty_curve() {
    let mut explorer = TopOpeBRepDSCurveExplorer::default();
    assert!(!explorer.more());
    assert_eq!(explorer.index(), 1);
    assert!(explorer.curve().curve().is_null());
    assert!(explorer.curve().keep());
    explorer.next();
    assert_eq!(explorer.index(), 2);
    assert!(!explorer.more());
}

#[test]
fn kept_iteration_skips_holes_and_discarded_curves() {
    let curves = sparse_curves();
    let mut explorer = TopOpeBRepDSCurveExplorer::new(&curves, true);
    assert_eq!(explorer.index(), 1);
    assert_eq!(explorer.curve().tolerance(), 0.25);
    assert!(std::ptr::eq(explorer.curve(), curves.curve(1)));
    explorer.next();
    assert_eq!(explorer.index(), 5);
    assert!(explorer.more());
    explorer.next();
    assert_eq!(explorer.index(), 6);
    assert!(!explorer.more());
    assert_eq!(explorer.curve().ds_index(), 0);
}

#[test]
fn reinitialization_changes_filter_and_indexed_lookup_ignores_filter() {
    let curves = sparse_curves();
    let mut explorer = TopOpeBRepDSCurveExplorer::new(&curves, true);
    assert!(explorer.is_curve(4));
    assert!(!explorer.is_curve_keep(4));
    assert_eq!(explorer.curve_at(4).ds_index(), 4);
    assert!(!explorer.curve_at(4).keep());
    explorer.init(&curves, false);
    explorer.next();
    assert_eq!(explorer.index(), 4);
    explorer.next();
    assert_eq!(explorer.index(), 5);
    assert!(explorer.is_curve_keep(5));
}

#[test]
fn unknown_indices_use_the_explorers_own_fallback() {
    let mut curves = sparse_curves();
    curves.change_curve(2).set_tolerance(12.0);
    let explorer = TopOpeBRepDSCurveExplorer::new(&curves, false);
    for index in [-1, 0, 2, 3, 6] {
        assert!(!explorer.is_curve(index));
        assert!(!explorer.is_curve_keep(index));
        assert_eq!(explorer.curve_at(index).ds_index(), 0);
        assert_ne!(explorer.curve_at(index).tolerance(), 12.0);
    }
}

#[test]
fn counting_restarts_and_exhausts_the_current_filter() {
    let curves = sparse_curves();
    let mut explorer = TopOpeBRepDSCurveExplorer::new(&curves, true);
    explorer.next();
    assert_eq!(explorer.nb_curve(), 2);
    assert_eq!(explorer.index(), 6);
    assert!(!explorer.more());
    explorer.init(&curves, false);
    assert_eq!(explorer.nb_curve(), 3);
    assert_eq!(explorer.nb_curve(), 3);
    assert!(!explorer.more());
}

#[test]
fn empty_structure_finishes_at_one() {
    let curves = TopOpeBRepDSDataStructure::new();
    let mut explorer = TopOpeBRepDSCurveExplorer::new(&curves, true);
    assert_eq!(explorer.nb_curve(), 0);
    assert_eq!(explorer.index(), 1);
    assert!(!explorer.more());
}

#[test]
fn exclusive_explorer_observes_edits_but_refreshes_its_limit_only_when_counting() {
    let mut curves = sparse_curves();
    let mut explorer = TopOpeBRepDSCurveExplorer::new_mut(&mut curves, true);
    let source = explorer.source_mut().unwrap();
    source.change_keep_curve(4, true);
    source.add_curve(&mut TopOpeBRepDSCurve::default());
    explorer.next();
    assert_eq!(explorer.index(), 4);
    explorer.next();
    assert_eq!(explorer.index(), 5);
    explorer.next();
    assert!(!explorer.more());
    assert!(explorer.is_curve(6));
    assert_eq!(explorer.curve_at(6).ds_index(), 6);
    assert_eq!(explorer.nb_curve(), 4);
    assert_eq!(explorer.index(), 7);
}

#[test]
fn shared_and_unbound_explorers_do_not_offer_mutable_access() {
    assert!(TopOpeBRepDSCurveExplorer::default().source_mut().is_none());
    let curves = sparse_curves();
    let mut explorer = TopOpeBRepDSCurveExplorer::new(&curves, true);
    assert!(explorer.source_mut().is_none());
}

#[test]
fn clearing_source_keeps_found_state_until_next_and_current_uses_source_fallback() {
    let mut curves = sparse_curves();
    let mut explorer = TopOpeBRepDSCurveExplorer::new_mut(&mut curves, true);
    let source = explorer.source_mut().unwrap();
    source.init();
    source.change_curve(-1).set_tolerance(12.0);
    assert!(explorer.more());
    assert_eq!(explorer.curve().tolerance(), 12.0);
    assert_ne!(explorer.curve_at(1).tolerance(), 12.0);
    explorer.next();
    assert!(!explorer.more());
    assert_eq!(explorer.index(), 6);
}
