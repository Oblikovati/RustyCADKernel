//! Exact curve-surface hits seed native BeanFace ranges before distance localization.
use super::{BeanFaceIntersector, Failure, empty_range};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeomalgo::int_curve_surface::{HInter, IntersectionPoint};

pub(super) fn compute(bean: &mut BeanFaceIntersector) -> Result<(), Failure> {
    let intersector = exact_intersector(bean)?;
    if !intersector.is_done() {
        return Ok(());
    }
    adjust_resolution_for_multiple_hits(bean, intersector.nb_points()?)?;
    process_exact_points(bean, &intersector)?;
    process_exact_segments(bean, &intersector)
}

fn exact_intersector(bean: &BeanFaceIntersector) -> Result<HInter, Failure> {
    let curve: rustycad_tkgeomalgo::int_curve_surface::HCurve = bean.curve.clone().cast();
    let surface = bean.surface.clone().cast::<dyn Adaptor3dSurface>();
    let mut intersector = HInter::new();
    intersector.perform(&curve, &surface)?;
    Ok(intersector)
}

fn adjust_resolution_for_multiple_hits(
    bean: &mut BeanFaceIntersector,
    point_count: i32,
) -> Result<(), Failure> {
    if point_count > 1 {
        bean.criteria = 3.0 * precision::confusion();
        bean.curve_resolution = bean
            .curve
            .get()
            .expect("bean curve")
            .resolution(bean.criteria)?;
    }
    Ok(())
}

fn process_exact_points(
    bean: &mut BeanFaceIntersector,
    intersector: &HInter,
) -> Result<(), Failure> {
    for index in 1..=intersector.nb_points()? {
        process_exact_point(bean, intersector.point(index)?)?;
    }
    Ok(())
}

fn process_exact_point(
    bean: &mut BeanFaceIntersector,
    point: IntersectionPoint,
) -> Result<(), Failure> {
    if point.w() < bean.range.first() || point.w() > bean.range.last() {
        return Ok(());
    }
    let Some((u, v)) = corrected_parameters(bean, point.u(), point.v())? else {
        return Ok(());
    };
    let count_before = bean.range_manager.length();
    bean.compute_range_from_start_point(false, point.w(), u, v)?;
    bean.compute_range_from_start_point(true, point.w(), u, v)?;
    record_exact_point(bean, count_before, point.w())
}

fn record_exact_point(
    bean: &mut BeanFaceIntersector,
    count_before: i32,
    parameter: f64,
) -> Result<(), Failure> {
    if count_before == bean.range_manager.length() {
        empty_range::set_empty_result(parameter, &mut bean.range_manager)?;
    } else {
        bean.minimum_square_distance = 0.0;
    }
    Ok(())
}

fn corrected_parameters(
    bean: &BeanFaceIntersector,
    u: f64,
    mut v: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    let [umin, umax, vmin, vmax] = bean.bounds;
    let (u, mut valid, u_corrected) = if inside(u, umin, umax) {
        (u, inside(v, vmin, vmax), true)
    } else {
        correct_u(bean, u)?
    };
    if u_corrected && !inside(v, vmin, vmax) {
        (v, valid) = correct_v(bean, v)?;
    }
    if !valid {
        return Ok(None);
    }
    Ok(Some((u, v)))
}

fn correct_u(bean: &BeanFaceIntersector, u: f64) -> Result<(f64, bool, bool), Failure> {
    let surface = bean.surface.get().expect("bean surface");
    if !surface.is_u_periodic()? {
        return Ok((u, false, false));
    }
    let adjusted = adjust_parameter(u, bean.bounds[0], bean.bounds[1], surface.u_period()?);
    Ok((adjusted, true, true))
}

fn correct_v(bean: &BeanFaceIntersector, v: f64) -> Result<(f64, bool), Failure> {
    let surface = bean.surface.get().expect("bean surface");
    if !surface.is_v_periodic()? {
        return Ok((v, false));
    }
    let adjusted = adjust_parameter(v, bean.bounds[2], bean.bounds[3], surface.v_period()?);
    Ok((adjusted, true))
}

fn adjust_parameter(value: f64, minimum: f64, maximum: f64, period: f64) -> f64 {
    rustycad_tkgeomalgo::geom_int::adjust_periodic(
        value,
        minimum,
        maximum,
        period,
        real::epsilon(period),
    )
    .1
}

fn process_exact_segments(
    bean: &mut BeanFaceIntersector,
    intersector: &HInter,
) -> Result<(), Failure> {
    for index in 1..=intersector.nb_segments()? {
        let segment = intersector.segment(index)?;
        let (first, last) = segment.values();
        record_exact_segment(bean, first, last)?;
    }
    Ok(())
}

fn record_exact_segment(
    bean: &mut BeanFaceIntersector,
    first: IntersectionPoint,
    last: IntersectionPoint,
) -> Result<(), Failure> {
    let start = native_max(bean.range.first(), first.w());
    let end = native_min(bean.range.last(), last.w());
    bean.range_manager.insert_range(start, end, 2)?;
    bean.compute_range_from_start_point(false, first.w(), first.u(), first.v())?;
    bean.compute_range_from_start_point(true, last.w(), last.u(), last.v())?;
    bean.minimum_square_distance = 0.0;
    Ok(())
}

fn inside(value: f64, minimum: f64, maximum: f64) -> bool {
    minimum <= value && value <= maximum
}

fn native_min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}

fn native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}

#[cfg(test)]
#[path = "exact_intersection_tests.rs"]
mod tests;
