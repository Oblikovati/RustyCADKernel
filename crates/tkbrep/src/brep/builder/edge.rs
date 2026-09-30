// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge curve members of `BRep_Builder`: the edge itself, its 3D curve, its
//! p-curves, its regularities and `Transfert`.
use super::update_curves::{update_curve3d, update_pcurve, update_pcurves, update_regularity};
use super::{BRepBuilder, check_unlocked, check_unlocked_shape, relative_to};
use crate::brep::access::edge_tshape;
use crate::brep::{BRepTEdge, BRepTool};
use crate::topods::{Edge, Face, TShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::toploc::Location;

/// The two p-curves of a seam edge.
pub type PCurvePair<'a> = (&'a Handle<dyn Geom2dCurve>, &'a Handle<dyn Geom2dCurve>);

impl BRepBuilder {
    /// `MakeEdge(E)`: an undefined edge (no geometry); `builder.make_edge(&mut edge)?`.
    pub fn make_edge(&self, edge: &mut Edge) -> Result<(), Failure> {
        let tedge = BRepTEdge::new();
        check_unlocked_shape(edge, "BRep_Builder::MakeEdge")?;
        self.make_shape(edge, &Handle::new(tedge).into_cast());
        Ok(())
    }

    /// `MakeEdge(E, C, Tol)`; `builder.make_edge_with_curve(&mut edge, &curve, 1e-7)?`.
    pub fn make_edge_with_curve(
        &self,
        edge: &mut Edge,
        curve: &Handle<dyn GeomCurve>,
        tolerance: f64,
    ) -> Result<(), Failure> {
        self.make_edge_with_located_curve(edge, curve, &Location::default(), tolerance)
    }

    /// `MakeEdge(E, C, L, Tol)`; `builder.make_edge_with_located_curve(&mut edge, &curve, &location, 1e-7)?`.
    pub fn make_edge_with_located_curve(
        &self,
        edge: &mut Edge,
        curve: &Handle<dyn GeomCurve>,
        location: &Location,
        tolerance: f64,
    ) -> Result<(), Failure> {
        self.make_edge(edge)?;
        self.update_edge_located_curve(edge, curve, location, tolerance)
    }

    /// `UpdateEdge(E, C, Tol)`: sets the 3D curve at the identity location;
    /// `builder.update_edge_curve(&edge, &curve, 1e-7)?`.
    pub fn update_edge_curve(
        &self,
        edge: &Edge,
        curve: &Handle<dyn GeomCurve>,
        tolerance: f64,
    ) -> Result<(), Failure> {
        self.update_edge_located_curve(edge, curve, &Location::default(), tolerance)
    }

    /// `UpdateEdge(E, C, L, Tol)`: sets the 3D curve (a null curve is stored as a null
    /// 3D curve) and raises the tolerance to `tolerance`;
    /// `builder.update_edge_located_curve(&edge, &curve, &location, 1e-7)?`.
    pub fn update_edge_located_curve(
        &self,
        edge: &Edge,
        curve: &Handle<dyn GeomCurve>,
        location: &Location,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::UpdateEdge")?;
        let relative = relative_to(location, edge)?;
        tedge.change_curves(|curves| update_curve3d(curves, curve, &relative))?;
        finish_edge_update(tedge, tolerance);
        Ok(())
    }

    /// `UpdateEdge(E, C, F, Tol)`: the p-curve on the face's located surface;
    /// `builder.update_edge_pcurve_on_face(&edge, &pcurve, &face, 1e-7)?`.
    pub fn update_edge_pcurve_on_face(
        &self,
        edge: &Edge,
        pcurve: &Handle<dyn Geom2dCurve>,
        face: &Face,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.update_edge_pcurve(edge, pcurve, &surface, &location, tolerance, None)
    }

    /// `UpdateEdge(E, C1, C2, F, Tol)`: the seam p-curves on the face's located surface;
    /// `builder.update_edge_pcurves_on_face(&edge, (&c1, &c2), &face, 1e-7)?`.
    pub fn update_edge_pcurves_on_face(
        &self,
        edge: &Edge,
        pcurves: PCurvePair<'_>,
        face: &Face,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.update_edge_pcurves(edge, pcurves, &surface, &location, tolerance, None)
    }

    /// `UpdateEdge(E, C, S, L, Tol)` and, with `uv`, `UpdateEdge(E, C, S, L, Tol, Pf, Pl)`:
    /// replaces the p-curves on (`surface`, `location`) by `pcurve` (a null one removes
    /// them); `builder.update_edge_pcurve(&edge, &pcurve, &surface, &location, 1e-7, None)?`.
    pub fn update_edge_pcurve(
        &self,
        edge: &Edge,
        pcurve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        tolerance: f64,
        uv: Option<(Point2, Point2)>,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::UpdateEdge")?;
        let relative = relative_to(location, edge)?;
        tedge.change_curves(|curves| update_pcurve(curves, pcurve, surface, &relative, uv))?;
        finish_edge_update(tedge, tolerance);
        Ok(())
    }

    /// `UpdateEdge(E, C1, C2, S, L, Tol)` and, with `uv`, the overload with `Pf, Pl`
    /// (which sets the UV points of the second p-curve): replaces the first curve on
    /// (`surface`, `location`) by a seam representation when both p-curves are
    /// non-null; `builder.update_edge_pcurves(&edge, (&c1, &c2), &surface, &location, 1e-7, None)?`.
    pub fn update_edge_pcurves(
        &self,
        edge: &Edge,
        pcurves: PCurvePair<'_>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        tolerance: f64,
        uv: Option<(Point2, Point2)>,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::UpdateEdge")?;
        let relative = relative_to(location, edge)?;
        tedge.change_curves(|curves| update_pcurves(curves, pcurves, surface, &relative, uv))?;
        finish_edge_update(tedge, tolerance);
        Ok(())
    }

    /// `UpdateEdge(E, Tol)`: raises the tolerance; `builder.update_edge_tolerance(&edge, 1e-5)?`.
    pub fn update_edge_tolerance(&self, edge: &Edge, tolerance: f64) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::UpdateEdge")?;
        finish_edge_update(tedge, tolerance);
        Ok(())
    }

    /// `Continuity(E, F1, F2, C)`: the regularity between the faces' located surfaces;
    /// `builder.continuity_on_faces(&edge, &f1, &f2, Shape::G1)?`.
    pub fn continuity_on_faces(
        &self,
        edge: &Edge,
        face1: &Face,
        face2: &Face,
        continuity: Continuity,
    ) -> Result<(), Failure> {
        let (surface1, location1) = BRepTool::surface_with_location(face1)?;
        let (surface2, location2) = BRepTool::surface_with_location(face2)?;
        self.continuity(
            edge,
            (&surface1, &surface2),
            (&location1, &location2),
            continuity,
        )
    }

    /// `Continuity(E, S1, S2, L1, L2, C)`: sets or adds the regularity between the
    /// located surfaces; `builder.continuity(&edge, (&s1, &s2), (&l1, &l2), Shape::C1)?`.
    pub fn continuity(
        &self,
        edge: &Edge,
        surfaces: (&Handle<dyn GeomSurface>, &Handle<dyn GeomSurface>),
        locations: (&Location, &Location),
        continuity: Continuity,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::Continuity")?;
        let relative1 = relative_to(locations.0, edge)?;
        let relative2 = relative_to(locations.1, edge)?;
        tedge.change_curves(|curves| {
            update_regularity(curves, surfaces, (&relative1, &relative2), continuity)
        })?;
        tedge.set_modified(true);
        Ok(())
    }

    /// `Transfert(Ein, Eout)`: adds to `edge_out` the p-curves and regularities of
    /// `edge_in`, with the raw tolerance of `edge_in`. A seam representation also
    /// answers `IsCurveOnSurface`, so only its first p-curve is transferred, as
    /// natively; `builder.transfert(&edge_in, &edge_out)?`.
    pub fn transfert(&self, edge_in: &Edge, edge_out: &Edge) -> Result<(), Failure> {
        let tedge = edge_tshape(edge_in, "BRep_Builder::Transfert")?;
        let tolerance = tedge.tolerance();
        for representation in tedge.curves() {
            let Some(representation) = representation.get() else {
                continue;
            };
            // The native code composes the location inside each branch only.
            let located = || edge_in.location() * &representation.location();
            if representation.is_curve_on_surface() {
                let (pcurve, surface) = (representation.pcurve()?, representation.surface()?);
                self.update_edge_pcurve(edge_out, &pcurve, &surface, &located()?, tolerance, None)?;
            } else if representation.is_curve_on_closed_surface() {
                let pcurves = (&representation.pcurve()?, &representation.pcurve2()?);
                let surface = representation.surface()?;
                self.update_edge_pcurves(
                    edge_out,
                    pcurves,
                    &surface,
                    &located()?,
                    tolerance,
                    None,
                )?;
            }
            if representation.is_regularity() {
                let surfaces = (&representation.surface()?, &representation.surface2()?);
                let location2 = (edge_in.location() * &representation.location2()?)?;
                let locations = (&located()?, &location2);
                self.continuity(edge_out, surfaces, locations, representation.continuity()?)?;
            }
        }
        Ok(())
    }
}

/// The `BRep_TEdge` of an edge that the builder may change: raises
/// TopoDS_LockedShape for a locked edge.
pub(super) fn locked_checked_edge<'a>(
    edge: &'a Edge,
    member: &str,
) -> Result<&'a BRepTEdge, Failure> {
    let tedge = edge_tshape(edge, member)?;
    check_unlocked(tedge, member)?;
    Ok(tedge)
}

/// `TE->UpdateTolerance(Tol); TE->Modified(true)`.
fn finish_edge_update(tedge: &BRepTEdge, tolerance: f64) {
    tedge.update_tolerance(tolerance);
    tedge.set_modified(true);
}
