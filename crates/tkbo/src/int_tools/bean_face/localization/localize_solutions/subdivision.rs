//! Parameter partition helpers shared by the native-style box traversal.
use super::super::ranges;
use super::super::{BeanFaceIntersector, Failure};
use crate::int_tools::{
    CurveRangeLocalizeData, CurveRangeSample, Range, SurfaceRangeLocalizeData, SurfaceRangeSample,
};

pub(super) struct PatchSubdivision {
    pub(super) curve: Range,
    pub(super) u: Range,
    pub(super) v: Range,
    pub(super) steps: [f64; 3],
    pub(super) indexes: [i32; 3],
    pub(super) allow: [bool; 3],
    curve_template: CurveRangeSample,
    surface_template: SurfaceRangeSample,
    pub(super) counts: [i32; 3],
    pub(super) source_depths: [i32; 3],
}

pub(super) fn subdivision(
    bean: &BeanFaceIntersector,
    curve_cache: &CurveRangeLocalizeData,
    surface_cache: &SurfaceRangeLocalizeData,
    curve: CurveRangeSample,
    surface: SurfaceRangeSample,
) -> Result<PatchSubdivision, Failure> {
    let ranges = [
        curve.get_range(
            bean.range.first(),
            bean.range.last(),
            curve_cache.get_nb_sample(),
        ),
        surface.get_range_u(
            bean.bounds[0],
            bean.bounds[1],
            surface_cache.get_nb_sample_u(),
        ),
        surface.get_range_v(
            bean.bounds[2],
            bean.bounds[3],
            surface_cache.get_nb_sample_v(),
        ),
    ];
    let counts = [
        curve_cache.get_nb_sample(),
        surface_cache.get_nb_sample_u(),
        surface_cache.get_nb_sample_v(),
    ];
    let steps = std::array::from_fn(|axis| ranges[axis].length() / f64::from(counts[axis]));
    let allow = ranges::check_sampling(curve, surface, curve_cache, surface_cache, steps);
    Ok(make_templates(curve, surface, ranges, steps, allow, counts))
}

fn make_templates(
    curve: CurveRangeSample,
    surface: SurfaceRangeSample,
    intervals: [Range; 3],
    steps: [f64; 3],
    allow: [bool; 3],
    counts: [i32; 3],
) -> PatchSubdivision {
    let curve_index = if allow[0] {
        curve.get_range_index_deeper(counts[0])
    } else {
        curve.get_range_index()
    };
    let mut curve_template = curve;
    if allow[0] {
        curve_template.set_depth(curve.get_depth().wrapping_add(1));
    }
    curve_template.set_range_index(curve_index);
    let (surface_template, u_index, v_index) = make_surface_template(surface, counts, allow);
    PatchSubdivision {
        curve: intervals[0],
        u: intervals[1],
        v: intervals[2],
        steps: std::array::from_fn(|axis| {
            if allow[axis] {
                steps[axis]
            } else {
                intervals[axis].length()
            }
        }),
        indexes: [curve_index, u_index, v_index],
        allow,
        curve_template,
        surface_template,
        counts: std::array::from_fn(|axis| if allow[axis] { counts[axis] } else { 1 }),
        source_depths: [
            curve.get_depth(),
            surface.get_depth_u(),
            surface.get_depth_v(),
        ],
    }
}

fn make_surface_template(
    surface: SurfaceRangeSample,
    counts: [i32; 3],
    allow: [bool; 3],
) -> (SurfaceRangeSample, i32, i32) {
    let (mut u, mut v) = surface.get_ranges();
    let u_index = if allow[1] {
        u.get_range_index_deeper(counts[1])
    } else {
        u.get_range_index()
    };
    let v_index = if allow[2] {
        v.get_range_index_deeper(counts[2])
    } else {
        v.get_range_index()
    };
    if allow[1] {
        let depth = u.get_depth().wrapping_add(1);
        u.set_depth(depth);
    }
    if allow[2] {
        let depth = v.get_depth().wrapping_add(1);
        v.set_depth(depth);
    }
    u.set_range_index(u_index);
    v.set_range_index(v_index);
    (SurfaceRangeSample::from_ranges(u, v), u_index, v_index)
}

pub(super) fn surface_cell(
    state: &PatchSubdivision,
    original: SurfaceRangeSample,
    u: i32,
    v: i32,
) -> SurfaceRangeSample {
    let mut sample = state.surface_template;
    sample.set_index_u(if state.allow[1] {
        state.indexes[1] + u - 1
    } else {
        original.get_index_u()
    });
    sample.set_index_v(if state.allow[2] {
        state.indexes[2] + v - 1
    } else {
        original.get_index_v()
    });
    sample
}

pub(super) fn curve_cell(
    state: &PatchSubdivision,
    original: CurveRangeSample,
    cell: i32,
) -> CurveRangeSample {
    let mut sample = state.curve_template;
    sample.set_range_index(if state.allow[0] {
        state.indexes[0] + cell - 1
    } else {
        original.get_range_index()
    });
    sample
}

pub(super) fn cell_bounds(range: Range, step: f64, cell: i32) -> (f64, f64) {
    let first = range.first() + f64::from(cell - 1) * step;
    (first, first + step)
}

impl Range {
    fn length(self) -> f64 {
        self.last() - self.first()
    }
}
