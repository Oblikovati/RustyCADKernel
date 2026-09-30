//! Caller-owned outputs for native branches that intentionally leave values untouched.
use super::*;

/// Writable outputs of GetEndTangent2d; a missing p-curve retains `point` and clears
/// `tangent`. For example, `let outputs = EdgeTangent2d::default();`.
#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeTangent2d {
    /// The evaluated point, or the caller's earlier point when no p-curve exists.
    pub point: Point2,
    /// The tangent; zero when the edge has no p-curve.
    pub tangent: Vector2,
}

impl ShapeAnalysisEdge {
    /// Evaluates a face tangent while retaining untouched native output values;
    /// `analyzer.get_end_tangent_2d_on_face_into(&edge, &face, false, 0.0, &mut outputs)?`.
    pub fn get_end_tangent_2d_on_face_into(
        &self,
        edge: &Edge,
        face: &Face,
        at_end: bool,
        dparam: f64,
        outputs: &mut EdgeTangent2d,
    ) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.get_end_tangent_2d_on_surface_into(edge, &surface, &location, at_end, dparam, outputs)
    }

    /// Evaluates a located-surface tangent with caller-owned point and tangent outputs;
    /// `analyzer.get_end_tangent_2d_on_surface_into(&edge, &surface, &location, false, 0.0, &mut outputs)?`.
    pub fn get_end_tangent_2d_on_surface_into(
        &self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        at_end: bool,
        dparam: f64,
        outputs: &mut EdgeTangent2d,
    ) -> Result<bool, Failure> {
        let (pcurve, first, last) = self.pcurve_on_surface(edge, surface, location, true)?;
        let Some(curve) = pcurve.get() else {
            outputs.tangent = Vector2::new(0.0, 0.0);
            return Ok(false);
        };
        let (found, point, tangent) =
            tangent::end_tangent(curve, (first, last), at_end, dparam, edge.orientation())?;
        *outputs = EdgeTangent2d { point, tangent };
        Ok(found)
    }

    /// Preserves both tolerances when an endpoint vertex is missing;
    /// `analyzer.check_vertex_tolerance_on_face_into(&edge, &face, &mut tolerances)?`.
    pub fn check_vertex_tolerance_on_face_into(
        &mut self,
        edge: &Edge,
        face: &Face,
        tolerances: &mut (f64, f64),
    ) -> Result<bool, Failure> {
        self.update_vertex_tolerance_outputs(edge, face, false, tolerances)
    }

    /// Checks every stored p-curve with native tolerance output preservation;
    /// `analyzer.check_vertex_tolerance_into(&edge, &mut tolerances)?`.
    pub fn check_vertex_tolerance_into(
        &mut self,
        edge: &Edge,
        tolerances: &mut (f64, f64),
    ) -> Result<bool, Failure> {
        self.update_vertex_tolerance_outputs(edge, &Face::default(), true, tolerances)
    }

    fn update_vertex_tolerance_outputs(
        &mut self,
        edge: &Edge,
        face: &Face,
        check_all: bool,
        tolerances: &mut (f64, f64),
    ) -> Result<bool, Failure> {
        let (status, first, last) =
            vertex_tolerance::check_vertex_tolerance(edge, face, check_all)?;
        self.status = status;
        if !self.status_flag(ShapeExtendStatus::Fail1) {
            *tolerances = (first, last);
        }
        Ok(self.status_flag(ShapeExtendStatus::Done))
    }
}
