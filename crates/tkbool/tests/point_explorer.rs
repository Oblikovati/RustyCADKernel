//! Behavior of TopOpeBRepDS_PointExplorer at pinned OCCT 3d097a0.
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSDataStructure, TopOpeBRepDSPoint, TopOpeBRepDSPointExplorer,
};
use rustycad_tkmath::gp::Point3;

fn point_source() -> TopOpeBRepDSDataStructure {
    let mut source = TopOpeBRepDSDataStructure::new();
    for index in 1..=4 {
        let mut point = TopOpeBRepDSPoint::new(Point3::new(f64::from(index), 2., 3.), 0.25);
        point.change_keep(index % 2 == 0);
        source.add_point(&point);
    }
    source
}

#[test]
fn unbound_point_explorer_is_exhausted_and_returns_origin() {
    let mut explorer = TopOpeBRepDSPointExplorer::default();
    assert!(!explorer.more());
    assert_eq!(explorer.index(), 1);
    assert_eq!(explorer.point().unwrap().point(), Point3::default());
    assert!(explorer.point().unwrap().keep());
    assert!(explorer.source_mut().is_none());
    explorer.next();
    assert_eq!(explorer.index(), 2);
    assert!(!explorer.more());
}

#[test]
fn point_filter_skips_discarded_points_but_indexed_access_keeps_them() {
    let source = point_source();
    let mut explorer = TopOpeBRepDSPointExplorer::new(&source, true);
    assert_eq!(explorer.index(), 2);
    assert_eq!(explorer.point().unwrap().point().x(), 2.);
    assert!(std::ptr::eq(
        explorer.point().unwrap(),
        source.point(2).unwrap()
    ));
    assert!(explorer.is_point(1));
    assert!(!explorer.is_point_keep(1));
    assert_eq!(explorer.point_at(1).point().x(), 1.);
    assert!(explorer.is_point_keep(2));
    explorer.next();
    assert_eq!(explorer.index(), 4);
    explorer.next();
    assert_eq!(explorer.index(), 5);
    assert!(!explorer.more());
}

#[test]
fn counting_points_restarts_and_exhausts_its_filter() {
    let source = point_source();
    let mut explorer = TopOpeBRepDSPointExplorer::new(&source, true);
    explorer.next();
    assert_eq!(explorer.nb_point(), 2);
    assert_eq!(explorer.index(), 5);
    assert!(!explorer.more());
    explorer.init(&source, false);
    assert_eq!(explorer.index(), 1);
    assert_eq!(explorer.nb_point(), 4);
    assert!(!explorer.more());
    assert!(explorer.source_mut().is_none());
}

#[test]
fn unknown_point_indices_return_the_explorers_origin() {
    let source = point_source();
    let explorer = TopOpeBRepDSPointExplorer::new(&source, true);
    for index in [-1, 0, 5] {
        assert!(!explorer.is_point(index));
        assert!(!explorer.is_point_keep(index));
        assert_eq!(explorer.point_at(index).point(), Point3::default());
    }
}

#[test]
fn point_edits_are_live_and_recount_refreshes_the_initial_limit() {
    let mut source = point_source();
    let mut explorer = TopOpeBRepDSPointExplorer::new_mut(&mut source, true);
    let source = explorer.source_mut().unwrap();
    source.change_keep_point(3, true).unwrap();
    source.add_point(&TopOpeBRepDSPoint::new(Point3::new(5., 2., 3.), 0.25));
    explorer.next();
    assert_eq!(explorer.index(), 3);
    explorer.next();
    assert_eq!(explorer.index(), 4);
    explorer.next();
    assert!(!explorer.more());
    assert!(explorer.is_point(5));
    assert_eq!(explorer.nb_point(), 4);
    assert_eq!(explorer.index(), 6);
}

#[test]
fn cleared_point_source_retains_found_until_next() {
    let mut source = point_source();
    let mut explorer = TopOpeBRepDSPointExplorer::new_mut(&mut source, false);
    let source = explorer.source_mut().unwrap();
    source.init();
    assert!(explorer.more());
    assert_eq!(
        explorer.point().unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    assert_eq!(explorer.point_at(1).point(), Point3::default());
    explorer.next();
    assert!(!explorer.more());
    assert_eq!(explorer.index(), 5);
    assert_eq!(explorer.nb_point(), 0);
    assert_eq!(explorer.index(), 1);
}
