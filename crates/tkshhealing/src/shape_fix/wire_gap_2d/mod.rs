//! Native ShapeFix_Wire_1.cxx pcurve conversion and gap mutations.
mod clipping;
mod convert;
mod domains;
mod fallback;
mod ranges;
mod replace;
use super::{FixRoot, ShapeFixWire};
use crate::{
    shape_analysis::ShapeAnalysisEdge, shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendStatus,
};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topods::{self, Edge, Face},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCurve, Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;
struct PcurveGap {
    edge: Edge,
    index: i32,
    curve: Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    point: Point2,
    reversed: bool,
    trimmed: bool,
}
impl ShapeFixWire {
    /// Repair pcurve junctions and accumulate native status flags; `tool.fix_gaps_2d()?`.
    pub fn fix_gaps_2d(&self) -> Result<bool, Failure> {
        self.state.write().gaps_2d_status = 0;
        let start = if self.closed_mode() { 1 } else { 2 };
        if self.fix_gaps_by_ranges() {
            self.fix_gap_2d_pass(start, false)?;
        }
        self.fix_gap_2d_pass(start, true)?;
        Ok(self.status_gaps_2d(ShapeExtendStatus::Done))
    }
    fn fix_gap_2d_pass(&self, start: i32, convert: bool) -> Result<(), Failure> {
        for index in start..=self.nb_edges() {
            self.fix_gap_2d(index, convert)?;
            let flags = self.state.read().last_fix_status;
            self.state.write().gaps_2d_status |= flags;
        }
        Ok(())
    }

    /// Repair one pcurve junction by native conversion; `tool.fix_gap_2d(2,true)?`.
    pub fn fix_gap_2d(&self, index: i32, convert: bool) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        if self.context().is_null() {
            self.set_context(&Handle::new(ShapeBuildReShape::new()));
        }
        let count = self.nb_edges();
        let second = if index > 0 { index } else { count };
        let first = if second > 1 { second - 1 } else { count };
        let face = self.working_analyzer().face();
        let first = PcurveGap::load(self, &face, first, true)?;
        let second = PcurveGap::load(self, &face, second, false)?;
        let (Some(first), Some(second)) = (first, second) else {
            self.set_last_fix_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        };
        if first.point.distance(second.point) <= precision::p_confusion() {
            return Ok(false);
        }
        if !convert {
            return ranges::repair_range(self, &face, &first, &second);
        }
        self.convert_pcurve_gap(&face, &first, &second)
    }
    fn convert_pcurve_gap(
        &self,
        face: &Face,
        first: &PcurveGap,
        second: &PcurveGap,
    ) -> Result<bool, Failure> {
        let first_curve = convert::convert_gap_curve(self, first)?;
        let same = first.index == second.index;
        let second_curve = if same {
            Handle::default()
        } else {
            convert::convert_gap_curve(self, second)?
        };
        let target = if first_curve.is_null() {
            first.point
        } else if second_curve.is_null() {
            second.point
        } else {
            Point2::new(
                (first.point.x() + second.point.x()) * 0.5,
                (first.point.y() + second.point.y()) * 0.5,
            )
        };
        convert::move_gap_endpoint(&first_curve, target, first.reversed, same)?;
        convert::move_gap_endpoint(&second_curve, target, !second.reversed, false)?;
        if first_curve.is_null() && second_curve.is_null() {
            self.set_last_fix_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        replace::replace_pcurves(self, face, (first, second), (&first_curve, &second_curve))?;
        self.set_last_fix_status(ShapeExtendStatus::Done1);
        Ok(true)
    }
}
impl PcurveGap {
    fn load(
        tool: &ShapeFixWire,
        face: &Face,
        index: i32,
        last_end: bool,
    ) -> Result<Option<Self>, Failure> {
        let edge = tool.wire_data()?.edge(index)?;
        let edge = topods::edge(&ReShape::apply(&*tool.context(), &edge, ShapeEnum::Shape)?)?;
        let (curve, start, end) = ShapeAnalysisEdge::new().pcurve_on_face(&edge, face, true)?;
        if curve.is_null() || tool.wire_data()?.is_seam(index) {
            return Ok(None);
        }
        let trimmed = has_trimmed_basis(&curve);
        let point = curve.value(if last_end { end } else { start })?;
        let reversed = edge.orientation() == Orientation::Reversed;
        let (first, last) = if reversed { (end, start) } else { (start, end) };
        Ok(Some(Self {
            edge,
            index,
            curve,
            first,
            last,
            point,
            reversed,
            trimmed,
        }))
    }
}

fn has_trimmed_basis(curve: &Handle<dyn Geom2dCurve>) -> bool {
    let mut current = curve.clone();
    loop {
        if !current.cast::<Geom2dTrimmedCurve>().is_null() {
            return true;
        }
        let offset = current.cast::<Geom2dOffsetCurve>();
        let Some(offset) = offset.get() else {
            return false;
        };
        current = offset.basis_curve();
    }
}
