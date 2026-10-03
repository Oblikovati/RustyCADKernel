//! Native adjacent notch classification in the p-curve parameter plane.
// ShapeAnalysis_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use crate::shape_analysis::{CurveProjection, ShapeAnalysisCurve};
use rustycad_tkbrep::{brep_tools, topods::Edge};
use rustycad_tkg2d::{geom2d::Geom2dCurve, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface},
    geom::GeomPlane,
    geom_adaptor::GeomAdaptorSurface,
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{self, Axis3, Point2, Point3, Vector2};

struct NotchCurve {
    curve: Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    orientation: Orientation,
    junction: (Point2, Vector2),
}

impl ShapeAnalysisWire {
    /// Locate the shorter overlapping adjacent edge and its projection parameter.
    /// Example: `analyzer.check_notched_edges(2, &mut short_index, &mut parameter, 1e-6)?`.
    pub fn check_notched_edges(
        &self,
        index: i32,
        short_index: &mut i32,
        parameter: &mut f64,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        let Some(curves) = self.prepare_notch_curves(index)? else {
            return Ok(false);
        };
        if !notch_tangents_match(&curves, tolerance)? {
            return Ok(false);
        }
        let indices = self.gap_indices(index);
        compare_notch_curves(&curves, indices, short_index, parameter, tolerance)
    }

    fn prepare_notch_curves(&self, index: i32) -> Result<Option<[NotchCurve; 2]>, Failure> {
        let (previous, current) = self.gap_pair(index)?;
        if BRepTool::degenerated(&previous)? || BRepTool::degenerated(&current)? {
            return Ok(None);
        }
        if !self.notch_vertices_connected(&previous, &current)? {
            return Ok(None);
        }
        let Some(first) = self.notch_curve(&previous, true)? else {
            return Ok(None);
        };
        let Some(second) = self.notch_curve(&current, false)? else {
            return Ok(None);
        };
        Ok(Some([first, second]))
    }

    fn notch_vertices_connected(&self, previous: &Edge, current: &Edge) -> Result<bool, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let first = analysis.last_vertex(previous);
        let last = analysis.first_vertex(current);
        if first.is_null() || last.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        if !brep_tools::compare_vertices(&first, &last)? {
            self.set_last_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        Ok(true)
    }

    fn notch_curve(&self, edge: &Edge, preceding: bool) -> Result<Option<NotchCurve>, Failure> {
        let (curve, first, last) =
            ShapeAnalysisEdge::new().pcurve_on_face(edge, &self.face(), false)?;
        if curve.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail3);
            return Ok(None);
        }
        let mut result = NotchCurve {
            curve,
            first,
            last,
            orientation: edge.orientation(),
            junction: (Point2::default(), Vector2::default()),
        };
        result.junction = result.junction_tangent(preceding)?;
        Ok(Some(result))
    }
}

impl NotchCurve {
    fn junction_tangent(&self, preceding: bool) -> Result<(Point2, Vector2), Failure> {
        let reversed = self.orientation == Orientation::Reversed;
        let parameter = if reversed == preceding {
            self.first
        } else {
            self.last
        };
        let (point, mut tangent) = self.curve.d1(parameter)?;
        if reversed != preceding {
            tangent.reverse();
        }
        Ok((point, tangent))
    }

    fn opposite_endpoint(&self, preceding: bool) -> Result<Point3, Failure> {
        let forward = self.orientation == Orientation::Forward;
        let parameter = if forward == preceding {
            self.first
        } else {
            self.last
        };
        let point = self.curve.value(parameter)?;
        Ok(Point3::new(point.x(), point.y(), 0.))
    }

    fn planar_adaptor(&self) -> Result<Adaptor3dCurveOnSurface, Failure> {
        let curve = Handle::new(Geom2dAdaptorCurve::with_range(
            &self.curve,
            self.first,
            self.last,
        )?);
        let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        let surface = Handle::new(GeomAdaptorSurface::with_surface(&plane)?);
        Adaptor3dCurveOnSurface::with_curve(&curve.cast(), &surface.cast())
    }
}

fn notch_tangents_match(curves: &[NotchCurve; 2], tolerance: f64) -> Result<bool, Failure> {
    let (first, first_tangent) = curves[0].junction;
    let (last, last_tangent) = curves[1].junction;
    if first_tangent.magnitude() < gp::resolution() || last_tangent.magnitude() < gp::resolution() {
        return Ok(false);
    }
    Ok(!(last_tangent.angle(first_tangent)?.abs() > 0.1 || first.distance(last) > tolerance))
}

fn compare_notch_curves(
    curves: &[NotchCurve; 2],
    indices: [i32; 2],
    short_index: &mut i32,
    parameter: &mut f64,
    tolerance: f64,
) -> Result<bool, Failure> {
    let adapted = [curves[0].planar_adaptor()?, curves[1].planar_adaptor()?];
    let opposite = [
        curves[1].opposite_endpoint(false)?,
        curves[0].opposite_endpoint(true)?,
    ];
    let mut first = notch_project_inside(&adapted[0], opposite[0], tolerance)?;
    let second = notch_project_inside(&adapted[1], opposite[1], tolerance)?;
    if first.0 > tolerance && second.0 > tolerance {
        return Ok(false);
    }
    let shorter = usize::from(first.0 < second.0);
    *short_index = indices[shorter];
    *parameter = if shorter == 1 {
        first.1.parameter
    } else {
        second.1.parameter
    };
    sample_notch_projection(
        &adapted[shorter],
        &adapted[1 - shorter],
        tolerance,
        &mut first.1,
    )
}

fn notch_project_inside(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    tolerance: f64,
) -> Result<(f64, CurveProjection), Failure> {
    let mut output = CurveProjection::default();
    let distance =
        ShapeAnalysisCurve.project_adaptor_into(curve, point, tolerance, false, &mut output)?;
    let first = curve.first_parameter()?;
    let last = curve.last_parameter()?;
    if output.parameter < first {
        output = CurveProjection {
            point: curve.value(first)?,
            parameter: first,
        };
        return Ok((output.point.distance(&point), output));
    }
    if output.parameter > last {
        output = CurveProjection {
            point: curve.value(last)?,
            parameter: last,
        };
        return Ok((output.point.distance(&point), output));
    }
    Ok((distance, output))
}

fn sample_notch_projection(
    short: &dyn Adaptor3dCurve,
    long: &dyn Adaptor3dCurve,
    tolerance: f64,
    output: &mut CurveProjection,
) -> Result<bool, Failure> {
    let mut parameter = short.first_parameter()?;
    let step = (short.last_parameter()? - parameter) / 23.;
    for _ in 1..23 {
        let distance = ShapeAnalysisCurve.project_adaptor_into(
            long,
            short.value(parameter)?,
            tolerance,
            true,
            output,
        )?;
        if distance > tolerance {
            return Ok(false);
        }
        parameter += step;
    }
    Ok(true)
}
