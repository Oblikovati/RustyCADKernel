//! Exact endpoint projection seeds short native overlap ranges.
use super::{BeanFaceIntersector, Failure, empty_range};

pub(super) fn compute(bean: &mut BeanFaceIntersector) -> Result<(), Failure> {
    let (mut u, mut v) = (bean.bounds[0], bean.bounds[2]);
    scan_initial_boundaries(bean, &mut u, &mut v)?;
    scan_final_boundary(bean, &mut u, &mut v)
}

fn scan_initial_boundaries(
    bean: &mut BeanFaceIntersector,
    u: &mut f64,
    v: &mut f64,
) -> Result<(), Failure> {
    let mut index = 1;
    while index <= bean.range_manager.length() {
        if should_skip_range(bean, index)? {
            index += 1;
            continue;
        }
        seed_near_first(bean, index, u, v)?;
        index += 1;
    }
    Ok(())
}

fn should_skip_range(bean: &BeanFaceIntersector, index: i32) -> Result<bool, Failure> {
    if bean.range_manager.flag(index)? > 0 {
        return Ok(true);
    }
    Ok(index > 1 && bean.range_manager.flag(index - 1)? > 0)
}

fn seed_near_first(
    bean: &mut BeanFaceIntersector,
    index: i32,
    u: &mut f64,
    v: &mut f64,
) -> Result<(), Failure> {
    let range = bean.range_manager.range(index)?;
    if bean.distance_uv(range.first(), u, v)? >= bean.criteria {
        return Ok(());
    }
    let count_before = bean.range_manager.length();
    if index > 1 {
        bean.compute_range_from_index(false, range.first(), *u, *v, index - 1)?;
    }
    let next_index = index + bean.range_manager.length() - count_before;
    bean.compute_range_from_index(true, range.first(), *u, *v, next_index)?;
    set_empty_if_unchanged(bean, count_before, range.first())
}

fn scan_final_boundary(
    bean: &mut BeanFaceIntersector,
    u: &mut f64,
    v: &mut f64,
) -> Result<(), Failure> {
    let last_index = bean.range_manager.length();
    if bean.range_manager.flag(last_index)? != 0 {
        return Ok(());
    }
    seed_near_last(bean, last_index, u, v)
}

fn seed_near_last(
    bean: &mut BeanFaceIntersector,
    index: i32,
    u: &mut f64,
    v: &mut f64,
) -> Result<(), Failure> {
    let range = bean.range_manager.range(index)?;
    if bean.distance_uv(range.last(), u, v)? >= bean.criteria {
        return Ok(());
    }
    let count_before = bean.range_manager.length();
    bean.compute_range_from_index(false, range.last(), *u, *v, index)?;
    set_empty_if_unchanged(bean, count_before, range.last())
}

fn set_empty_if_unchanged(
    bean: &mut BeanFaceIntersector,
    count_before: i32,
    parameter: f64,
) -> Result<(), Failure> {
    if bean.range_manager.length() == count_before {
        empty_range::set_empty_result(parameter, &mut bean.range_manager)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "near_range_boundaries_tests.rs"]
mod tests;
