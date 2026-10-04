//! Traverses the native U/V/C box partition and preserves its exclusion cache behavior.
use super::super::boxes;
use super::super::{BeanFaceIntersector, Failure};
use super::{
    SamplePair,
    subdivision::{self, PatchSubdivision, cell_bounds, curve_cell, surface_cell},
};
use crate::int_tools::{
    CurveRangeLocalizeData, CurveRangeSample, SurfaceRangeLocalizeData, SurfaceRangeSample,
};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::bnd_lib::{Add3dCurve, AddSurface};
use rustycad_tkmath::bnd::BoundingBox3;
struct Localizer<'a> {
    bean: &'a BeanFaceIntersector,
    curve_cache: &'a mut CurveRangeLocalizeData,
    surface_cache: &'a mut SurfaceRangeLocalizeData,
    curve_root: &'a BoundingBox3,
    surface_root: &'a BoundingBox3,
    output: &'a mut Vec<SamplePair>,
}

struct CurveCandidate {
    sample: CurveRangeSample,
    bounds: BoundingBox3,
}

struct CandidateSet {
    boxes: Vec<CurveCandidate>,
    has_out: bool,
}

struct CurveScan<'a> {
    state: &'a PatchSubdivision,
    surface_box: &'a BoundingBox3,
    original_sample: CurveRangeSample,
    global_checked: &'a mut bool,
    avoid_cells: &'a mut Vec<i32>,
    candidates: CandidateSet,
}

struct CandidateVisit<'a> {
    state: &'a PatchSubdivision,
    surface_sample: SurfaceRangeSample,
    surface_box: &'a BoundingBox3,
    original_curve: CurveRangeSample,
    curve_box: &'a BoundingBox3,
    parent_surface: SurfaceRangeSample,
    parent_surface_box: &'a BoundingBox3,
    candidates: &'a CandidateSet,
}

struct CandidateVisitResult {
    has_out: bool,
    accepted: Vec<SamplePair>,
}

pub(super) fn compute(
    bean: &BeanFaceIntersector,
    curve_cache: &mut CurveRangeLocalizeData,
    surface_cache: &mut SurfaceRangeLocalizeData,
    curve_box: &BoundingBox3,
    surface_box: &BoundingBox3,
    output: &mut Vec<SamplePair>,
) -> Result<bool, Failure> {
    let mut localizer = Localizer::new(
        bean,
        curve_cache,
        surface_cache,
        curve_box,
        surface_box,
        output,
    );
    localizer.localize(
        CurveRangeSample::default(),
        curve_box,
        SurfaceRangeSample::default(),
        surface_box,
    )
}

impl Localizer<'_> {
    fn new<'a>(
        bean: &'a BeanFaceIntersector,
        curve_cache: &'a mut CurveRangeLocalizeData,
        surface_cache: &'a mut SurfaceRangeLocalizeData,
        curve_root: &'a BoundingBox3,
        surface_root: &'a BoundingBox3,
        output: &'a mut Vec<SamplePair>,
    ) -> Localizer<'a> {
        Localizer {
            bean,
            curve_cache,
            surface_cache,
            curve_root,
            surface_root,
            output,
        }
    }

    fn localize(
        &mut self,
        curve_sample: CurveRangeSample,
        curve_box: &BoundingBox3,
        surface_sample: SurfaceRangeSample,
        surface_box: &BoundingBox3,
    ) -> Result<bool, Failure> {
        let state = subdivision::subdivision(
            self.bean,
            self.curve_cache,
            self.surface_cache,
            curve_sample,
            surface_sample,
        )?;
        if !state.allow.into_iter().any(|allowed| allowed) {
            self.output.push((curve_sample, surface_sample));
            return Ok(true);
        }
        self.visit_surface_cells(&state, curve_sample, curve_box, surface_sample, surface_box)
    }

    fn visit_surface_cells(
        &mut self,
        state: &PatchSubdivision,
        curve_sample: CurveRangeSample,
        curve_box: &BoundingBox3,
        surface_sample: SurfaceRangeSample,
        surface_box: &BoundingBox3,
    ) -> Result<bool, Failure> {
        let mut found_pairs = Vec::new();
        let mut any_out = false;
        let mut global_checked = false;
        let mut avoid_curve_cells = Vec::new();
        for u_cell in 1..=state.counts[1] {
            let u_bounds = cell_bounds(state.u, state.steps[1], u_cell);
            let mut row_has_out = false;
            for v_cell in 1..=state.counts[2] {
                let v_bounds = cell_bounds(state.v, state.steps[2], v_cell);
                let bounds = [u_bounds.0, u_bounds.1, v_bounds.0, v_bounds.1];
                let cell = surface_cell(state, surface_sample, u_cell, v_cell);
                if self.surface_cache.is_range_out(&cell) {
                    row_has_out = true;
                    continue;
                }
                let cell_box = self.surface_cell_box(cell, bounds)?;
                if self.cell_is_out(cell, &cell_box, curve_box, &mut row_has_out)? {
                    continue;
                }
                let candidates = self.curve_candidates(
                    state,
                    &cell_box,
                    curve_sample,
                    &mut global_checked,
                    &mut avoid_curve_cells,
                )?;
                let Some(visited) = self.visit_candidates(CandidateVisit {
                    state,
                    surface_sample: cell,
                    surface_box: &cell_box,
                    original_curve: curve_sample,
                    curve_box,
                    parent_surface: surface_sample,
                    parent_surface_box: surface_box,
                    candidates: &candidates,
                })?
                else {
                    return Ok(false);
                };
                row_has_out |= visited.has_out;
                found_pairs.extend(visited.accepted);
            }
            any_out |= row_has_out;
        }
        self.finish_surface_partition(any_out, curve_sample, surface_sample, found_pairs)
    }

    fn surface_cell_box(
        &mut self,
        sample: SurfaceRangeSample,
        bounds: [f64; 4],
    ) -> Result<BoundingBox3, Failure> {
        let mut output = BoundingBox3::default();
        if self.surface_cache.find_box(&sample, &mut output) {
            return Ok(output);
        }
        self.build_surface_cell_box(bounds, &mut output)?;
        self.surface_cache.add_box(&sample, &output);
        Ok(output)
    }

    fn build_surface_cell_box(
        &mut self,
        bounds: [f64; 4],
        output: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        let surface = self.bean.surface.get().expect("bean surface");
        if surface.get_type()? == rustycad_tkmath::geom_abs::SurfaceType::BSplineSurface {
            *output = boxes::surface_box(surface, bounds, self.bean.criteria, self.surface_cache)?;
            return Ok(());
        }
        AddSurface::add_range(surface, bounds, self.bean.criteria, output)
    }

    fn cell_is_out(
        &mut self,
        sample: SurfaceRangeSample,
        cell_box: &BoundingBox3,
        curve_box: &BoundingBox3,
        row_has_out: &mut bool,
    ) -> Result<bool, Failure> {
        let root_curve_box = self.cached_root_curve_box();
        if cell_box.is_out_box(&root_curve_box) {
            self.surface_cache.add_out_range(&sample);
            *row_has_out = true;
            return Ok(true);
        }
        if cell_box.is_out_box(curve_box) {
            *row_has_out = true;
            return Ok(true);
        }
        Ok(false)
    }

    fn cached_root_curve_box(&self) -> BoundingBox3 {
        self.curve_root.clone()
    }

    fn curve_candidates(
        &mut self,
        state: &PatchSubdivision,
        surface_box: &BoundingBox3,
        original_sample: CurveRangeSample,
        global_checked: &mut bool,
        avoid_cells: &mut Vec<i32>,
    ) -> Result<CandidateSet, Failure> {
        let mut scan = CurveScan {
            state,
            surface_box,
            original_sample,
            global_checked,
            avoid_cells,
            candidates: CandidateSet {
                boxes: Vec::new(),
                has_out: false,
            },
        };
        for cell in 1..=state.counts[0] {
            self.push_curve_candidate(&mut scan, cell)?;
        }
        *scan.global_checked = true;
        Ok(scan.candidates)
    }

    fn push_curve_candidate(&mut self, scan: &mut CurveScan<'_>, cell: i32) -> Result<(), Failure> {
        if scan.avoid_cells.contains(&cell) {
            scan.candidates.has_out = true;
            return Ok(());
        }
        let sample = curve_cell(scan.state, scan.original_sample, cell);
        if self.curve_cache.is_range_out(&sample) {
            scan.candidates.has_out = true;
            return Ok(());
        }
        let (first, last) = cell_bounds(scan.state.curve, scan.state.steps[0], cell);
        let bounds = self.curve_cell_box(sample, first, last)?;
        if bounds.is_out_box(self.surface_root) && !*scan.global_checked {
            self.curve_cache.add_out_range(&sample);
            scan.avoid_cells.push(cell);
            scan.candidates.has_out = true;
            return Ok(());
        }
        if bounds.is_out_box(scan.surface_box) {
            scan.candidates.has_out = true;
            return Ok(());
        }
        scan.candidates
            .boxes
            .push(CurveCandidate { sample, bounds });
        Ok(())
    }

    fn curve_cell_box(
        &mut self,
        sample: CurveRangeSample,
        first: f64,
        last: f64,
    ) -> Result<BoundingBox3, Failure> {
        let mut bounds = BoundingBox3::default();
        if self.curve_cache.find_box(&sample, &mut bounds) {
            return Ok(bounds);
        }
        Add3dCurve::add_range(
            self.bean.curve.get().expect("bean curve"),
            first,
            last,
            self.bean.criteria,
            &mut bounds,
        )?;
        self.curve_cache.add_box(&sample, &bounds);
        Ok(bounds)
    }

    fn visit_candidates(
        &mut self,
        input: CandidateVisit<'_>,
    ) -> Result<Option<CandidateVisitResult>, Failure> {
        let mut output = CandidateVisitResult {
            has_out: input.candidates.has_out,
            accepted: Vec::new(),
        };
        let mut use_old_curve = false;
        let mut candidate_has_out = input.candidates.has_out;
        let mut check_size = !input.candidates.has_out;
        for candidate in &input.candidates.boxes {
            let size = self.choose_parent_range(
                input.state,
                candidate,
                input.surface_box,
                candidate_has_out,
                &mut check_size,
            )?;
            if size.use_old_curve {
                use_old_curve = true;
                output.has_out = true;
                break;
            }
            candidate_has_out |= size.has_out;
            let (next_surface, next_box) = if size.use_old_surface {
                (input.parent_surface, input.parent_surface_box)
            } else {
                (input.surface_sample, input.surface_box)
            };
            if size.use_old_surface && candidate.sample == input.original_curve {
                return Ok(None);
            }
            if !candidate_has_out {
                output
                    .accepted
                    .push((candidate.sample, input.surface_sample));
                continue;
            }
            output.has_out = true;
            if !self.localize(candidate.sample, &candidate.bounds, next_surface, next_box)? {
                return Ok(None);
            }
        }
        if !self.retry_old_curve(&input, use_old_curve)? {
            return Ok(None);
        }
        output.has_out |= use_old_curve;
        Ok(Some(output))
    }

    fn choose_parent_range(
        &self,
        state: &PatchSubdivision,
        candidate: &CurveCandidate,
        surface_box: &BoundingBox3,
        already_out: bool,
        check_size: &mut bool,
    ) -> Result<ParentChoice, Failure> {
        let mut choice = ParentChoice {
            has_out: already_out,
            ..ParentChoice::default()
        };
        if !*check_size {
            return Ok(choice);
        }
        if state.source_depths.contains(&0) {
            choice.has_out = true;
            return Ok(choice);
        }
        if state.source_depths.iter().any(|depth| *depth >= 4) {
            return Ok(choice);
        }
        let extents = [
            candidate.bounds.square_extent(),
            surface_box.square_extent(),
        ];
        if candidate.bounds.is_whole() || surface_box.is_whole() {
            return Ok(choice);
        }
        if extents[0] < extents[1] && extents[0] * 10. < extents[1] {
            choice.use_old_curve = true;
            choice.has_out = true;
        } else if extents[1] <= extents[0] && extents[1] * 10. < extents[0] {
            choice.use_old_surface = true;
            choice.has_out = true;
        }
        Ok(choice)
    }

    fn retry_old_curve(
        &mut self,
        input: &CandidateVisit<'_>,
        use_old_curve: bool,
    ) -> Result<bool, Failure> {
        if !use_old_curve
            || !input.state.allow[0]
            || !(input.state.allow[1] || input.state.allow[2])
        {
            return Ok(true);
        }
        self.localize(
            input.original_curve,
            input.curve_box,
            input.surface_sample,
            input.surface_box,
        )
    }

    fn finish_surface_partition(
        &mut self,
        any_out: bool,
        curve_sample: CurveRangeSample,
        surface_sample: SurfaceRangeSample,
        found_pairs: Vec<SamplePair>,
    ) -> Result<bool, Failure> {
        if !any_out {
            self.output.push((curve_sample, surface_sample));
            return Ok(true);
        }
        self.output.extend(found_pairs);
        Ok(true)
    }
}

#[derive(Default)]
struct ParentChoice {
    has_out: bool,
    use_old_curve: bool,
    use_old_surface: bool,
}
