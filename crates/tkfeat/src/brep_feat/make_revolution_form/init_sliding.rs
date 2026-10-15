// Rust translation of the sliding test of BRepFeat_MakeRevolutionForm::Init of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevolutionForm.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Whether a rib of revolution slides: its extreme faces planes, cylinders, cones or
//! tori, and the arcs its end points describe lying on them (or, for an end on an edge
//! of the basis, that edge a circle of the same axis and radius).
use super::BRepFeatMakeRevolutionForm;
use crate::brep_feat::StatusError;
use crate::brep_feat::rib_slot::Extremes;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane,
    GeomRectangularTrimmedSurface, GeomToroidalSurface,
};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::{Axis2, Circle3, Point3};
use rustycad_tktopalgo::brep_extrema::BRepExtremaExtCF;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// Whether the surface of the face is a plane, a cylinder, a cone or a torus.
fn of_revolution(face: &Face) -> Result<bool, Failure> {
    let mut s = BRepTool::surface(face)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    if let Some(t) = trimmed.get() {
        s = t.basis_surface();
    }
    Ok(!s.cast::<GeomPlane>().is_null()
        || !s.cast::<GeomCylindricalSurface>().is_null()
        || !s.cast::<GeomConicalSurface>().is_null()
        || !s.cast::<GeomToroidalSurface>().is_null())
}

/// The vertex at a point with the confusion tolerance.
fn vertex_at(p: Point3) -> Result<Vertex, Failure> {
    let mut v = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut v, p, precision::confusion())?;
    Ok(v)
}

/// Whether the arc touches the face (one extremum at most the confusion away).
fn arc_on(ee: &Edge, face: &Face) -> Result<bool, Failure> {
    let ext = BRepExtremaExtCF::with_edge_face(ee, face)?;
    Ok(!(ext.nb_ext() < 1 || ext.square_distance(1)? > precision::square_confusion()))
}

impl BRepFeatMakeRevolutionForm {
    /// Whether the edge of the basis under an end is a circle of the axis of radius
    /// `rayon`.
    fn coaxial_circle(&self, on_edge: &Edge, rayon: f64) -> Result<bool, Failure> {
        let (crv, _, _) = BRepTool::curve(on_edge)?;
        let circle: Handle<GeomCircle> = crv.cast();
        let Some(c) = circle.get() else {
            return Ok(false);
        };
        let circ = c.circ()?;
        if !circ
            .axis()
            .is_coaxial(self.axe, precision::confusion(), precision::confusion())
        {
            return Ok(false);
        }
        Ok((circ.radius() - rayon).abs() < precision::confusion())
    }

    /// The arc of the circle of the axis through `p` from `-angle2` to `angle1`, with
    /// its radius.
    fn arc_of(&self, center: Point3, rayon: f64, p: Point3) -> Result<Edge, Failure> {
        let ax2 = Axis2::new(center, self.axe.direction())?;
        let circ = Circle3::new(ax2, rayon)?;
        let v1 = vertex_at(p.rotated(self.axe, -self.angle2)?)?;
        let v2 = vertex_at(p.rotated(self.axe, self.angle1)?)?;
        BRepLibMakeEdge::from_circ_vertices(circ, &v1, &v2)?.edge()
    }

    /// Whether the rib slides (None after NoProjPt): see the module.
    pub(super) fn sliding_allowed(
        &mut self,
        modify: bool,
        x: &Extremes,
    ) -> Result<Option<bool>, Failure> {
        let mut sliding = modify && of_revolution(&x.first_face)?;
        if sliding {
            sliding = of_revolution(&x.last_face)?;
        }
        if !sliding {
            return Ok(Some(false));
        }
        // Control only start and end points no control at the middle to improve
        let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(self.axe)).cast();
        let proj = ProjectPointOnCurve::with_curve(self.rib.first_pnt, &line)?;
        if proj.nb_points() < 1 {
            self.fail(StatusError::NoProjPt);
            return Ok(None);
        }
        let (first_rayon, first_center) = (proj.distance(1)?, proj.point(1)?);
        // natively the projection of the first point is tested again here
        let proj1 = ProjectPointOnCurve::with_curve(self.rib.last_pnt, &line)?;
        let (last_rayon, last_center) = (proj1.distance(1)?, proj1.point(1)?);
        let ee1 = self.arc_of(first_center, first_rayon, self.rib.first_pnt)?;
        let ee2 = self.arc_of(last_center, last_rayon, self.rib.last_pnt)?;
        if sliding && !x.pt_on_first_edge {
            sliding = arc_on(&ee1, &x.first_face)?;
        }
        if sliding && !x.pt_on_last_edge {
            sliding = arc_on(&ee2, &x.last_face)?;
        }
        if sliding && x.pt_on_first_edge {
            sliding = self.coaxial_circle(&x.on_first_edge, first_rayon)?;
        }
        if sliding && x.pt_on_last_edge {
            sliding = self.coaxial_circle(&x.on_last_edge, last_rayon)?;
        }
        Ok(Some(sliding))
    }
}
