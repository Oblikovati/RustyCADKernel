//! Extrema seed edge intervals whose closest point is near the face.
use super::{BeanFaceIntersector, Failure, Range, empty_range};
use rustycad_tkbrep::{brep::BRepTool, topods::Edge};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg3d::{geom::GeomCurve, geom_adaptor::GeomAdaptorCurve};
use rustycad_tkgeombase::extrema::ExtCS;

pub(super) fn compute(bean: &mut BeanFaceIntersector) -> Result<(), Failure> {
    let edge: Edge = bean.curve.get().expect("bean curve").edge();
    let curve = BRepTool::curve_with_location(&edge)?.curve;
    let periodic = curve.get().expect("bean edge curve").is_periodic();
    process_ranges(bean, curve, periodic)
}

#[derive(Clone, Copy)]
struct Extremum {
    square_distance: f64,
    seed: Option<(f64, f64, f64)>,
}

struct ExtremaSnapshot {
    parallel: bool,
    values: Vec<Extremum>,
}

fn process_ranges(
    bean: &mut BeanFaceIntersector,
    curve: Handle<dyn GeomCurve>,
    periodic: bool,
) -> Result<(), Failure> {
    let mut index = 1;
    while index <= bean.range_manager.length() {
        index = process_range_at(bean, &curve, periodic, index)?;
    }
    Ok(())
}

fn process_range_at(
    bean: &mut BeanFaceIntersector,
    curve: &Handle<dyn GeomCurve>,
    periodic: bool,
    index: i32,
) -> Result<i32, Failure> {
    if bean.range_manager.flag(index)? > 0 {
        return Ok(index + 1);
    }
    let old_count = bean.range_manager.length();
    let bypass_delta = process_unmarked_range(bean, curve, periodic, index)?;
    let added = if bypass_delta {
        0
    } else {
        bean.range_manager.length() - old_count
    };
    Ok(index + added.max(0) + 1)
}

fn process_unmarked_range(
    bean: &mut BeanFaceIntersector,
    curve: &Handle<dyn GeomCurve>,
    periodic: bool,
    index: i32,
) -> Result<bool, Failure> {
    let range = bean.range_manager.range(index)?;
    if short_neighbor_range(bean, index, range)? {
        return Ok(false);
    }
    let extrema = snapshot_range_extrema(bean, curve, periodic, range)?;
    let Some(extrema) = extrema else {
        return Ok(false);
    };
    process_extrema_snapshot(bean, &extrema, index)
}

fn short_neighbor_range(
    bean: &mut BeanFaceIntersector,
    index: i32,
    range: Range,
) -> Result<bool, Failure> {
    if range.last() - range.first() >= precision::p_confusion() {
        return Ok(false);
    }
    let previous = index > 1 && bean.range_manager.flag(index - 1)? == 2;
    let next = index < bean.range_manager.length() && bean.range_manager.flag(index + 1)? == 2;
    if previous || next {
        bean.range_manager.set_flag(index, 1)?;
        return Ok(true);
    }
    Ok(false)
}

fn snapshot_range_extrema(
    bean: &BeanFaceIntersector,
    curve: &Handle<dyn GeomCurve>,
    periodic: bool,
    range: Range,
) -> Result<Option<ExtremaSnapshot>, Failure> {
    let mut extrema = initialized_extrema(bean)?;
    if periodic || inside_curve_domain(curve, range) {
        perform_extrema(&mut extrema, curve, range)?;
    }
    snapshot_extrema(&extrema)
}

fn initialized_extrema(bean: &BeanFaceIntersector) -> Result<ExtCS<'_>, Failure> {
    let mut extrema = ExtCS::new();
    extrema.initialize_range(
        bean.surface.get().expect("bean surface"),
        (bean.bounds[0], bean.bounds[1]),
        (bean.bounds[2], bean.bounds[3]),
        (precision::p_confusion(), precision::p_confusion()),
    )?;
    Ok(extrema)
}

fn inside_curve_domain(curve: &Handle<dyn GeomCurve>, range: Range) -> bool {
    let curve = curve.get().expect("bean edge curve");
    range.first() >= curve.first_parameter() - precision::p_confusion()
        && range.last() <= curve.last_parameter() + precision::p_confusion()
}

fn perform_extrema(
    extrema: &mut ExtCS<'_>,
    curve: &Handle<dyn GeomCurve>,
    range: Range,
) -> Result<(), Failure> {
    let adaptor = GeomAdaptorCurve::with_range(curve, range.first(), range.last())?;
    extrema.perform(&adaptor, range.first(), range.last())
}

fn snapshot_extrema(extrema: &ExtCS<'_>) -> Result<Option<ExtremaSnapshot>, Failure> {
    if !extrema.is_done() {
        return Ok(None);
    }
    let parallel = extrema.is_parallel()?;
    let values = collect_extrema(extrema, parallel)?;
    if values.is_empty() && !parallel {
        return Ok(None);
    }
    Ok(Some(ExtremaSnapshot { parallel, values }))
}

fn collect_extrema(extrema: &ExtCS<'_>, parallel: bool) -> Result<Vec<Extremum>, Failure> {
    let count = if parallel { 1 } else { extrema.nb_ext()? };
    let mut values = Vec::with_capacity(count as usize);
    for index in 1..=count {
        values.push(snapshot_extremum(extrema, parallel, index)?);
    }
    Ok(values)
}

fn snapshot_extremum(extrema: &ExtCS<'_>, parallel: bool, index: i32) -> Result<Extremum, Failure> {
    let square_distance = extrema.square_distance(index)?;
    let seed = if parallel {
        None
    } else {
        Some(extremum_seed(extrema, index)?)
    };
    Ok(Extremum {
        square_distance,
        seed,
    })
}

fn extremum_seed(extrema: &ExtCS<'_>, index: i32) -> Result<(f64, f64, f64), Failure> {
    let (curve, surface) = extrema.points(index)?;
    let (u, v) = surface.parameter();
    Ok((curve.parameter(), u, v))
}

fn process_extrema_snapshot(
    bean: &mut BeanFaceIntersector,
    extrema: &ExtremaSnapshot,
    index: i32,
) -> Result<bool, Failure> {
    if extrema.parallel {
        return process_parallel_extremum(bean, extrema.values[0], index);
    }
    process_ordinary_extrema(bean, &extrema.values, index)?;
    Ok(false)
}

fn process_parallel_extremum(
    bean: &mut BeanFaceIntersector,
    extremum: Extremum,
    index: i32,
) -> Result<bool, Failure> {
    update_minimum(bean, extremum.square_distance);
    if extremum.square_distance >= bean.criteria * bean.criteria {
        bean.range_manager.set_flag(index, 1)?;
        return Ok(false);
    }
    process_parallel_candidate(bean, index)
}

fn process_parallel_candidate(bean: &mut BeanFaceIntersector, index: i32) -> Result<bool, Failure> {
    let range = bean.range_manager.range(index)?;
    let first = endpoint_distance(bean, range.first())?;
    let last = endpoint_distance(bean, range.last())?;
    process_parallel_endpoints(bean, index, range, first, last)
}

#[derive(Clone, Copy)]
struct EndpointProjection {
    distance: f64,
    u: f64,
    v: f64,
}

fn endpoint_distance(
    bean: &BeanFaceIntersector,
    parameter: f64,
) -> Result<EndpointProjection, Failure> {
    let (mut u, mut v) = (0., 0.);
    let distance = bean.distance_uv(parameter, &mut u, &mut v)?;
    Ok(EndpointProjection { distance, u, v })
}

fn process_parallel_endpoints(
    bean: &mut BeanFaceIntersector,
    index: i32,
    range: Range,
    first: EndpointProjection,
    last: EndpointProjection,
) -> Result<bool, Failure> {
    let first_valid = first.distance < bean.criteria;
    let last_valid = last.distance < bean.criteria;
    match (first_valid, last_valid) {
        (true, true) => insert_parallel_range(bean, range),
        (true, false) => walk_parallel_endpoint(bean, true, range.first(), first),
        (false, true) => walk_parallel_endpoint(bean, false, range.last(), last),
        (false, false) => bisect_parallel_range(bean, index, range),
    }
}

fn insert_parallel_range(bean: &mut BeanFaceIntersector, range: Range) -> Result<bool, Failure> {
    bean.range_manager
        .insert_range(range.first(), range.last(), 2)?;
    Ok(true)
}

fn walk_parallel_endpoint(
    bean: &mut BeanFaceIntersector,
    increasing: bool,
    parameter: f64,
    projection: EndpointProjection,
) -> Result<bool, Failure> {
    bean.compute_range_from_start_point(increasing, parameter, projection.u, projection.v)?;
    Ok(false)
}

fn bisect_parallel_range(
    bean: &mut BeanFaceIntersector,
    index: i32,
    range: Range,
) -> Result<bool, Failure> {
    let mut search = ParallelSearch::new(
        range,
        bean.distance(range.first())?,
        bean.distance(range.last())?,
    );
    while search.width() > bean.curve_resolution {
        if search_midpoint(bean, &mut search)? {
            return expand_parallel_seed(bean, search.solution);
        }
    }
    bean.range_manager.set_flag(index, 1)?;
    Ok(false)
}

struct ParallelSearch {
    first: f64,
    last: f64,
    first_distance: f64,
    last_distance: f64,
    solution: f64,
}

impl ParallelSearch {
    fn new(range: Range, first_distance: f64, last_distance: f64) -> Self {
        Self {
            first: range.first(),
            last: range.last(),
            first_distance,
            last_distance,
            solution: range.first(),
        }
    }

    fn width(&self) -> f64 {
        self.last - self.first
    }

    fn discard_farther_half(&mut self, middle: f64, distance: f64) {
        if self.first_distance < self.last_distance {
            self.last = middle;
            self.last_distance = distance;
            return;
        }
        self.first = middle;
        self.first_distance = distance;
    }
}

fn search_midpoint(
    bean: &BeanFaceIntersector,
    search: &mut ParallelSearch,
) -> Result<bool, Failure> {
    let middle = (search.first + search.last) * 0.5;
    let distance = bean.distance(middle)?;
    if distance < bean.criteria {
        search.solution = middle;
        return Ok(true);
    }
    search.discard_farther_half(middle, distance);
    Ok(false)
}

fn expand_parallel_seed(bean: &mut BeanFaceIntersector, parameter: f64) -> Result<bool, Failure> {
    let projection = endpoint_distance(bean, parameter)?;
    bean.compute_range_from_start_point(false, parameter, projection.u, projection.v)?;
    bean.compute_range_from_start_point(true, parameter, projection.u, projection.v)?;
    Ok(false)
}

fn process_ordinary_extrema(
    bean: &mut BeanFaceIntersector,
    extrema: &[Extremum],
    index: i32,
) -> Result<(), Failure> {
    let mut found = false;
    for extremum in extrema {
        found |= process_ordinary_extremum(bean, *extremum)?;
    }
    if !found {
        bean.range_manager.set_flag(index, 1)?;
    }
    Ok(())
}

fn process_ordinary_extremum(
    bean: &mut BeanFaceIntersector,
    extremum: Extremum,
) -> Result<bool, Failure> {
    if extremum.square_distance < bean.criteria * bean.criteria {
        seed_ordinary_extremum(bean, extremum)?;
        return Ok(true);
    }
    update_minimum(bean, extremum.square_distance);
    Ok(false)
}

fn seed_ordinary_extremum(
    bean: &mut BeanFaceIntersector,
    extremum: Extremum,
) -> Result<(), Failure> {
    let (parameter, u, v) = extremum.seed.expect("ordinary extremum has a seed");
    let count_before = bean.range_manager.length();
    bean.compute_range_from_start_point(false, parameter, u, v)?;
    bean.compute_range_from_start_point(true, parameter, u, v)?;
    if bean.range_manager.length() == count_before {
        empty_range::set_empty_result(parameter, &mut bean.range_manager)?;
    }
    update_minimum(bean, extremum.square_distance);
    Ok(())
}

fn update_minimum(bean: &mut BeanFaceIntersector, distance: f64) {
    bean.minimum_square_distance = native_min(bean.minimum_square_distance, distance);
}

fn native_min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}

#[cfg(test)]
#[path = "extremum_tests.rs"]
mod tests;
