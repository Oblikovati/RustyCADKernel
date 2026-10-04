//! General solver dispatch preserves the native localization gate and result coalescing.
use super::{Adaptor3dSurface, BeanFaceIntersector, Failure, Range, SurfaceType, precision};

pub(super) fn compute(bean: &mut BeanFaceIntersector) -> Result<(), Failure> {
    let localized = requires_localization(bean)? && super::localization::compute(bean)?;
    if !localized {
        super::exact_intersection::compute(bean)?;
        super::extremum::compute(bean)?;
        super::near_range_boundaries::compute(bean)?;
    }
    collect_results(bean)
}

pub(super) fn requires_localization(bean: &BeanFaceIntersector) -> Result<bool, Failure> {
    if bean
        .bounds
        .iter()
        .any(|value| precision::is_infinite(*value))
    {
        return Ok(false);
    }
    let surface = bean.surface.get().expect("bean surface");
    match surface.get_type()? {
        SurfaceType::BezierSurface | SurfaceType::OtherSurface => Ok(true),
        SurfaceType::BSplineSurface => Ok((surface.u_degree()? > 2 || surface.v_degree()? > 2)
            && surface.nb_u_knots()? > 2
            && surface.nb_v_knots()? > 2),
        _ => Ok(false),
    }
}

pub(super) fn collect_results(bean: &mut BeanFaceIntersector) -> Result<(), Failure> {
    bean.done = true;
    for index in 1..=bean.range_manager.length() {
        if bean.range_manager.flag(index)? == 2 {
            append_or_merge(bean, bean.range_manager.range(index)?);
        }
    }
    Ok(())
}

fn append_or_merge(bean: &mut BeanFaceIntersector, range: Range) {
    if let Some(last) = bean.results.last_mut() {
        // Native merges unless the gap is strictly greater, including unordered values.
        let gap = (range.first() - last.last()).abs();
        if gap.partial_cmp(&precision::p_confusion()) != Some(std::cmp::Ordering::Greater) {
            last.set_last(range.last());
            return;
        }
    }
    bean.results.append(range);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_collection_filters_flags_and_merges_adjacent_intervals() {
        let mut bean = BeanFaceIntersector::new();
        bean.range_manager.set_ranges(&[0., 1., 2., 3., 4.], 2);
        bean.range_manager.set_flag(3, 1).unwrap();
        collect_results(&mut bean).unwrap();
        assert!(bean.is_done());
        let bounds: Vec<_> = bean.result().iter().map(|range| range.range()).collect();
        assert_eq!(bounds, vec![(0., 2.), (3., 4.)]);
    }

    #[test]
    fn native_result_gap_boundary_merges_at_parameter_confusion() {
        let mut bean = BeanFaceIntersector::new();
        append_or_merge(&mut bean, Range::new(-1., 0.));
        append_or_merge(&mut bean, Range::new(precision::p_confusion(), 1.));
        append_or_merge(&mut bean, Range::new(2., 3.));
        assert_eq!(bean.result().length(), 2);
        assert_eq!(bean.result().first().unwrap().range(), (-1., 1.));
    }
}
