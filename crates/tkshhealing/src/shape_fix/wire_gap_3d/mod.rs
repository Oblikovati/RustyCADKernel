//! Native range and conversion branches of ShapeFix_Wire_1.cxx::FixGap3d.
mod convert;
mod ranges;
mod replace;
use super::{FixRoot, ShapeFixWire};
use crate::{
    shape_analysis::ShapeAnalysisEdge, shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendStatus,
};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_tools::ReShape,
    topods::{self, Edge, Vertex},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    geom::{GeomBSplineCurve, GeomCurve},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::Point3;
pub(super) struct GapCurve {
    edge: Edge,
    index: i32,
    curve: Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
    point: Point3,
    reversed: bool,
}
struct GapPair {
    first: GapCurve,
    second: GapCurve,
    vertices: (Vertex, Vertex),
    target: Point3,
}
impl ShapeFixWire {
    /// Repair 3D junctions by range adjustment and spline conversion; `tool.fix_gaps_3d()?`.
    pub fn fix_gaps_3d(&self) -> Result<bool, Failure> {
        self.state.write().gaps_3d_status = 0;
        let start = if self.closed_mode() { 1 } else { 2 };
        if self.fix_gaps_by_ranges() {
            self.fix_gap_3d_pass(start, false)?;
        }
        self.fix_gap_3d_pass(start, true)?;
        Ok(self.status_gaps_3d(ShapeExtendStatus::Done))
    }

    fn fix_gap_3d_pass(&self, start: i32, convert: bool) -> Result<(), Failure> {
        for index in start..=self.nb_edges() {
            self.fix_gap_3d(index, convert)?;
            let last = self.state.read().last_fix_status;
            self.state.write().gaps_3d_status |= last;
        }
        Ok(())
    }
    /// Read the preliminary range-repair mode; `tool.fix_gaps_by_ranges()`.
    pub fn fix_gaps_by_ranges(&self) -> bool {
        self.state.read().fix_gaps_by_ranges
    }
    /// Enable preliminary range repair; `tool.set_fix_gaps_by_ranges(true)`.
    pub fn set_fix_gaps_by_ranges(&self, enabled: bool) {
        self.state.write().fix_gaps_by_ranges = enabled;
    }

    /// Repair one 3D curve junction; `tool.fix_gap_3d(2,true)?` uses native spline conversion.
    /// With conversion disabled, adjust ranges using projections and extrema.
    pub fn fix_gap_3d(&self, index: i32, convert: bool) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        if self.context().is_null() {
            self.set_context(&Handle::new(ShapeBuildReShape::new()));
        }
        let Some(pair) = GapPair::load(self, index)? else {
            self.set_last_fix_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        };
        if !convert {
            if pair.first.point.distance(&pair.second.point) <= self.precision() {
                return Ok(false);
            }
            return ranges::repair(self, &pair);
        }
        self.convert_gap_pair(pair)
    }
    fn convert_gap_pair(&self, pair: GapPair) -> Result<bool, Failure> {
        let precision = self.precision();
        if pair.first.point.distance(&pair.target) < precision
            && pair.second.point.distance(&pair.target) < precision
        {
            return Ok(false);
        }
        let same = pair.first.index == pair.second.index;
        let first = convert::convert_gap_curve(
            &pair.first,
            pair.target,
            precision,
            self.working_analyzer().precision(),
            same,
        )?;
        let second = if same {
            Handle::default()
        } else {
            convert::convert_gap_curve(
                &pair.second,
                pair.target,
                precision,
                self.working_analyzer().precision(),
                false,
            )?
        };
        let target = if first.is_null() {
            pair.first.point
        } else if second.is_null() {
            pair.second.point
        } else {
            pair.target
        };
        convert::move_gap_endpoint(&first, target, pair.first.reversed, same)?;
        convert::move_gap_endpoint(&second, target, !pair.second.reversed, false)?;
        if first.is_null() && second.is_null() {
            self.set_last_fix_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        replace::replace_gap_curves(
            self,
            &pair,
            replace::CurveRepair::spline(&first).as_ref(),
            replace::CurveRepair::spline(&second).as_ref(),
        )?;
        self.set_last_fix_status(ShapeExtendStatus::Done1);
        Ok(true)
    }
}
impl GapPair {
    fn load(tool: &ShapeFixWire, index: i32) -> Result<Option<Self>, Failure> {
        let count = tool.wire_data()?.nb_edges();
        let second = if index > 0 { index } else { count };
        let first = if second > 1 { second - 1 } else { count };
        let first = GapCurve::load(tool, first, true)?;
        let second = GapCurve::load(tool, second, false)?;
        let (Some(first), Some(second)) = (first, second) else {
            return Ok(None);
        };
        let analysis = ShapeAnalysisEdge::new();
        let vertices = (
            analysis.last_vertex(&first.edge),
            analysis.first_vertex(&second.edge),
        );
        let a = BRepTool::pnt(&vertices.0)?;
        let b = BRepTool::pnt(&vertices.1)?;
        let target = if vertices.0.is_same(&vertices.1) {
            a
        } else {
            Point3::new(
                (a.x() + b.x()) * 0.5,
                (a.y() + b.y()) * 0.5,
                (a.z() + b.z()) * 0.5,
            )
        };
        Ok(Some(Self {
            first,
            second,
            vertices,
            target,
        }))
    }
}
impl GapCurve {
    fn load(tool: &ShapeFixWire, index: i32, last_end: bool) -> Result<Option<Self>, Failure> {
        let edge = tool.wire_data()?.edge(index)?;
        let edge = topods::edge(&ReShape::apply(
            &*tool.context(),
            &edge,
            rustycad_tkg3d::top_abs::ShapeEnum::Shape,
        )?)?;
        let (curve, start, end) = ShapeAnalysisEdge::new().curve_3d(&edge, true)?;
        if curve.is_null() {
            return Ok(None);
        }
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
        }))
    }
}
