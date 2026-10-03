use super::ShapeAnalysisWireVertex;
use crate::shape_analysis::{ShapeAnalysisCurve, ShapeAnalysisEdge};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::Point3;

struct JunctionCurves {
    previous: Handle<dyn GeomCurve>,
    following: Handle<dyn GeomCurve>,
    previous_range: (f64, f64),
    following_range: (f64, f64),
    previous_point: Point3,
    following_point: Point3,
}
impl ShapeAnalysisWireVertex {
    /// Analyze cyclic consecutive edges; `tool.analyze()?`.
    pub fn analyze(&mut self) -> Result<(), Failure> {
        let count = self.records.read().len() as i32;
        if count == 0 {
            return Ok(());
        }
        self.done = true;
        for index in 1..=count {
            self.analyze_junction(index, if index == count { 1 } else { index + 1 })?;
        }
        Ok(())
    }
    fn analyze_junction(&mut self, index: i32, next: i32) -> Result<(), Failure> {
        let first = self.wire.edge(index)?;
        let second = self.wire.edge(next)?;
        let endpoints = JunctionEndpoints::from_edges(&first, &second)?;
        let Some(curves) = JunctionCurves::from_edges(&first, &second)? else {
            return Ok(());
        };
        let status = junction_status(&endpoints, &curves, self.precision);
        let status = if status > 0 && endpoints.same_vertex {
            0
        } else {
            status
        };
        self.update_record(index, |record| record.status = status)?;
        if status >= 0 {
            return Ok(());
        }
        self.project_junction(index, &curves)
    }
    fn project_junction(&mut self, index: i32, curves: &JunctionCurves) -> Result<(), Failure> {
        // Both projections run before deciding, as in pinned OCCT.
        let first = curves.project_previous(self.precision)?;
        let second = curves.project_following(self.precision)?;
        if first.0 <= self.precision {
            return self.set_start(index, first.1.xyz(), first.2);
        }
        if second.0 <= self.precision {
            return self.set_end(index, second.1.xyz(), second.2);
        }
        Ok(())
    }
}
struct JunctionEndpoints {
    first: Point3,
    second: Point3,
    tolerance_first: f64,
    tolerance_second: f64,
    same_vertex: bool,
}
impl JunctionEndpoints {
    fn from_edges(
        first: &rustycad_tkbrep::topods::Edge,
        second: &rustycad_tkbrep::topods::Edge,
    ) -> Result<Self, Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        let v1 = analyzer.last_vertex(first);
        let v2 = analyzer.first_vertex(second);
        let first = BRepTool::pnt(&v1)?;
        let second = BRepTool::pnt(&v2)?;
        Ok(Self {
            first,
            second,
            tolerance_first: BRepTool::vertex_tolerance(&v1)?,
            tolerance_second: BRepTool::vertex_tolerance(&v2)?,
            same_vertex: v1.is_equal(&v2),
        })
    }
}
impl JunctionCurves {
    fn from_edges(
        first: &rustycad_tkbrep::topods::Edge,
        second: &rustycad_tkbrep::topods::Edge,
    ) -> Result<Option<Self>, Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        let (previous, cf, upre) = analyzer.curve_3d(first, true)?;
        let (following, ufol, cl) = analyzer.curve_3d(second, true)?;
        if previous.is_null() || following.is_null() {
            return Ok(None);
        }
        Ok(Some(Self {
            previous_point: previous.value(upre)?,
            following_point: following.value(ufol)?,
            previous,
            following,
            previous_range: (cf, upre),
            following_range: (ufol, cl),
        }))
    }
    fn project_previous(&self, precision: f64) -> Result<(f64, Point3, f64), Failure> {
        let (cf, upre) = self.previous_range;
        ShapeAnalysisCurve.project(
            &self.previous,
            self.following_point,
            precision,
            ((cf + upre) / 2., upre),
            true,
        )
    }
    fn project_following(&self, precision: f64) -> Result<(f64, Point3, f64), Failure> {
        let (ufol, cl) = self.following_range;
        ShapeAnalysisCurve.project(
            &self.following,
            self.previous_point,
            precision,
            (ufol, (ufol + cl) / 2.),
            true,
        )
    }
}
fn junction_status(endpoints: &JunctionEndpoints, curves: &JunctionCurves, precision: f64) -> i32 {
    let d1 = endpoints.first.distance(&curves.previous_point);
    let d2 = endpoints.second.distance(&curves.following_point);
    let between = endpoints.first.distance(&endpoints.second);
    if d1 <= endpoints.tolerance_first
        && d2 <= endpoints.tolerance_second
        && between <= endpoints.tolerance_first + endpoints.tolerance_second
    {
        return 1;
    }
    if d1 <= precision && d2 <= precision && between <= precision {
        return 2;
    }
    -1
}
