//! Counterpart of tools/oracle/tkbool_probe_curve_explorer.cxx.
use super::common::{Reader, end, flag, int, real};
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSCurveExplorer, TopOpeBRepDSDataStructure,
};

fn explorer_curve(curve: &TopOpeBRepDSCurve) {
    int(curve.ds_index());
    flag(curve.keep());
    real(curve.tolerance());
    flag(curve.curve().is_null());
}

fn explorer_walk(explorer: &mut TopOpeBRepDSCurveExplorer<'_>) {
    while explorer.more() {
        int(explorer.index());
        explorer_curve(explorer.curve());
        explorer.next();
    }
    int(explorer.index());
    explorer_curve(explorer.curve());
    int(explorer.nb_curve());
    flag(explorer.more());
    int(explorer.index());
}

fn explorer_source(count: i32, gap: i32, mask: i32) -> TopOpeBRepDSDataStructure {
    let mut curves = TopOpeBRepDSDataStructure::new();
    for slot in 0..count {
        if slot == 1 {
            curves.change_nb_curves(curves.nb_curves() + gap);
        }
        let mut curve = TopOpeBRepDSCurve::default();
        curve.set_tolerance(0.25 * f64::from(slot + 1));
        curve.change_keep(mask & (1 << slot) != 0);
        curves.add_curve(&mut curve);
    }
    curves.change_curve(-1).set_tolerance(12.0);
    curves
}

/// Compares filtered traversal and lookup; `curve_explorer_case(&[3., 2., 5., 1., 4.])`.
pub fn curve_explorer_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let curves = explorer_source(reader.integer(), reader.integer(), reader.integer());
    let only_kept = reader.integer() != 0;
    let query = reader.integer();
    let mut explorer = TopOpeBRepDSCurveExplorer::new(&curves, only_kept);
    flag(explorer.is_curve(query));
    flag(explorer.is_curve_keep(query));
    explorer_curve(explorer.curve_at(query));
    explorer_walk(&mut explorer);
    explorer.init(&curves, !only_kept);
    explorer_walk(&mut explorer);
    end();
}

/// Edits the borrowed source during traversal; `curve_explorer_mutation_case(&[2.])`.
pub fn curve_explorer_mutation_case(values: &[f64]) {
    let gap = Reader::new(values).integer();
    let mut curves = explorer_source(2, gap, 1);
    let mut explorer = TopOpeBRepDSCurveExplorer::new_mut(&mut curves, true);
    let source = explorer.source_mut().expect("exclusive explorer source");
    source.change_keep_curve(2 + gap, true);
    let index = source.add_curve(&mut TopOpeBRepDSCurve::default());
    flag(explorer.is_curve(index));
    explorer_curve(explorer.curve_at(index));
    explorer_walk(&mut explorer);
    explorer_clear_source(&mut curves);
    end();
}

fn explorer_clear_source(curves: &mut TopOpeBRepDSDataStructure) {
    let mut explorer = TopOpeBRepDSCurveExplorer::new_mut(curves, true);
    let source = explorer.source_mut().expect("exclusive explorer source");
    source.init();
    source.change_curve(-1).set_tolerance(12.0);
    flag(explorer.more());
    explorer_curve(explorer.curve());
    explorer_curve(explorer.curve_at(1));
    explorer.next();
    flag(explorer.more());
    int(explorer.index());
}
