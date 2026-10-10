//! Native tolerance traces and paired ranking predicates.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkv3d::select_mgr::{SelectMgrSortCriterion, SelectMgrToleranceMap};
fn tolerance_state(map: &SelectMgrToleranceMap) {
    int(map.tolerance());
    int(map.custom_tolerance());
    flag(map.is_custom_tol_set());
}
/// Execute count/custom updates, preserving state after a failing decrement; `tolerance_case(&values)`.
pub fn tolerance_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut map = SelectMgrToleranceMap::new();
    tolerance_state(&map);
    for _ in 0..r.integer() {
        let (operation, tolerance) = (r.integer(), r.integer());
        match operation {
            0 => map.add(tolerance),
            1 => attempt(|| map.decrement(tolerance)),
            2 => map.set_custom_tolerance(tolerance),
            3 => map.reset_defaults(),
            _ => panic!("tolerance operation={operation}; expected 0..=3"),
        }
        tolerance_state(&map);
    }
    end();
}
fn criterion_read(r: &mut Reader<'_>) -> SelectMgrSortCriterion {
    SelectMgrSortCriterion {
        normal: Vec3::new(r.real() as f32, r.real() as f32, r.real() as f32),
        depth: r.real(),
        min_dist: r.real(),
        tolerance: r.real(),
        selection_priority: r.integer(),
        display_priority: r.integer(),
        z_layer_position: r.integer(),
        nb_owner_matches: r.integer(),
        is_prefer_priority: r.flag(),
        ..Default::default()
    }
}
pub(super) fn criterion_state(criterion: &SelectMgrSortCriterion) {
    flag(criterion.entity.is_null());
    pnt(criterion.point);
    for value in [criterion.normal.x, criterion.normal.y, criterion.normal.z] {
        real(f64::from(value));
    }
    real(criterion.depth);
    real(criterion.min_dist);
    real(criterion.tolerance);
    int(criterion.selection_priority);
    int(criterion.display_priority);
    int(criterion.z_layer_position);
    int(criterion.nb_owner_matches);
    flag(criterion.is_prefer_priority);
}
/// Compare both directions and both native predicates; `criterion_case(&values)`.
pub fn criterion_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (first, second) = if r.flag() {
        (criterion_read(&mut r), criterion_read(&mut r))
    } else {
        (SelectMgrSortCriterion::new(), SelectMgrSortCriterion::new())
    };
    criterion_state(&first);
    criterion_state(&second);
    attempt(|| {
        flag(first.is_closer_depth(&second)?);
        Ok(())
    });
    attempt(|| {
        flag(second.is_closer_depth(&first)?);
        Ok(())
    });
    flag(first.is_higher_priority(&second));
    flag(second.is_higher_priority(&first));
    end();
}
