//! Native dispatch keeps errors and prior common parts across repeated Perform calls.
use super::{
    EdgeEdge,
    edge_coincidence::CurvePair,
    edge_lines::LinePair,
    edge_solutions::{SearchCurve, SolutionSearch},
};
use crate::int_tools::{CommonPrt, Range};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurve, top_abs::ShapeEnum};
use rustycad_tkgeombase::extrema::ExtFlag;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tktopalgo::brep_extrema::BRepExtremaDistShapeShape;
impl EdgeEdge {
    pub(super) fn perform_native(&mut self) -> Result<(), Failure> {
        self.check_data()?;
        if self.error != 0 {
            return Ok(());
        }
        self.prepare()?;
        let kinds = [self.curves[0].get_type()?, self.curves[1].get_type()?];
        if kinds == [CurveType::Line, CurveType::Line] {
            return self.line_intersection();
        }
        if self.quick && self.pair(self.ranges).is_coincident(self.tolerance)? {
            return self.add_solution(self.ranges, ShapeEnum::Edge);
        }
        if self.analytic_separation(kinds)? {
            return Ok(());
        }
        let mut solutions = Vec::new();
        let split = self.solution_search().find(self.ranges, &mut solutions)?;
        self.merge_solutions(&solutions, split)
    }
    fn analytic_separation(&self, kinds: [CurveType; 2]) -> Result<bool, Failure> {
        if !kinds
            .iter()
            .all(|kind| (*kind as i32) <= CurveType::Parabola as i32)
            || !kinds.contains(&CurveType::Line)
        {
            return Ok(false);
        }
        let extrema = BRepExtremaDistShapeShape::with_shapes(
            &self.edges[0],
            &self.edges[1],
            (ExtFlag::Min, Default::default()),
        )?;
        Ok(extrema.is_done() && extrema.value()? > 1.1 * self.tolerance)
    }
    fn line_intersection(&mut self) -> Result<(), Failure> {
        let pair = LinePair {
            edges: (&self.edges[0], &self.edges[1]),
            lines: (self.curves[0].line()?, self.curves[1].line()?),
            ranges: (self.ranges[0], self.ranges[1]),
            tolerances: (self.tolerances[0], self.tolerances[1]),
            tolerance: self.tolerance,
        };
        if let Some(part) = pair.compute()? {
            self.parts.push(part);
        }
        Ok(())
    }
    fn solution_search(&self) -> SolutionSearch<'_> {
        SolutionSearch {
            curves: [self.search_curve(0), self.search_curve(1)],
            tolerance: self.tolerance,
        }
    }
    fn search_curve(&self, index: usize) -> SearchCurve<'_> {
        SearchCurve {
            geometry: &self.geometry[index],
            adaptor: &self.curves[index],
            tolerance: self.tolerances[index],
            resolution: self.resolutions[index],
            precision: self.precisions[index],
            coefficient: self.coefficients[index],
        }
    }
    pub(super) fn pair(&self, ranges: [Range; 2]) -> CurvePair<'_> {
        CurvePair {
            first: &self.geometry[0],
            second: &self.geometry[1],
            first_range: ranges[0],
            second_range: ranges[1],
        }
    }
    pub(super) fn add_solution(
        &mut self,
        ranges: [Range; 2],
        kind: ShapeEnum,
    ) -> Result<(), Failure> {
        let (first, second) = if self.swapped { (1, 0) } else { (0, 1) };
        let mut part = CommonPrt::default();
        part.set_type(kind);
        part.set_edge1(&self.edges[first]);
        part.set_edge2(&self.edges[second]);
        part.set_range1(ranges[first]);
        part.append_range2(ranges[second]);
        if kind == ShapeEnum::Vertex {
            let mut best = (0., 0.);
            super::edge_best::best_solution(
                &self.pair(ranges),
                &self.curves[0],
                self.coefficients[0],
                self.tolerance,
                self.precisions[0],
                &mut best,
            )?;
            let parameters = [best.0, best.1];
            part.set_vertex_parameter1(parameters[first]);
            part.set_vertex_parameter2(parameters[second]);
        }
        self.parts.push(part.clone());
        Ok(())
    }
}
