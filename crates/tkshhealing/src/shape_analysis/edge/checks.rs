// Rust translation of CheckCurve3dWithPCurve, CheckPoints, CheckVerticesWithCurve3d,
// CheckVerticesWithPCurve and CheckPCurveRange of ModelingAlgorithms/TKShHealing/ShapeAnalysis/
// ShapeAnalysis_Edge.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks of the ends of an edge against its curves.
use super::ShapeAnalysisEdge;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

/// The tolerance a vertex check uses: the vertex tolerance when `preci < 0`.
fn check_precision(vertex: &Vertex, preci: f64) -> Result<f64, Failure> {
    if preci < 0.0 {
        return BRepTool::vertex_tolerance(vertex);
    }
    Ok(preci)
}

impl ShapeAnalysisEdge {
    pub(super) fn set_status(&mut self, status: ShapeExtendStatus) {
        self.status = ShapeExtend::encode_status(status);
    }

    pub(super) fn add_status(&mut self, status: ShapeExtendStatus) {
        self.status |= ShapeExtend::encode_status(status);
    }

    /// `CheckCurve3dWithPCurve(edge, face)`; `sae.check_curve_3d_with_pcurve_on_face(&e, &f)?`.
    pub fn check_curve_3d_with_pcurve_on_face(
        &mut self,
        edge: &Edge,
        face: &Face,
    ) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.check_curve_3d_with_pcurve_on_surface(edge, &surface, &location)
    }

    /// `CheckCurve3dWithPCurve(edge, surface, location)`: true when the ends of the 3D
    /// curve and of the p-curve (both unoriented) do not match within the vertex
    /// tolerances, with DONE1 when they match better crossed (the orientations differ).
    /// False on a plane, FAIL1 without p-curve, FAIL2 without 3D curve, false without
    /// both vertices; `sae.check_curve_3d_with_pcurve_on_surface(&e, &s, &l)?`.
    pub fn check_curve_3d_with_pcurve_on_surface(
        &mut self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<bool, Failure> {
        self.set_status(ShapeExtendStatus::Ok);
        let surf = surface
            .get()
            .ok_or_else(|| null_surface("CheckCurve3dWithPCurve"))?;
        if !surface.cast::<GeomPlane>().is_null() {
            return Ok(false);
        }
        let Some(pc) = self.required_pcurve(edge, surface, location, false)? else {
            return Ok(false);
        };
        let Some(c3) = self.required_curve(edge, false, ShapeExtendStatus::Fail2)? else {
            return Ok(false);
        };
        let Some(tolerances) = self.endpoint_tolerances(edge)? else {
            return Ok(false);
        };
        let points = compared_curve_ends(&c3, &pc, surf, location)?;
        Ok(self.check_points(points, tolerances.0, tolerances.1))
    }

    /// The private `CheckPoints(P1A, P1B, P2A, P2B, preci1, preci2)`: false when the pairs
    /// (A, A) and (B, B) match within the precisions, else true with DONE1 when the
    /// crossed pairs are closer.
    fn check_points(&mut self, points: [Point3; 4], preci1: f64, preci2: f64) -> bool {
        let [p1a, p1b, p2a, p2b] = points;
        self.set_status(ShapeExtendStatus::Ok);
        if p1a.square_distance(&p2a) <= preci1 * preci1
            && p1b.square_distance(&p2b) <= preci2 * preci2
        {
            return false;
        }
        if p1a.distance(&p2b) + p1b.distance(&p2a) < p1a.distance(&p2a) + p1b.distance(&p2b) {
            self.add_status(ShapeExtendStatus::Done1);
        }
        true
    }

    /// Adds the raw enumerator `ShapeExtend_DONE1` or `DONE2` (natively
    /// `myStatus |= ShapeExtend_DONE1`, not encoded; the bits happen to be the encoded
    /// ones) when the vertex is farther than its precision from the point.
    fn check_end(
        &mut self,
        vertex: &Vertex,
        point: Point3,
        preci: f64,
        done: ShapeExtendStatus,
    ) -> Result<(), Failure> {
        let vertex_point = BRepTool::pnt(vertex)?;
        if vertex_point.distance(&point) > check_precision(vertex, preci)? {
            self.status |= done as i32;
        }
        Ok(())
    }

    /// `CheckVerticesWithCurve3d(edge, preci, vtx)`: checks the start (`vtx` 1), end (2)
    /// or both (0) vertices against the ends of the oriented 3D curve, within `preci` or
    /// their own tolerances when `preci < 0`: DONE1 for the start, DONE2 for the end,
    /// FAIL1 without 3D curve; true when a vertex is off;
    /// `sae.check_vertices_with_curve_3d(&edge, -1.0, 0)?`.
    pub fn check_vertices_with_curve_3d(
        &mut self,
        edge: &Edge,
        preci: f64,
        vtx: i32,
    ) -> Result<bool, Failure> {
        self.set_status(ShapeExtendStatus::Ok);
        let vertices = self.checked_vertices(edge)?;
        let Some((curve, first, last)) =
            self.required_curve(edge, true, ShapeExtendStatus::Fail1)?
        else {
            return Ok(false);
        };
        self.check_vertex_pair(&vertices, (first, last), preci, vtx, |u| curve.value(u))?;
        Ok(self.status_flag(ShapeExtendStatus::Done))
    }

    /// `CheckVerticesWithPCurve(edge, face, preci, vtx)`;
    /// `sae.check_vertices_with_pcurve_on_face(&edge, &face, -1.0, 0)?`.
    pub fn check_vertices_with_pcurve_on_face(
        &mut self,
        edge: &Edge,
        face: &Face,
        preci: f64,
        vtx: i32,
    ) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.check_vertices_with_pcurve_on_surface(edge, &surface, &location, preci, vtx)
    }

    /// `CheckVerticesWithPCurve(edge, surface, location, preci, vtx)`: as
    /// [`Self::check_vertices_with_curve_3d`] with the surface points of the ends of the
    /// oriented p-curve (FAIL1 without p-curve);
    /// `sae.check_vertices_with_pcurve_on_surface(&e, &s, &l, -1.0, 0)?`.
    pub fn check_vertices_with_pcurve_on_surface(
        &mut self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        preci: f64,
        vtx: i32,
    ) -> Result<bool, Failure> {
        self.set_status(ShapeExtendStatus::Ok);
        let vertices = self.checked_vertices(edge)?;
        let Some((curve, first, last)) = self.required_pcurve(edge, surface, location, true)?
        else {
            return Ok(false);
        };
        let surf = surface
            .get()
            .ok_or_else(|| null_surface("CheckVerticesWithPCurve"))?;
        self.check_vertex_pair(&vertices, (first, last), preci, vtx, |u| {
            let uv = curve.value(u)?;
            let point = surf.value(uv.x(), uv.y())?;
            Ok(if location.is_identity() {
                point
            } else {
                point.transformed(location.transformation())
            })
        })?;
        Ok(self.status_flag(ShapeExtendStatus::Done))
    }

    /// `CheckPCurveRange(theFirst, theLast, thePC)`: the range fits the p-curve (the basis
    /// of a trimmed curve): at most one period on a periodic curve, else inside its
    /// bounds within `PConfusion`; `sae.check_pcurve_range(0.0, 1.0, &pcurve)?`.
    pub fn check_pcurve_range(
        &mut self,
        first: f64,
        last: f64,
        pc: &Handle<dyn Geom2dCurve>,
    ) -> Result<bool, Failure> {
        let (periodic, period, first_bound, last_bound) = pcurve_bounds(pc)?;
        let epsilon = precision::p_confusion();
        if periodic && last - first > period + epsilon {
            return Ok(false);
        }
        Ok(periodic || !(first < first_bound - epsilon || last > last_bound + epsilon))
    }
}

fn null_surface(member: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("ShapeAnalysis_Edge::{member}: null surface; expected a Geom_Surface"),
    )
}

impl ShapeAnalysisEdge {
    fn required_curve(
        &mut self,
        edge: &Edge,
        orient: bool,
        failure: ShapeExtendStatus,
    ) -> Result<Option<super::Curve3dRange>, Failure> {
        let curve = self.curve_3d(edge, orient)?;
        if curve.0.is_null() {
            self.add_status(failure);
            return Ok(None);
        }
        Ok(Some(curve))
    }

    fn required_pcurve(
        &mut self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        orient: bool,
    ) -> Result<Option<super::PCurveRange>, Failure> {
        let curve = self.pcurve_on_surface(edge, surface, location, orient)?;
        if curve.0.is_null() {
            self.add_status(ShapeExtendStatus::Fail1);
            return Ok(None);
        }
        Ok(Some(curve))
    }

    fn checked_vertices(&self, edge: &Edge) -> Result<[Vertex; 2], Failure> {
        let vertices = [self.first_vertex(edge), self.last_vertex(edge)];
        // The native reads both points before looking at the curve.
        BRepTool::pnt(&vertices[0])?;
        BRepTool::pnt(&vertices[1])?;
        Ok(vertices)
    }

    fn check_vertex_pair(
        &mut self,
        vertices: &[Vertex; 2],
        range: (f64, f64),
        precision: f64,
        selected: i32,
        point_at: impl Fn(f64) -> Result<Point3, Failure>,
    ) -> Result<(), Failure> {
        if selected != 2 {
            self.check_end(
                &vertices[0],
                point_at(range.0)?,
                precision,
                ShapeExtendStatus::Done1,
            )?;
        }
        if selected != 1 {
            self.check_end(
                &vertices[1],
                point_at(range.1)?,
                precision,
                ShapeExtendStatus::Done2,
            )?;
        }
        Ok(())
    }
}

fn compared_curve_ends(
    curve: &super::Curve3dRange,
    pcurve: &super::PCurveRange,
    surface: &dyn GeomSurface,
    location: &Location,
) -> Result<[Point3; 4], Failure> {
    let uv1 = pcurve.0.value(pcurve.1)?;
    let uv2 = pcurve.0.value(pcurve.2)?;
    // #39 rln 17.11.98 S4054, annie_surf.igs entity 39: transform surface endpoints.
    let on_surface = |uv: rustycad_tkmath::gp::Point2| {
        surface
            .value(uv.x(), uv.y())
            .map(|p| p.transformed(location.transformation()))
    };
    Ok([
        curve.0.value(curve.1)?,
        curve.0.value(curve.2)?,
        on_surface(uv1)?,
        on_surface(uv2)?,
    ])
}

fn pcurve_bounds(pc: &Handle<dyn Geom2dCurve>) -> Result<(bool, f64, f64, f64), Failure> {
    let curve = pc.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ShapeAnalysis_Edge::CheckPCurveRange: null p-curve; expected a Geom2d_Curve",
        )
    })?;
    let mut bounds = raw_pcurve_bounds(curve)?;
    if let Some(trimmed) = pc.cast::<Geom2dTrimmedCurve>().get() {
        let basis = trimmed.basis_curve();
        if let Some(basis) = basis.get() {
            bounds = raw_pcurve_bounds(basis)?;
        }
    }
    Ok(bounds)
}

fn raw_pcurve_bounds(curve: &dyn Geom2dCurve) -> Result<(bool, f64, f64, f64), Failure> {
    let periodic = curve.is_periodic();
    let period = if periodic {
        curve.period()?
    } else {
        real_last()
    };
    Ok((
        periodic,
        period,
        curve.first_parameter(),
        curve.last_parameter(),
    ))
}

impl ShapeAnalysisEdge {
    fn endpoint_tolerances(&self, edge: &Edge) -> Result<Option<(f64, f64)>, Failure> {
        let first = self.first_vertex(edge);
        let last = self.last_vertex(edge);
        if first.is_null() || last.is_null() {
            return Ok(None);
        }
        Ok(Some((
            BRepTool::vertex_tolerance(&first)?,
            BRepTool::vertex_tolerance(&last)?,
        )))
    }
}
