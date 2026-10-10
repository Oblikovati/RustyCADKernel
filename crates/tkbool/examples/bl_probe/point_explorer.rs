//! Counterpart of tools/oracle/tkbool_probe_point_explorer.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSDataStructure, TopOpeBRepDSPoint, TopOpeBRepDSPointExplorer,
};
use rustycad_tkmath::gp::Point3;

fn explorer_point(point: &TopOpeBRepDSPoint) {
    pnt(point.point());
    flag(point.keep());
}

fn point_source(count: i32, mask: i32) -> TopOpeBRepDSDataStructure {
    let mut source = TopOpeBRepDSDataStructure::new();
    for slot in 0..count {
        let mut point = TopOpeBRepDSPoint::new(
            Point3::new(f64::from(slot + 1), 2., 3.),
            0.25 * f64::from(slot + 1),
        );
        point.change_keep(mask & (1 << slot) != 0);
        source.add_point(&point);
    }
    source
}

fn point_walk(explorer: &mut TopOpeBRepDSPointExplorer<'_>) {
    while explorer.more() {
        int(explorer.index());
        explorer_point(explorer.point().unwrap());
        real(explorer.point().unwrap().tolerance());
        explorer.next();
    }
    int(explorer.index());
    explorer_point(explorer.point().unwrap());
    int(explorer.nb_point());
    flag(explorer.more());
    int(explorer.index());
}

fn point_default(explorer: &mut TopOpeBRepDSPointExplorer<'_>) {
    flag(explorer.more());
    int(explorer.index());
    explorer_point(explorer.point().unwrap());
    explorer.next();
    int(explorer.index());
}

/// Exercises binding, filtering and recount; `point_explorer_case(&[4., 5., 1., 2.])`.
pub fn point_explorer_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let source = point_source(reader.integer(), reader.integer());
    let kept = reader.integer() != 0;
    let query = reader.integer();
    let mut explorer = TopOpeBRepDSPointExplorer::default();
    point_default(&mut explorer);
    explorer.init(&source, kept);
    flag(explorer.is_point(query));
    flag(explorer.is_point_keep(query));
    explorer_point(explorer.point_at(query));
    point_walk(&mut explorer);
    explorer.init(&source, !kept);
    point_walk(&mut explorer);
    end();
}

/// Compares live edits and stale-current failure after clearing; `point_explorer_mutation_case(&[])`.
pub fn point_explorer_mutation_case(_: &[f64]) {
    let mut source = point_source(2, 1);
    let mut explorer = TopOpeBRepDSPointExplorer::new_mut(&mut source, true);
    let editable = explorer.source_mut().unwrap();
    editable.change_keep_point(2, true).unwrap();
    editable.add_point(&TopOpeBRepDSPoint::new(Point3::new(3., 2., 3.), 0.75));
    flag(explorer.is_point(3));
    explorer_point(explorer.point_at(3));
    point_walk(&mut explorer);
    point_clear_source(&mut source);
    end();
}

fn point_clear_source(source: &mut TopOpeBRepDSDataStructure) {
    let mut explorer = TopOpeBRepDSPointExplorer::new_mut(source, true);
    explorer.source_mut().unwrap().init();
    flag(explorer.more());
    attempt(|| explorer.point().map(explorer_point));
    explorer_point(explorer.point_at(1));
    explorer.next();
    flag(explorer.more());
    int(explorer.index());
}
