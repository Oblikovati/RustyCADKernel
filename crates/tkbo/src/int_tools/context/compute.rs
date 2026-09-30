use super::*;
use rustycad_tkbrep::topods::{ShapeIterator, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d::native::std_max;
impl Context {
    /// Projects and falls back to edge vertices when no extremum exists; `context.compute_pe(p, 1e-7, &edge, &mut t, &mut distance)?`.
    pub fn compute_pe(
        &self,
        point: Point3,
        tolerance: f64,
        edge: &Edge,
        parameter: &mut f64,
        distance: &mut f64,
    ) -> Result<i32, Failure> {
        if !BRepTool::is_geometric_edge(edge)? {
            return Ok(-2);
        }
        let service = self.proj_pc(edge)?;
        let mut projector = service.write();
        projector.perform(point)?;
        if projector.nb_points() == 0 {
            return self.edge_vertex_projection(point, tolerance, edge, parameter, distance);
        }
        *distance = projector.lower_distance()?;
        let limit = tolerance + BRepTool::edge_tolerance(edge)? + precision::confusion();
        *parameter = projector.lower_distance_parameter()?;
        Ok(if *distance > limit { -4 } else { 0 })
    }
    fn edge_vertex_projection(
        &self,
        point: Point3,
        tolerance: f64,
        edge: &Edge,
        parameter: &mut f64,
        distance: &mut f64,
    ) -> Result<i32, Failure> {
        let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
        *distance = f64::MAX;
        for child in ShapeIterator::new(&forward, true, true) {
            let vertex = rustycad_tkbrep::topods::vertex(&child)?;
            if !matches!(
                vertex.orientation(),
                Orientation::Forward | Orientation::Reversed
            ) {
                continue;
            }
            let candidate = point.distance(&BRepTool::pnt(&vertex)?);
            let limit = tolerance + BRepTool::vertex_tolerance(&vertex)? + precision::confusion();
            if candidate < *distance && candidate < limit {
                *distance = candidate;
                *parameter = BRepTool::parameter(&vertex, &forward)?;
            }
        }
        Ok(negative_status(precision::is_infinite(*distance), -3))
    }
    /// Preserves native early status codes and untouched outputs; `context.compute_ve(&vertex, &edge, &mut t, &mut tol, 0.)?`.
    pub fn compute_ve(
        &self,
        vertex: &Vertex,
        edge: &Edge,
        parameter: &mut f64,
        tolerance: &mut f64,
        fuzz: f64,
    ) -> Result<i32, Failure> {
        if BRepTool::degenerated(edge)? {
            return Ok(-1);
        }
        if !BRepTool::is_geometric_edge(edge)? {
            return Ok(-2);
        }
        let service = self.proj_pc(edge)?;
        let mut projector = service.write();
        projector.perform(BRepTool::pnt(vertex)?)?;
        if projector.nb_points() == 0 {
            return Ok(-3);
        }
        let distance = projector.lower_distance()?;
        let edge_tol = BRepTool::edge_tolerance(edge)?;
        let limit =
            BRepTool::vertex_tolerance(vertex)? + edge_tol + std_max(fuzz, precision::confusion());
        *tolerance = distance + edge_tol;
        *parameter = projector.lower_distance_parameter()?;
        Ok(if distance > limit { -4 } else { 0 })
    }
    /// Projects a vertex and requires its projection strictly inside the face; `context.compute_vf(&vertex, &face, &mut u, &mut v, &mut tol, 0.)?`.
    pub fn compute_vf(
        &self,
        vertex: &Vertex,
        face: &Face,
        u: &mut f64,
        v: &mut f64,
        tolerance: &mut f64,
        fuzz: f64,
    ) -> Result<i32, Failure> {
        let service = self.proj_ps(face)?;
        let mut projector = service.write();
        projector.perform(BRepTool::pnt(vertex)?)?;
        if !projector.is_done() {
            return Ok(-1);
        }
        let distance = projector.lower_distance()?;
        let face_tol = BRepTool::face_tolerance(face)?;
        let limit =
            BRepTool::vertex_tolerance(vertex)? + face_tol + std_max(fuzz, precision::confusion());
        *tolerance = distance + face_tol;
        (*u, *v) = projector.lower_distance_parameters()?;
        if distance > limit {
            return Ok(-2);
        }
        Ok(negative_status(
            !self.is_point_in_face(face, Point2::new(*u, *v))?,
            -3,
        ))
    }
    /// Updates the parameter only when a curve extremum exists; `context.project_point_on_edge(p, &edge, &mut t)?`.
    pub fn project_point_on_edge(
        &self,
        point: Point3,
        edge: &Edge,
        parameter: &mut f64,
    ) -> Result<bool, Failure> {
        let service = self.proj_pc(edge)?;
        let mut projector = service.write();
        projector.perform(point)?;
        if projector.nb_points() == 0 {
            return Ok(false);
        }
        *parameter = projector.lower_distance_parameter()?;
        Ok(true)
    }
}

fn negative_status(failed: bool, failure: i32) -> i32 {
    if failed {
        return failure;
    }
    0
}
