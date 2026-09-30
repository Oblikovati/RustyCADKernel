//! Recursive range clipping uses native box order and rejects nonshrinking subdivisions.
use super::{
    edge_coincidence::CurvePair,
    edge_math::{is_closed, split_range},
    edge_parameters::ParameterSearch,
};
use crate::int_tools::Range;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkmath::bnd::BoundingBox3;
#[path = "solution_clip.rs"]
mod clip;
pub(super) struct SearchCurve<'a> {
    pub geometry: &'a Handle<dyn GeomCurve>,
    pub adaptor: &'a BRepAdaptorCurve,
    pub tolerance: f64,
    pub resolution: f64,
    pub precision: f64,
    pub coefficient: f64,
}
pub(super) struct SolutionSearch<'a> {
    pub curves: [SearchCurve<'a>; 2],
    pub tolerance: f64,
}
struct RangeBoxes {
    ranges: [Range; 2],
    boxes: [BoundingBox3; 2],
}
impl SearchCurve<'_> {
    fn bounds(&self, range: Range) -> Result<BoundingBox3, Failure> {
        let mut bounds = BoundingBox3::default();
        Add3dCurve::add_range(
            self.adaptor,
            range.first(),
            range.last(),
            self.tolerance,
            &mut bounds,
        )?;
        Ok(bounds)
    }
    fn closed(&self, range: Range) -> Result<bool, Failure> {
        is_closed(
            self.geometry,
            range.first(),
            range.last(),
            self.tolerance,
            self.resolution,
        )
    }
    fn parameters(
        &self,
        range: Range,
        bounds: &BoundingBox3,
        output: &mut Range,
    ) -> Result<bool, Failure> {
        let search = ParameterSearch {
            curve: self.adaptor,
            range,
            tolerance: self.tolerance,
            resolution: self.resolution,
            precision: self.precision,
            coefficient: self.coefficient,
            bounds,
        };
        let (mut first, mut last) = output.range();
        let result = search.find(&mut first, &mut last);
        *output = Range::new(first, last);
        result
    }
    fn split(&self, range: Range, count: i32) -> Vec<Range> {
        let mut segments = Vec::new();
        split_range(
            range.first(),
            range.last(),
            self.resolution,
            count,
            &mut segments,
        );
        segments
    }
}
impl SolutionSearch<'_> {
    pub(super) fn find(
        &self,
        ranges: [Range; 2],
        output: &mut Vec<[Range; 2]>,
    ) -> Result<bool, Failure> {
        let closed = self.curves[1].closed(ranges[1])?;
        let first_box = self.curves[0].bounds(ranges[0])?;
        if !closed || first_box.is_out_point(self.curves[1].geometry.eval_d0(ranges[1].first())?) {
            let boxes = [
                self.curves[0].bounds(ranges[0])?,
                self.curves[1].bounds(ranges[1])?,
            ];
            self.find_patch(RangeBoxes { ranges, boxes }, output)?;
            return Ok(false);
        }
        if self
            .pair(ranges)
            .check_coincidence(self.tolerance, self.curves[0].resolution)?
            == 0
        {
            output.push(ranges);
            return Ok(false);
        }
        self.find_closed(ranges, output)
    }
    fn pair(&self, ranges: [Range; 2]) -> CurvePair<'_> {
        CurvePair {
            first: self.curves[0].geometry,
            second: self.curves[1].geometry,
            first_range: ranges[0],
            second_range: ranges[1],
        }
    }
    fn find_closed(
        &self,
        ranges: [Range; 2],
        output: &mut Vec<[Range; 2]>,
    ) -> Result<bool, Failure> {
        let count = if self.curves[0].closed(ranges[0])? {
            2
        } else {
            1
        };
        let first = self.curves[0].split(ranges[0], count);
        let second = self.curves[1].split(ranges[1], 2);
        for a in first {
            let first_box = self.curves[0].bounds(a)?;
            for b in &second {
                let boxes = [first_box.clone(), self.curves[1].bounds(*b)?];
                self.find_patch(
                    RangeBoxes {
                        ranges: [a, *b],
                        boxes,
                    },
                    output,
                )?;
            }
        }
        Ok(second.len() > 1)
    }
    fn find_patch(
        &self,
        mut patch: RangeBoxes,
        output: &mut Vec<[Range; 2]>,
    ) -> Result<(), Failure> {
        let Some(mut thin) = self.clip(&mut patch)? else {
            return Ok(());
        };
        let mut coincidence = 1;
        if !thin {
            coincidence = self
                .pair(patch.ranges)
                .check_coincidence(self.tolerance, self.curves[0].resolution)?;
            thin = coincidence == 0;
        }
        if thin {
            if coincidence == 0 || self.midpoint_close(patch.ranges)? {
                output.push(patch.ranges);
            }
            return Ok(());
        }
        let resolutions = (self.curves[0].resolution, self.curves[1].resolution);
        if super::edge_intersection::is_intersection(
            &self.pair(patch.ranges),
            self.tolerance,
            resolutions,
        )? {
            self.subdivide(patch.ranges, output)?;
        }
        Ok(())
    }
    fn midpoint_close(&self, ranges: [Range; 2]) -> Result<bool, Failure> {
        let point = self.curves[0]
            .geometry
            .eval_d0((ranges[0].first() + ranges[0].last()) * 0.5)?;
        let mut projector = self.pair(ranges).projector()?;
        projector.perform(point)?;
        if projector.nb_points() > 0 {
            return Ok(projector.lower_distance()? <= self.tolerance);
        }
        let other = self.curves[1]
            .geometry
            .eval_d0((ranges[1].first() + ranges[1].last()) * 0.5)?;
        Ok(point.is_equal(&other, self.tolerance))
    }
    fn subdivide(&self, ranges: [Range; 2], output: &mut Vec<[Range; 2]>) -> Result<(), Failure> {
        let extent = self.curves[0].bounds(ranges[0])?.square_extent();
        let second_box = self.curves[1].bounds(ranges[1])?;
        let segments = self.curves[0].split(ranges[0], 3);
        for first in &segments {
            let first_box = self.curves[0].bounds(*first)?;
            if !first_box.is_out_box(&second_box)
                && (segments.len() == 1 || first_box.square_extent() < extent)
            {
                let patch = RangeBoxes {
                    ranges: [*first, ranges[1]],
                    boxes: [first_box, second_box.clone()],
                };
                self.find_patch(patch, output)?;
            }
        }
        Ok(())
    }
}
