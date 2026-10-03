use super::spot::ShapeAnalysisCheckSmallFace;
use crate::shape_analysis::curve::ShapeAnalysisCurve;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::vertices;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::Point3;

struct PinEdgeCurve {
    curve: Handle<dyn GeomCurve>,
    range: (f64, f64),
}

impl ShapeAnalysisCheckSmallFace {
    /// Tests the native extended-curve projection and endpoint derivative-angle gates.
    /// `analyzer.check_pin_edges(&short, &long, 100.0, 200.0, 1e-3)?`.
    pub fn check_pin_edges(
        &self,
        first_edge: &Edge,
        second_edge: &Edge,
        first_coefficient: f64,
        second_coefficient: f64,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let first = edge_curve(first_edge)?;
        let second = edge_curve(second_edge)?;
        let parameters = shared_parameters(first_edge, &first, &second, tolerance)?;
        self.check_pin_pair(
            (&first, &second),
            parameters,
            (first_coefficient, second_coefficient),
            tolerance,
        )
    }

    fn check_pin_pair(
        &self,
        curves: (&PinEdgeCurve, &PinEdgeCurve),
        parameters: (f64, f64),
        coefficients: (f64, f64),
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let target = extension_point(curves, parameters, coefficients)?;
        if !projects_inside_shorter(curves, target, tolerance)? {
            return Ok(false);
        }
        derivative_angle_gate(curves, parameters)
    }
}

fn edge_curve(edge: &Edge) -> Result<PinEdgeCurve, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    if curve.get().is_none() {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "ShapeAnalysis_CheckSmallFace::CheckPinEdges: curve is null; expected a 3D curve",
        ));
    }
    Ok(PinEdgeCurve {
        curve,
        range: (first, last),
    })
}

fn shared_parameters(
    first_edge: &Edge,
    first: &PinEdgeCurve,
    second: &PinEdgeCurve,
    tolerance: f64,
) -> Result<(f64, f64), Failure> {
    let shared_vertex = vertices(first_edge, true).1;
    let point = BRepTool::pnt(&shared_vertex)?;
    let tolerance = shared_tolerance(&shared_vertex, tolerance)?;
    Ok((
        parameter_at_point(&first.curve, first.range, point, tolerance)?,
        parameter_at_point(&second.curve, second.range, point, tolerance)?,
    ))
}

fn shared_tolerance(
    vertex: &rustycad_tkbrep::topods::Vertex,
    tolerance: f64,
) -> Result<f64, Failure> {
    if tolerance != -1.0 {
        return Ok(tolerance);
    }
    BRepTool::vertex_tolerance(vertex)
}

fn parameter_at_point(
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    point: Point3,
    tolerance: f64,
) -> Result<f64, Failure> {
    if curve.value(range.0)?.distance(&point) <= tolerance {
        return Ok(range.0);
    }
    if curve.value(range.1)?.distance(&point) <= tolerance {
        return Ok(range.1);
    }
    Ok(0.0)
}

fn extension_point(
    curves: (&PinEdgeCurve, &PinEdgeCurve),
    parameters: (f64, f64),
    coefficients: (f64, f64),
) -> Result<Point3, Failure> {
    let first_length = endpoint_chord(curves.0)?;
    let second_length = endpoint_chord(curves.1)?;
    if first_length < second_length {
        return curve_extension(curves.0, parameters.0, coefficients.0);
    }
    curve_extension(curves.1, parameters.1, coefficients.1)
}

fn endpoint_chord(curve: &PinEdgeCurve) -> Result<f64, Failure> {
    Ok(curve
        .curve
        .value(curve.range.0)?
        .distance(&curve.curve.value(curve.range.1)?))
}

fn curve_extension(
    curve: &PinEdgeCurve,
    shared_parameter: f64,
    coefficient: f64,
) -> Result<Point3, Failure> {
    let step = (curve.range.0 - curve.range.1) / coefficient;
    let parameter = if shared_parameter == curve.range.0 {
        shared_parameter + (coefficient - 3.0) * step
    } else {
        shared_parameter - 3.0 * step
    };
    curve.curve.value(parameter)
}

fn projects_inside_shorter(
    curves: (&PinEdgeCurve, &PinEdgeCurve),
    point: Point3,
    tolerance: f64,
) -> Result<bool, Failure> {
    let chosen = if endpoint_chord(curves.0)? < endpoint_chord(curves.1)? {
        &curves.0.curve
    } else {
        &curves.1.curve
    };
    let adaptor = GeomAdaptorCurve::with_curve(chosen)?;
    let (distance, _, parameter) =
        ShapeAnalysisCurve.project_adaptor(&adaptor, point, tolerance, false)?;
    Ok(distance <= tolerance
        && parameter >= chosen.first_parameter()
        && parameter <= chosen.last_parameter())
}

fn derivative_angle_gate(
    curves: (&PinEdgeCurve, &PinEdgeCurve),
    parameters: (f64, f64),
) -> Result<bool, Failure> {
    let first = curves.0.curve.d2(parameters.0)?;
    let second = curves.1.curve.d2(parameters.1)?;
    let angles = match (first.1.angle(second.1), first.2.angle(second.2)) {
        (Ok(first), Ok(second)) => (first, second),
        _ => return Ok(false),
    };
    // OCCT's antiparallel branch repeats a weaker check after the tighter one.
    let opposite_second = std::f64::consts::PI - angles.1 <= 0.001;
    Ok(angles.0 <= 0.001 && angles.1 <= 0.01 || opposite_second)
}
