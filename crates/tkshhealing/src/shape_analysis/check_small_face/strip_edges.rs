use super::spot::ShapeAnalysisCheckSmallFace;
use crate::shape_analysis::curve::ShapeAnalysisCurve;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;

struct TrimmedEdgeCurve {
    geometry: Handle<dyn GeomCurve>,
    range: (f64, f64),
}

impl ShapeAnalysisCheckSmallFace {
    /// Compares both trimmed edge curves at eleven native samples per direction.
    /// `analyzer.check_strip_edges(&first, &second, 1e-3, &mut maximum)?`.
    pub fn check_strip_edges(
        &self,
        first_edge: &Edge,
        second_edge: &Edge,
        tolerance: f64,
        maximum_distance: &mut f64,
    ) -> Result<bool, Failure> {
        let tolerance = comparison_tolerance(first_edge, second_edge, tolerance)?;
        *maximum_distance = 0.0;
        // Both curves are read and null-checked before either is trimmed, as natively.
        let first = BRepTool::curve(first_edge)?;
        let second = BRepTool::curve(second_edge)?;
        if first.0.is_null() || second.0.is_null() {
            return Ok(false);
        }
        let first = trimmed_edge_curve(first)?;
        let second = trimmed_edge_curve(second)?;
        sample_curve_pair((&first, &second), tolerance, maximum_distance)
    }
}

fn comparison_tolerance(first: &Edge, second: &Edge, tolerance: f64) -> Result<f64, Failure> {
    if tolerance < 0.0 {
        return edge_tolerance_half_sum(first, second, tolerance);
    }
    Ok(tolerance)
}

fn edge_tolerance_half_sum(first: &Edge, second: &Edge, tolerance: f64) -> Result<f64, Failure> {
    let edge_tolerances = BRepTool::edge_tolerance(first)? + BRepTool::edge_tolerance(second)?;
    let half_sum = edge_tolerances / 2.0;
    Ok(if tolerance < half_sum {
        half_sum
    } else {
        tolerance
    })
}

fn trimmed_edge_curve(
    (curve, first, last): (Handle<dyn GeomCurve>, f64, f64),
) -> Result<TrimmedEdgeCurve, Failure> {
    let geometry = curve
        .get()
        .ok_or_else(|| Failure::new("CheckStripEdges: null edge curve; expected a 3D curve"))?;
    let first = native_max(first, geometry.first_parameter());
    let last = native_min(last, geometry.last_parameter());
    let trimmed: Handle<GeomTrimmedCurve> =
        Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?);
    let bounded = trimmed.cast::<dyn GeomCurve>();
    let parameters = (trimmed.first_parameter(), trimmed.last_parameter());
    Ok(TrimmedEdgeCurve {
        geometry: bounded,
        range: parameters,
    })
}

fn sample_curve_pair(
    curves: (&TrimmedEdgeCurve, &TrimmedEdgeCurve),
    tolerance: f64,
    maximum_distance: &mut f64,
) -> Result<bool, Failure> {
    let forward = sample_one_direction(
        curves.0,
        curves.1,
        span_step(curves.0),
        tolerance,
        maximum_distance,
    )?;
    if !forward {
        return Ok(false);
    }
    let reverse = sample_one_direction(
        curves.1,
        curves.0,
        span_step(curves.1),
        tolerance,
        maximum_distance,
    )?;
    Ok(reverse && *maximum_distance < tolerance)
}

fn span_step(curve: &TrimmedEdgeCurve) -> f64 {
    (curve.range.1 - curve.range.0) / 10.0
}

fn sample_one_direction(
    source: &TrimmedEdgeCurve,
    target: &TrimmedEdgeCurve,
    source_step: f64,
    tolerance: f64,
    maximum_distance: &mut f64,
) -> Result<bool, Failure> {
    let adaptor = GeomAdaptorCurve::with_curve(&target.geometry)?;
    // The native parameter accumulates `u += cd1`, so its rounding differs from
    // `first + i * step`.
    let mut parameter = source.range.0;
    for _ in 0..=10 {
        let point = source.geometry.value(parameter)?;
        // ShapeAnalysis_Curve::Project with its default AdjustToEnds = true.
        let (distance, _, target_parameter) =
            ShapeAnalysisCurve.project_adaptor(&adaptor, point, tolerance, true)?;
        if target_parameter < target.range.0 || target_parameter > target.range.1 {
            return Ok(false);
        }
        *maximum_distance = if distance > *maximum_distance {
            distance
        } else {
            *maximum_distance
        };
        if distance > tolerance {
            return Ok(false);
        }
        parameter += source_step;
    }
    Ok(true)
}

fn native_max(first: f64, second: f64) -> f64 {
    if second > first { second } else { first }
}

fn native_min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}
