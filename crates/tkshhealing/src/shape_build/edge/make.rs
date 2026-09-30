// Rust translation of ShapeBuild_Edge::RemoveCurve3d, BuildCurve3d and the six MakeEdge
// members of ShapeBuild_Edge.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Building the 3D curve of an edge and making edges on curves and p-curves.
use super::{ShapeBuildEdge, is_periodic_3d};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_lib::{BUILD_CURVE3D_DEFAULTS, build_curve3d};

/// The curve of a `MakeEdge`, Standard_NullObject when null (a null dereference natively).
fn live<'a, T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    curve: &'a Handle<T>,
    what: &str,
) -> Result<&'a T, Failure> {
    curve.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("ShapeBuild_Edge::MakeEdge: null {what}; expected a curve"),
        )
    })
}

impl ShapeBuildEdge {
    /// `RemoveCurve3d(edge)`: removes the 3D curve of the edge (the tolerance passed is 0,
    /// S4136); `ShapeBuildEdge.remove_curve_3d(&e)?`.
    pub fn remove_curve_3d(&self, edge: &Edge) -> Result<(), Failure> {
        BRepBuilder::default().update_edge_curve(edge, &Handle::default(), 0.0)
    }

    /// `BuildCurve3d(edge)`: `BRepLib::BuildCurve3d` with a tolerance of at least 1e-5
    /// (#48 S4054: a C0 surface needs a C1 curve); on a SameRange edge the range is set on
    /// every representation (#50 S4054), and on a non-periodic curve a range beyond the
    /// curve is clamped to it (the edge loses SameRange). Any failure gives false, as the
    /// native `try/catch`; `ShapeBuildEdge.build_curve_3d(&e)`.
    pub fn build_curve_3d(&self, edge: &Edge) -> bool {
        self.try_build_curve_3d(edge).unwrap_or(false)
    }

    fn try_build_curve_3d(&self, edge: &Edge) -> Result<bool, Failure> {
        let (continuity, max_degree, max_segment) = BUILD_CURVE3D_DEFAULTS;
        let tolerance = 1.0e-5_f64.max(BRepTool::edge_tolerance(edge)?);
        if !build_curve3d(edge, tolerance, continuity, max_degree, max_segment)? {
            return Ok(false);
        }
        if BRepTool::same_range(edge)? {
            let (first, last) = BRepTool::range(edge)?;
            BRepBuilder::default().range(edge, first, last, false)?;
        }
        let (curve, first, last) = BRepTool::curve(edge)?;
        if curve.is_null() {
            return Ok(false);
        }
        if !is_periodic_3d(&curve) {
            self.clamp_range_3d(edge, &curve, first, last)?;
        }
        Ok(true)
    }

    /// The 15.11.2002 PTV OCC966 part of `BuildCurve3d`: a range beyond a non-periodic
    /// curve is clamped to it.
    fn clamp_range_3d(
        &self,
        edge: &Edge,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        let (mut f, mut l, mut is_less) = (first, last, false);
        if f < curve.first_parameter() {
            is_less = true;
            f = curve.first_parameter();
        }
        if l > curve.last_parameter() {
            is_less = true;
            l = curve.last_parameter();
        }
        if is_less {
            self.set_range_3d(edge, f, l)?;
            BRepBuilder::default().same_range(edge, false)?;
        }
        Ok(())
    }

    /// `MakeEdge(edge, curve, L)`: [`Self::make_edge_on_curve_range`] on the whole curve;
    /// `ShapeBuildEdge.make_edge_on_curve(&mut e, &c, &l)?`.
    pub fn make_edge_on_curve(
        &self,
        edge: &mut Edge,
        curve: &Handle<dyn GeomCurve>,
        location: &Location,
    ) -> Result<(), Failure> {
        let c = live(curve, "curve")?;
        self.make_edge_on_curve_range(
            edge,
            curve,
            location,
            c.first_parameter(),
            c.last_parameter(),
        )
    }

    /// `MakeEdge(edge, curve, L, p1, p2)`: `edge` becomes the edge
    /// `BRepBuilderAPI_MakeEdge(curve, p1, p2)` builds (unchanged when it fails); with a
    /// location, the curve is set again at that location and the vertices are moved by
    /// it; `ShapeBuildEdge.make_edge_on_curve_range(&mut e, &c, &l, 0.0, 1.0)?`.
    pub fn make_edge_on_curve_range(
        &self,
        edge: &mut Edge,
        curve: &Handle<dyn GeomCurve>,
        location: &Location,
        p1: f64,
        p2: f64,
    ) -> Result<(), Failure> {
        let maker = BRepBuilderAPIMakeEdge::from_curve_params(curve, p1, p2)?;
        if !maker.is_done() {
            return Ok(());
        }
        let made = maker.edge()?;
        if !location.is_identity() {
            let builder = BRepBuilder::default();
            builder.update_edge_located_curve(&made, curve, location, 0.0)?;
            builder.range(&made, p1, p2, false)?;
            move_vertices(&made, location)?;
        }
        *edge = made;
        Ok(())
    }

    /// `MakeEdge(edge, pcurve, face)`: [`Self::make_edge_on_face_range`] on the whole
    /// p-curve; `ShapeBuildEdge.make_edge_on_face(&mut e, &c, &f)?`.
    pub fn make_edge_on_face(
        &self,
        edge: &mut Edge,
        pcurve: &Handle<dyn Geom2dCurve>,
        face: &Face,
    ) -> Result<(), Failure> {
        let c = live(pcurve, "p-curve")?;
        self.make_edge_on_face_range(edge, pcurve, face, c.first_parameter(), c.last_parameter())
    }

    /// `MakeEdge(edge, pcurve, face, p1, p2)`: on the located surface of the face;
    /// `ShapeBuildEdge.make_edge_on_face_range(&mut e, &c, &f, 0.0, 1.0)?`.
    pub fn make_edge_on_face_range(
        &self,
        edge: &mut Edge,
        pcurve: &Handle<dyn Geom2dCurve>,
        face: &Face,
        p1: f64,
        p2: f64,
    ) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.make_edge_on_surface_range(edge, pcurve, &surface, &location, p1, p2)
    }

    /// `MakeEdge(edge, pcurve, S, L)`: on the whole p-curve;
    /// `ShapeBuildEdge.make_edge_on_surface(&mut e, &c, &s, &l)?`.
    pub fn make_edge_on_surface(
        &self,
        edge: &mut Edge,
        pcurve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(), Failure> {
        let c = live(pcurve, "p-curve")?;
        self.make_edge_on_surface_range(
            edge,
            pcurve,
            surface,
            location,
            c.first_parameter(),
            c.last_parameter(),
        )
    }

    /// `MakeEdge(edge, pcurve, S, L, p1, p2)`: `edge` becomes the edge
    /// `BRepBuilderAPI_MakeEdge(pcurve, S, p1, p2)` builds (unchanged when it fails); with a
    /// location, the p-curve is moved onto the located surface and the vertices are moved;
    /// `ShapeBuildEdge.make_edge_on_surface_range(&mut e, &c, &s, &l, 0.0, 1.0)?`.
    pub fn make_edge_on_surface_range(
        &self,
        edge: &mut Edge,
        pcurve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        p1: f64,
        p2: f64,
    ) -> Result<(), Failure> {
        let maker = BRepBuilderAPIMakeEdge::from_pcurve_params(pcurve, surface, p1, p2)?;
        if !maker.is_done() {
            return Ok(());
        }
        let made = maker.edge()?;
        if !location.is_identity() {
            self.remove_pcurve_on_surface(&made, surface)?;
            let builder = BRepBuilder::default();
            builder.update_edge_pcurve(&made, pcurve, surface, location, 0.0, None)?;
            builder.range_on_surface(&made, surface, location, p1, p2)?;
            move_vertices(&made, location)?;
        }
        *edge = made;
        Ok(())
    }
}

/// The end of the located `MakeEdge`: both vertices of the edge (TopExp::Vertices)
/// moved by the location, with a tolerance of 0 (the vertex tolerance is kept).
fn move_vertices(edge: &Edge, location: &Location) -> Result<(), Failure> {
    let (v1, v2) = topexp::vertices(edge, false);
    let transformation = location.transformation();
    let (p1, p2) = (BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
    let builder = BRepBuilder::default();
    let moved = |v: &Vertex, p: rustycad_tkmath::gp::Point3| {
        builder.update_vertex_point(v, p.transformed(transformation), 0.0)
    };
    moved(&v1, p1)?;
    moved(&v2, p2)
}
