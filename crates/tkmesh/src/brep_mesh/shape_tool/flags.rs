//! One-way deterioration of discrete edge flags from actual curve geometry.
use super::*;
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorCurve, topexp};
use rustycad_tkernel::precision;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
impl ShapeTool {
    /// Preserve false consistency flags and true degeneracy; `ShapeTool::check_and_update_flags(&edge, &pcurve)?`.
    pub fn check_and_update_flags(edge: &EdgeHandle, pcurve: &PCurveHandle) -> Result<(), Failure> {
        if !edge.get_same_param() && !edge.get_same_range() && edge.get_degenerated() {
            return Ok(());
        }
        let topology = edge.get_edge()?;
        let face = pcurve.get_face().upgrade().get_face()?;
        let (curve, first, last) = Self::range_3d(&topology, false)?;
        if curve.is_null() {
            edge.set_degenerated(true);
            return Ok(());
        }
        let on_surface = BRepAdaptorCurve::with_edge_on_face(&topology, &face)?;
        Self::update_same_range(edge, &on_surface, first, last)?;
        if !edge.get_degenerated() {
            Self::update_degenerated(edge, &topology, &curve, first, last)?;
        }
        Ok(())
    }
    fn update_same_range(
        edge: &EdgeHandle,
        on_surface: &BRepAdaptorCurve,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        if !edge.get_same_range() {
            return Ok(());
        }
        let first_difference = on_surface.first_parameter()? - first;
        let last_difference = on_surface.last_parameter()? - last;
        edge.set_same_range(
            first_difference.abs() < precision::p_confusion()
                && last_difference.abs() < precision::p_confusion(),
        );
        if !edge.get_same_range() {
            edge.set_same_param(false);
        }
        Ok(())
    }
    fn update_degenerated(
        edge: &EdgeHandle,
        topology: &Edge,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        let (start, end) = topexp::vertices(topology, false);
        if start.is_null() || end.is_null() {
            edge.set_degenerated(true);
            return Ok(());
        }
        if start.is_same(&end) {
            let tolerance = BRepTool::vertex_tolerance(&start)?;
            let length = Self::sampled_length(curve, first, last, tolerance)?;
            edge.set_degenerated(length < tolerance);
        }
        Ok(())
    }
    fn sampled_length(
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<f64, Failure> {
        let delta = (last - first) / 20.;
        let mut previous = curve.value(first)?;
        let mut length = 0.;
        for index in 1..=20 {
            let point = curve.value(first + f64::from(index) * delta)?;
            length += previous.distance(&point);
            if length > tolerance {
                break;
            }
            previous = point;
        }
        Ok(length)
    }
}
