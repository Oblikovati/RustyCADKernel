//! Native preparation swaps actual retained inputs and only ever sets the swap flag.
use super::{
    EdgeEdge,
    edge_math::{curve_deflection, curve_priority},
    edge_resolution::{curve_resolution, resolution_coefficient},
};
use crate::int_tools::Range;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
impl EdgeEdge {
    pub(super) fn check_data(&mut self) -> Result<(), Failure> {
        if self.edges[0].is_null() || self.edges[1].is_null() {
            self.error = 1;
            return Ok(());
        }
        if BRepTool::degenerated(&self.edges[0])? || BRepTool::degenerated(&self.edges[1])? {
            self.error = 2;
            return Ok(());
        }
        if !BRepTool::is_geometric_edge(&self.edges[0])?
            || !BRepTool::is_geometric_edge(&self.edges[1])?
        {
            self.error = 3;
        }
        Ok(())
    }
    pub(super) fn prepare(&mut self) -> Result<(), Failure> {
        self.curves[0].initialize(&self.edges[0])?;
        self.curves[1].initialize(&self.edges[1])?;
        for index in 0..2 {
            if self.ranges[index].first() == 0. && self.ranges[index].last() == 0. {
                self.ranges[index] = Range::new(
                    self.curves[index].first_parameter()?,
                    self.curves[index].last_parameter()?,
                );
            }
        }
        let priority = self.priorities()?;
        if priority.0 < priority.1 {
            self.edges.swap(0, 1);
            self.curves.swap(0, 1);
            self.ranges.swap(0, 1);
            self.swapped = true;
        }
        self.tolerances[0] = self.curves[0].tolerance()? + self.fuzzy / 2.;
        self.tolerances[1] = self.curves[1].tolerance()? + self.fuzzy / 2.;
        self.tolerance = self.tolerances[0] + self.tolerances[1];
        if priority.0 != 0 || priority.1 != 0 {
            self.prepare_geometry()?;
        }
        Ok(())
    }
    fn priorities(&self) -> Result<(i32, i32), Failure> {
        let mut first = curve_priority(self.curves[0].get_type()?);
        let second = curve_priority(self.curves[1].get_type()?);
        if first == second && first != 0 {
            let other = curve_deflection(&self.curves[1], self.ranges[1])?;
            let this = if other > precision::confusion() {
                curve_deflection(&self.curves[0], self.ranges[0])?
            } else {
                1.
            };
            if this < other {
                first -= 1;
            }
        }
        Ok((first, second))
    }
    fn prepare_geometry(&mut self) -> Result<(), Failure> {
        self.geometry[0] = BRepTool::curve(&self.edges[0])?.0;
        self.geometry[1] = BRepTool::curve(&self.edges[1])?.0;
        self.coefficients[0] = resolution_coefficient(&self.curves[0], self.ranges[0])?;
        self.coefficients[1] = resolution_coefficient(&self.curves[1], self.ranges[1])?;
        self.resolutions[0] = self.curve_resolution(0, self.tolerances[0])?;
        self.resolutions[1] = self.curve_resolution(1, self.tolerances[1])?;
        for index in 0..2 {
            self.precisions[index] = 5e-13;
            let maximum = std_max(
                self.ranges[index].first().abs(),
                self.ranges[index].last().abs(),
            );
            if maximum > 999. {
                self.precisions[index] = 5e-16 * maximum;
            }
        }
        Ok(())
    }
    pub(super) fn curve_resolution(&self, index: usize, tolerance: f64) -> Result<f64, Failure> {
        curve_resolution(
            &self.curves[index].transformed_curve().geom_curve(),
            self.curves[index].get_type()?,
            self.coefficients[index],
            tolerance,
        )
    }
}
