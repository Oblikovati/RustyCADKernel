//! Native adaptive box pruning for bounded Bezier and high-degree spline faces.
use super::{BeanFaceIntersector, Failure, boxes, grid, near_range_boundaries, ranges};
use crate::int_tools::{
    CurveRangeLocalizeData, CurveRangeSample, SurfaceRangeLocalizeData, SurfaceRangeSample,
};
use rustycad_tkernel::precision;
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dSurface},
    geom::GeomBSplineSurface,
};
use rustycad_tkgeombase::{
    bnd_lib::{Add3dCurve, AddSurface},
    extrema::GenExtCS,
};
use rustycad_tkmath::{bnd::BoundingBox3, elclib};

mod localize_solutions;

pub(super) fn compute(bean: &mut BeanFaceIntersector) -> Result<bool, Failure> {
    let cached = surface_cache(bean)?;
    let mut surface_data = cached.write();
    prepare_surface_data(bean, &mut surface_data)?;
    let (surface_box, curve_box) = compute_root_boxes(bean, &mut surface_data)?;
    if curve_box.is_out_box(&surface_box) {
        mark_all_ranges_out(bean)?;
        surface_data.clear_grid();
        return Ok(true);
    }
    compute_localized_ranges(bean, &mut surface_data, &curve_box, &surface_box)
}

fn surface_cache(
    bean: &BeanFaceIntersector,
) -> Result<crate::int_tools::Cached<SurfaceRangeLocalizeData>, Failure> {
    let context = bean.context.get().expect("bean context");
    context.surface_data(&bean.surface.get().expect("bean surface").face())
}

fn prepare_surface_data(
    bean: &BeanFaceIntersector,
    cache: &mut SurfaceRangeLocalizeData,
) -> Result<(), Failure> {
    cache.remove_range_out_all();
    cache.clear_grid();
    if bean.surface.get().expect("bean surface").get_type()?
        != rustycad_tkmath::geom_abs::SurfaceType::BSplineSurface
    {
        return Ok(());
    }
    compute_bspline_grid(bean, cache)
}

fn compute_bspline_grid(
    bean: &BeanFaceIntersector,
    cache: &mut SurfaceRangeLocalizeData,
) -> Result<(), Failure> {
    let surface = bean.surface.get().expect("bean surface");
    let geometry = bean.transformed_surface.cast::<GeomBSplineSurface>();
    let spline = geometry.get().ok_or_else(|| {
        Failure::new(
            "BeanFace localized B-spline surface cast failed; expected Geom_BSplineSurface",
        )
    })?;
    grid::compute_grid(surface, spline, bean.bounds, bean.tolerances[1], cache)
}

fn compute_root_boxes(
    bean: &BeanFaceIntersector,
    cache: &mut SurfaceRangeLocalizeData,
) -> Result<(BoundingBox3, BoundingBox3), Failure> {
    let surface_box = cached_surface_box(bean, cache)?;
    let curve_box = bounded_curve_box(bean)?;
    Ok((surface_box, curve_box))
}

fn cached_surface_box(
    bean: &BeanFaceIntersector,
    cache: &mut SurfaceRangeLocalizeData,
) -> Result<BoundingBox3, Failure> {
    let root = SurfaceRangeSample::default();
    let mut bounds = BoundingBox3::default();
    if cache.find_box(&root, &mut bounds) {
        return Ok(bounds);
    }
    compute_surface_box(bean, cache, &mut bounds)?;
    cache.add_box(&root, &bounds);
    Ok(bounds)
}

fn compute_surface_box(
    bean: &BeanFaceIntersector,
    cache: &mut SurfaceRangeLocalizeData,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let surface = bean.surface.get().expect("bean surface");
    if surface.get_type()? == rustycad_tkmath::geom_abs::SurfaceType::BSplineSurface {
        *bounds = boxes::surface_box(surface, bean.bounds, bean.criteria, cache)?;
        return Ok(());
    }
    AddSurface::add_range(surface, bean.bounds, bean.tolerances[1], bounds)
}

fn bounded_curve_box(bean: &BeanFaceIntersector) -> Result<BoundingBox3, Failure> {
    let curve = bean.curve.get().expect("bean curve");
    let trimmed = curve.trim(
        bean.range.first(),
        bean.range.last(),
        precision::p_confusion(),
    )?;
    let mut bounds = BoundingBox3::default();
    Add3dCurve::add(
        trimmed.get().expect("trimmed bean curve"),
        bean.tolerances[0],
        &mut bounds,
    )?;
    Ok(bounds)
}

fn compute_localized_ranges(
    bean: &mut BeanFaceIntersector,
    cache: &mut SurfaceRangeLocalizeData,
    curve_box: &BoundingBox3,
    surface_box: &BoundingBox3,
) -> Result<bool, Failure> {
    let mut curve_data = CurveRangeLocalizeData::new(3, 10. * bean.curve_resolution);
    curve_data.add_box(&CurveRangeSample::new(0), curve_box);
    let mut solutions = Vec::new();
    let localized = localize_solutions::compute(
        bean,
        &mut curve_data,
        cache,
        curve_box,
        surface_box,
        &mut solutions,
    )?;
    if !localized {
        cache.clear_grid();
        return Ok(false);
    }
    evaluate_localized_solutions(bean, cache, &solutions)?;
    insert_curve_ranges(bean, &curve_data)?;
    near_range_boundaries::compute(bean)?;
    cache.clear_grid();
    Ok(true)
}

fn evaluate_localized_solutions(
    bean: &mut BeanFaceIntersector,
    cache: &SurfaceRangeLocalizeData,
    solutions: &[(CurveRangeSample, SurfaceRangeSample)],
) -> Result<(), Failure> {
    let (curves, surfaces) = sorted_samples(solutions)?;
    evaluate_sorted_solutions(bean, cache, &curves, &surfaces)
}

fn sorted_samples(
    solutions: &[(CurveRangeSample, SurfaceRangeSample)],
) -> Result<(Vec<CurveRangeSample>, Vec<SurfaceRangeSample>), Failure> {
    let (curves, surfaces): (Vec<_>, Vec<_>) = solutions.iter().copied().unzip();
    let mut sorted_curves = Vec::new();
    let mut sorted_surfaces = Vec::new();
    ranges::merge_samples(&curves, &surfaces, &mut sorted_curves, &mut sorted_surfaces)?;
    Ok((sorted_curves, sorted_surfaces))
}

fn evaluate_sorted_solutions(
    bean: &mut BeanFaceIntersector,
    cache: &SurfaceRangeLocalizeData,
    curves: &[CurveRangeSample],
    surfaces: &[SurfaceRangeSample],
) -> Result<(), Failure> {
    let curve = bean.curve.get().expect("bean curve").shallow_copy()?;
    let surface = bean.surface.get().expect("bean surface").shallow_copy()?;
    let mut extrema = GenExtCS::new();
    let mut previous = SurfaceRangeSample::default();
    for (&curve_sample, &surface_sample) in curves.iter().zip(surfaces) {
        let ranges = sampled_ranges(bean, cache, curve_sample, surface_sample);
        initialize_or_reuse_extrema(
            &mut extrema,
            surface.get().expect("bean surface copy"),
            &mut previous,
            surface_sample,
            &ranges[1..],
        )?;
        evaluate_curve_sample(
            bean,
            &mut extrema,
            curve.get().expect("bean curve copy"),
            ranges[0],
        )?;
    }
    Ok(())
}

fn sampled_ranges(
    bean: &BeanFaceIntersector,
    cache: &SurfaceRangeLocalizeData,
    curve: CurveRangeSample,
    surface: SurfaceRangeSample,
) -> [super::Range; 3] {
    let curve_range = curve.get_range(bean.range.first(), bean.range.last(), 3);
    let u_range = surface.get_range_u(bean.bounds[0], bean.bounds[1], cache.get_nb_sample_u());
    let v_range = surface.get_range_v(bean.bounds[2], bean.bounds[3], cache.get_nb_sample_v());
    [curve_range, u_range, v_range]
}

fn initialize_or_reuse_extrema<'a>(
    extrema: &mut GenExtCS<'a>,
    surface: &'a dyn Adaptor3dSurface,
    previous: &mut SurfaceRangeSample,
    sample: SurfaceRangeSample,
    uv_ranges: &[super::Range],
) -> Result<(), Failure> {
    if previous.is_equal(&sample) {
        return Ok(());
    }
    extrema.initialize_range(
        surface,
        (10, 10),
        (uv_ranges[0].first(), uv_ranges[0].last()),
        (uv_ranges[1].first(), uv_ranges[1].last()),
        precision::p_confusion(),
    )?;
    *previous = sample;
    Ok(())
}

fn evaluate_curve_sample<'a>(
    bean: &mut BeanFaceIntersector,
    extrema: &mut GenExtCS<'a>,
    curve: &'a dyn Adaptor3dCurve,
    range: super::Range,
) -> Result<(), Failure> {
    extrema.perform_range(
        curve,
        10,
        (range.first(), range.last()),
        precision::p_confusion(),
    )?;
    handle_extrema(bean, extrema, range)
}

fn handle_extrema(
    bean: &mut BeanFaceIntersector,
    extrema: &GenExtCS<'_>,
    range: super::Range,
) -> Result<(), Failure> {
    if extrema.is_done() && extrema.nb_ext()? > 0 {
        return seed_localized_extrema(bean, extrema);
    }
    bean.range_manager
        .insert_range(range.first(), range.last(), 0)?;
    Ok(())
}

fn seed_localized_extrema(
    bean: &mut BeanFaceIntersector,
    extrema: &GenExtCS<'_>,
) -> Result<(), Failure> {
    for index in 1..=extrema.nb_ext()? {
        if extrema.square_distance(index)? < bean.criteria * bean.criteria {
            seed_localized_extremum(bean, extrema, index)?;
        }
    }
    Ok(())
}

fn seed_localized_extremum(
    bean: &mut BeanFaceIntersector,
    extrema: &GenExtCS<'_>,
    index: i32,
) -> Result<(), Failure> {
    let parameter = extrema.point_on_curve(index)?.parameter();
    let (u, v) = extrema.point_on_surface(index)?.parameter();
    let (parameter, u, v) = periodic_parameters(bean, parameter, u, v)?;
    let before = bean.range_manager.length();
    bean.compute_range_from_start_point(false, parameter, u, v)?;
    bean.compute_range_from_start_point(true, parameter, u, v)?;
    if before == bean.range_manager.length() {
        ranges::set_empty_result(parameter, &mut bean.range_manager)?;
    }
    Ok(())
}

fn periodic_parameters(
    bean: &BeanFaceIntersector,
    mut parameter: f64,
    mut u: f64,
    mut v: f64,
) -> Result<(f64, f64, f64), Failure> {
    let curve = bean.curve.get().expect("bean curve");
    let surface = bean.surface.get().expect("bean surface");
    if curve.is_periodic()? {
        parameter = elclib::in_period(
            parameter,
            bean.range.first(),
            bean.range.first() + curve.period()?,
        );
    }
    if surface.is_u_periodic()? {
        u = elclib::in_period(u, bean.bounds[0], bean.bounds[0] + surface.u_period()?);
    }
    if surface.is_v_periodic()? {
        v = elclib::in_period(v, bean.bounds[2], bean.bounds[2] + surface.v_period()?);
    }
    Ok((
        parameter,
        clamp(u, bean.bounds[0], bean.bounds[1]),
        clamp(v, bean.bounds[2], bean.bounds[3]),
    ))
}

fn clamp(value: f64, lower: f64, upper: f64) -> f64 {
    if value < lower {
        return lower;
    }
    if value > upper {
        return upper;
    }
    value
}

fn insert_curve_ranges(
    bean: &mut BeanFaceIntersector,
    cache: &CurveRangeLocalizeData,
) -> Result<(), Failure> {
    let mut ranges = Vec::new();
    cache.list_range_out(&mut ranges);
    for sample in ranges {
        let range = sample.get_range(bean.range.first(), bean.range.last(), 3);
        bean.range_manager
            .insert_range(range.first(), range.last(), 1)?;
    }
    Ok(())
}

fn mark_all_ranges_out(bean: &mut BeanFaceIntersector) -> Result<(), Failure> {
    for index in 1..=bean.range_manager.length() {
        bean.range_manager.set_flag(index, 1)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
