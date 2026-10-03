// TempSameRange differs from BRepLib::SameRange: OCC966 periodic shifts,
// Bezier segmentation, and the PConfusion comparison are significant.
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepGCurve, BRepTool};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkgeombase::geom_lib;

type ParameterRange = (f64, f64);

struct RangeRepair {
    current: Option<ParameterRange>,
    tolerance: f64,
    first_output: Handle<dyn Geom2dCurve>,
    second_output: Handle<dyn Geom2dCurve>,
}

pub(super) fn temporary_same_range(edge: &Edge, tolerance: f64) -> Result<(), Failure> {
    let curve = BRepTool::curve_with_location(edge)?;
    let mut repair = RangeRepair {
        current: curve.curve.is_some().then_some((curve.first, curve.last)),
        tolerance,
        first_output: Handle::default(),
        second_output: Handle::default(),
    };
    for representation in edge_tshape(edge, "ShapeFix_Edge::TempSameRange")?.curves() {
        if let Some(geometric) = as_gcurve(&representation).get() {
            repair.repair_representation(geometric)?;
        }
    }
    let (first, last) = repair.current.unwrap_or((curve.first, curve.last));
    BRepBuilder::default().range(edge, first, last, false)?;
    BRepBuilder::default().same_range(edge, true)
}

impl RangeRepair {
    fn repair_representation(&mut self, geometric: &dyn BRepGCurve) -> Result<(), Failure> {
        if !geometric.is_curve_on_surface() && !geometric.is_curve_on_closed_surface() {
            return Ok(());
        }
        let range = geometric.range();
        let current = *self.current.get_or_insert(range);
        if !range_differs(range, current) {
            return Ok(());
        }
        let mut old = (0.0, 0.0);
        if geometric.is_curve_on_surface() {
            let curve = geometric.pcurve()?;
            old = periodic_range(&curve, range)?;
            reparameterize(&curve, old, current, self.tolerance, &mut self.first_output)?;
            geometric.set_pcurve(&self.first_output)?;
        }
        if geometric.is_curve_on_closed_surface() {
            reparameterize(
                &geometric.pcurve2()?,
                old,
                current,
                self.tolerance,
                &mut self.second_output,
            )?;
            geometric.set_pcurve2(&self.second_output)?;
        }
        Ok(())
    }
}

fn range_differs(first: ParameterRange, second: ParameterRange) -> bool {
    (first.0 - second.0).abs() > precision::p_confusion()
        || (first.1 - second.1).abs() > precision::p_confusion()
}

fn periodic_range(
    curve: &Handle<dyn Geom2dCurve>,
    range: ParameterRange,
) -> Result<ParameterRange, Failure> {
    if !ShapeAnalysisCurve::is_periodic_2d(curve)? {
        return Ok(range);
    }
    let trimmed = Geom2dTrimmedCurve::new(curve, range.0, range.1, true, true)?;
    let shift = trimmed.first_parameter() - range.0;
    Ok((range.0 + shift, range.1 + shift))
}

fn reparameterize(
    curve: &Handle<dyn Geom2dCurve>,
    old: ParameterRange,
    requested: ParameterRange,
    tolerance: f64,
    output: &mut Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    let bezier: Handle<Geom2dBezierCurve> = curve.cast();
    let Some(bezier) = bezier.get() else {
        return geom_lib::same_range(tolerance, curve, old, requested, output);
    };
    let mut input = curve.clone();
    if old.0.abs() > precision::p_confusion() || (old.1 - 1.0).abs() > precision::p_confusion() {
        let copied = Geom2dBezierCurve::copy_of(bezier)?;
        copied.segment(old.0, old.1)?;
        input = Handle::new(copied).cast();
    }
    geom_lib::same_range(tolerance, &input, (0.0, 1.0), requested, output)
}
