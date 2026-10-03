//! Tail overlap detection and splitting from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use crate::shape_analysis::{CurveProjection, ShapeAnalysisCurve};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkg3d::{geom::GeomCurve, geom_adaptor::GeomAdaptorCurve, top_abs::Orientation};
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::gp::{Point3, Xyz};
mod bounds;
mod split;
use bounds::TailBounds;

struct TailCurve {
    geometry: Handle<dyn GeomCurve>,
    range: [f64; 2],
    junction: usize,
    point: Point3,
}

struct TailLimits {
    near: f64,
    far: f64,
    sample: f64,
}

impl ShapeAnalysisWire {
    /// Detect overlapping tails and write edge parts in `[edge11, edge12, edge21, edge22]` order.
    /// Example: `analyzer.check_tail(&first, &second, 0.1, 1e-6, 1e-3, &mut parts)?`.
    pub fn check_tail(
        &self,
        first: &Edge,
        second: &Edge,
        max_sine: f64,
        max_width: f64,
        max_tolerance: f64,
        parts: &mut [Edge; 4],
    ) -> Result<bool, Failure> {
        if !self.is_ready() || BRepTool::degenerated(first)? || BRepTool::degenerated(second)? {
            return Ok(false);
        }
        let Some(first_curve) = TailCurve::from_edge(first, 0)? else {
            return Ok(false);
        };
        let Some(second_curve) = TailCurve::from_edge(second, 1)? else {
            return Ok(false);
        };
        inspect_tail(
            [first, second],
            [first_curve, second_curve],
            max_sine,
            max_width,
            max_tolerance,
            parts,
        )
    }
}

fn inspect_tail(
    edges: [&Edge; 2],
    curves: [TailCurve; 2],
    max_sine: f64,
    max_width: f64,
    max_tolerance: f64,
    parts: &mut [Edge; 4],
) -> Result<bool, Failure> {
    let limits = TailLimits::new(max_width);
    if curves[0].point.square_distance(&curves[1].point) > limits.near * limits.near {
        return Ok(false);
    }
    if max_sine >= 0. && !tail_angle_matches(&curves, max_sine)? {
        return Ok(false);
    }
    let bounds = TailBounds::new(&curves, &limits)?;
    if !bounds.samples_match(&curves, &limits)? {
        return Ok(false);
    }
    if bounds.both_whole(&limits) {
        parts[0] = edges[0].clone();
        parts[2] = edges[1].clone();
        return Ok(true);
    }
    split::split_tail_edges(
        edges,
        &curves,
        bounds.cut_parameters(),
        max_tolerance,
        parts,
    )
}

impl TailCurve {
    fn from_edge(edge: &Edge, index: usize) -> Result<Option<Self>, Failure> {
        let (geometry, first, last) = ShapeAnalysisEdge::new().curve_3d(edge, false)?;
        if geometry.is_null() {
            return Ok(None);
        }
        let junction = if edge.orientation() == Orientation::Reversed {
            index
        } else {
            1 - index
        };
        let range = [first, last];
        let point = geometry.value(range[junction])?;
        Ok(Some(Self {
            geometry,
            range,
            junction,
            point,
        }))
    }

    fn direction_near_junction(&self) -> Result<Option<Xyz>, Failure> {
        let curve = GeomAdaptorCurve::with_curve(&self.geometry)?;
        let tolerance = 0.25 * precision::confusion();
        let length = AbscissaPoint::length_between_with_tolerance(
            &curve,
            self.range[0],
            self.range[1],
            tolerance,
        )?;
        if length < 0.5 * precision::confusion() {
            return Ok(None);
        }
        let signed_step = 0.5 * precision::confusion() * (1. - 2. * self.junction as f64);
        let step = AbscissaPoint::with_tolerance(
            tolerance,
            &curve,
            signed_step,
            self.range[self.junction],
        )?;
        if !step.is_done() {
            return Ok(None);
        }
        let mut points = [Xyz::default(); 2];
        points[self.junction] = self.point.xyz();
        points[1 - self.junction] = self.geometry.value(step.parameter()?)?.xyz();
        let direction = points[1] - points[0];
        let magnitude = direction.modulus();
        if magnitude < 0.1 * precision::confusion() {
            return Ok(None);
        }
        Ok(Some(direction * (1. / magnitude)))
    }

    fn project(&self, point: Point3, output: &mut CurveProjection) -> Result<f64, Failure> {
        let distance = ShapeAnalysisCurve.project_into(
            &self.geometry,
            point,
            0.25 * precision::confusion(),
            (self.range[0], self.range[1]),
            true,
            output,
        )?;
        if output.parameter >= self.range[0] && output.parameter <= self.range[1] {
            return Ok(distance);
        }
        let endpoints = [
            self.geometry.value(self.range[0])?,
            self.geometry.value(self.range[1])?,
        ];
        let distances = [point.distance(&endpoints[0]), point.distance(&endpoints[1])];
        let nearer = if distances[0] <= distances[1] { 0 } else { 1 };
        output.point = endpoints[nearer];
        output.parameter = self.range[nearer];
        Ok(distances[nearer])
    }
}

impl TailLimits {
    fn new(width: f64) -> Self {
        Self {
            near: width + 0.5 * precision::confusion(),
            far: width + precision::confusion(),
            sample: width + 1.5 * precision::confusion(),
        }
    }
}

fn tail_angle_matches(curves: &[TailCurve; 2], max_sine: f64) -> Result<bool, Failure> {
    let Some(mut first) = curves[0].direction_near_junction()? else {
        return Ok(false);
    };
    let Some(second) = curves[1].direction_near_junction()? else {
        return Ok(false);
    };
    if curves[0].junction ^ curves[1].junction != 0 {
        first.reverse();
    }
    Ok(!(first.dot(second) < 0. || first.cross_square_magnitude(second) > max_sine * max_sine))
}
