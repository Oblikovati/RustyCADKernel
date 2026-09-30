// Rust translation of the static CheckVertexTolerance and the two CheckVertexTolerance members of
// ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_Edge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
//
// :o8 abv 19.02.99: CTS18541.stp #18559: coefficient 1.0001 added in CheckVertexTol.

//! The tolerances the vertices of an edge need to hold the ends of its curves.
use super::ShapeAnalysisEdge;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

/// The squared distances of the vertex points to the ends of one p-curve.
struct EndGaps {
    pnt1: Point3,
    pnt2: Point3,
    toler1: f64,
    toler2: f64,
}

impl EndGaps {
    /// Widens the squared tolerances by the ends of the oriented p-curve on the surface.
    fn add_pcurve(
        &mut self,
        sae: &ShapeAnalysisEdge,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<bool, Failure> {
        let (pcurve, first, last) = sae.pcurve_on_surface(edge, surface, location, true)?;
        let Some(curve) = pcurve.get() else {
            return Ok(false);
        };
        let surf = surface.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ShapeAnalysis_Edge::CheckVertexTolerance: null surface; expected a Geom_Surface",
            )
        })?;
        self.add_surface_points(curve.value(first)?, curve.value(last)?, surf, location)?;
        Ok(true)
    }
}

/// The static `CheckVertexTolerance(edge, face, checkAll, toler1, toler2)`: the status
/// and the two tolerances. Natively the tolerances are left untouched when a vertex is
/// missing (FAIL1); they are 0 here then.
pub(super) fn check_vertex_tolerance(
    edge: &Edge,
    face: &Face,
    check_all: bool,
) -> Result<(i32, f64, f64), Failure> {
    let mut status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
    let sae = ShapeAnalysisEdge::new();
    let (v1, v2) = (sae.first_vertex(edge), sae.last_vertex(edge));
    if v1.is_null() || v2.is_null() {
        // :p1 abv 22 Feb 99: r76sy.stp
        return Ok((
            status | ShapeExtend::encode_status(ShapeExtendStatus::Fail1),
            0.0,
            0.0,
        ));
    }
    let previous = (
        BRepTool::vertex_tolerance(&v1)?,
        BRepTool::vertex_tolerance(&v2)?,
    );
    let mut gaps = EndGaps::from_vertices(BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
    gaps.add_curve(&sae, edge, &mut status)?;
    gaps.add_surfaces(&sae, edge, face, check_all, &mut status)?;
    gaps.finish(edge, previous, status)
}

/// The ends of every curve on surface stored in the edge.
fn all_pcurves(sae: &ShapeAnalysisEdge, edge: &Edge, gaps: &mut EndGaps) -> Result<(), Failure> {
    for representation in edge_tshape(edge, "ShapeAnalysis_Edge::CheckVertexTolerance")?.curves() {
        let gcurve = as_gcurve(&representation);
        let Some(gc) = gcurve.get().filter(|gc| gc.is_curve_on_surface()) else {
            continue;
        };
        let surface = gc.surface()?;
        let location = (edge.location() * &gc.location())?;
        gaps.add_pcurve(sae, edge, &surface, &location)?;
    }
    Ok(())
}

impl ShapeAnalysisEdge {
    /// `CheckVertexTolerance(edge, face, toler1, toler2)`: the tolerances the start and end
    /// vertices need to hold the ends of the 3D curve and of the p-curve on the face
    /// (1.0000001 times the largest distance, at least the edge tolerance): DONE1/DONE2
    /// when larger than the current ones, FAIL1 without vertices, FAIL2 without 3D curve
    /// (unless degenerated), FAIL3 without p-curve; true for DONE;
    /// `sae.check_vertex_tolerance_on_face(&edge, &face)?`.
    pub fn check_vertex_tolerance_on_face(
        &mut self,
        edge: &Edge,
        face: &Face,
    ) -> Result<(bool, f64, f64), Failure> {
        let mut tolerances = (0.0, 0.0);
        let changed = self.check_vertex_tolerance_on_face_into(edge, face, &mut tolerances)?;
        Ok((changed, tolerances.0, tolerances.1))
    }

    /// `CheckVertexTolerance(edge, toler1, toler2)`: the same with every p-curve stored in
    /// the edge; `sae.check_vertex_tolerance(&edge)?`.
    pub fn check_vertex_tolerance(&mut self, edge: &Edge) -> Result<(bool, f64, f64), Failure> {
        let mut tolerances = (0.0, 0.0);
        let changed = self.check_vertex_tolerance_into(edge, &mut tolerances)?;
        Ok((changed, tolerances.0, tolerances.1))
    }
}

impl EndGaps {
    fn from_vertices(pnt1: Point3, pnt2: Point3) -> Self {
        Self {
            pnt1,
            pnt2,
            toler1: 0.0,
            toler2: 0.0,
        }
    }

    fn add_surface_points(
        &mut self,
        uv1: rustycad_tkmath::gp::Point2,
        uv2: rustycad_tkmath::gp::Point2,
        surface: &dyn GeomSurface,
        location: &Location,
    ) -> Result<(), Failure> {
        let transform = location.transformation();
        let point1 = surface.value(uv1.x(), uv1.y())?.transformed(transform);
        let point2 = surface.value(uv2.x(), uv2.y())?.transformed(transform);
        self.toler1 = self.toler1.max(self.pnt1.square_distance(&point1));
        self.toler2 = self.toler2.max(self.pnt2.square_distance(&point2));
        Ok(())
    }

    fn add_curve(
        &mut self,
        analyzer: &ShapeAnalysisEdge,
        edge: &Edge,
        status: &mut i32,
    ) -> Result<(), Failure> {
        let (curve, first, last) = analyzer.curve_3d(edge, true)?;
        if let Some(curve) = curve.get() {
            self.toler1 = self.pnt1.square_distance(&curve.value(first)?);
            self.toler2 = self.pnt2.square_distance(&curve.value(last)?);
        } else if !BRepTool::degenerated(edge)? {
            *status |= ShapeExtend::encode_status(ShapeExtendStatus::Fail2);
        }
        Ok(())
    }

    fn add_surfaces(
        &mut self,
        analyzer: &ShapeAnalysisEdge,
        edge: &Edge,
        face: &Face,
        check_all: bool,
        status: &mut i32,
    ) -> Result<(), Failure> {
        if check_all {
            return all_pcurves(analyzer, edge, self);
        }
        if face.is_null() {
            return Ok(());
        }
        // :abv 10.06.02: C40 -> dev (CC670-12608.stp): include planes without stored p-curves.
        let (surface, location) = BRepTool::surface_with_location(face)?;
        if !self.add_pcurve(analyzer, edge, &surface, &location)? {
            *status |= ShapeExtend::encode_status(ShapeExtendStatus::Fail3);
        }
        Ok(())
    }

    fn finish(
        &self,
        edge: &Edge,
        previous: (f64, f64),
        mut status: i32,
    ) -> Result<(i32, f64, f64), Failure> {
        // szv 18 Aug 99: the edge tolerance is taken in consideration.
        let edge_tolerance = BRepTool::edge_tolerance(edge)?;
        let first = (1.0000001 * self.toler1.sqrt()).max(edge_tolerance);
        let last = (1.0000001 * self.toler2.sqrt()).max(edge_tolerance);
        if first > previous.0 {
            status |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
        }
        if last > previous.1 {
            status |= ShapeExtend::encode_status(ShapeExtendStatus::Done2);
        }
        Ok((status, first, last))
    }
}
