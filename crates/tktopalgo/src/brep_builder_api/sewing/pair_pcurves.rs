// Rust translation of the p-curve loop of BRepBuilderAPI_Sewing::SameParameterEdge(edge1,
// edge2, listFaces1, listFaces2, secForward, whichSec, firstCall)
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The p-curves of the merged edge: those of the reference section on its faces, and
//! those of the other section, reversed when the sections run opposite and brought to
//! the reference range, on the faces of the other section.
use super::BRepBuilderAPISewing;
use super::failures::null_object;
use super::pair_edge::SectionPair;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::toploc::Location;

/// The outcome of the loop: a seam in manifold mode rejects the merge; otherwise
/// whether a p-curve of the other section was stored.
pub(super) enum PcurveTransfer {
    Rejected,
    Done(bool),
}

/// A p-curve of the other section on one of its faces, with the surface of the face.
struct OtherPcurve {
    face: Face,
    surface: Handle<dyn GeomSurface>,
    location: Location,
    is_seam: bool,
    c2d: Handle<dyn Geom2dCurve>,
    c2d_seam: Handle<dyn Geom2dCurve>,
}

/// A private copy of a p-curve, as `down_cast<Geom2d_Curve>(C->Copy())`.
fn copied(c: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let curve = c
        .get()
        .ok_or_else(|| null_object("SameParameterEdge: null p-curve"))?;
    Ok(curve.copy()?.cast::<dyn Geom2dCurve>())
}

/// The reversal of a p-curve of an opposite section: a line is first trimmed to its
/// range; the range is reversed with the curve.
fn reversed_pcurve(
    c: Handle<dyn Geom2dCurve>,
    (first_old, last_old): (f64, f64),
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let mut c = c;
    if c.cast::<Geom2dLine>().get().is_some() {
        c = Handle::new(Geom2dTrimmedCurve::new(
            &c, first_old, last_old, true, true,
        )?)
        .cast();
    }
    let curve = c
        .get()
        .ok_or_else(|| null_object("SameParameterEdge: null p-curve"))?;
    let first = curve.reversed_parameter(last_old)?;
    let last = curve.reversed_parameter(first_old)?;
    curve.reverse()?;
    Ok((c, first, last))
}

impl BRepBuilderAPISewing {
    /// The loop over the faces of the other section (`itf2`), each one storing its
    /// p-curves on the merged edge along with those of the reference section.
    pub(super) fn transfer_pcurves(
        &self,
        sections: SectionPair<'_>,
        edge: &Edge,
        sec_forward: bool,
        range: (f64, f64),
    ) -> Result<PcurveTransfer, Failure> {
        let (edge1, edge2, faces1, faces2) = sections;
        let mut is_res_edge = false;
        for face in faces2.iter() {
            let fac2 = topods::face(face)?;
            let Some(other) = self.other_pcurve(edge2, &fac2, sec_forward, range)? else {
                return Ok(PcurveTransfer::Rejected);
            };
            if other.c2d.is_null() {
                continue;
            }
            match self.store_on_faces(edge1, edge2, faces1, edge, &other, sec_forward, range)? {
                PcurveTransfer::Rejected => return Ok(PcurveTransfer::Rejected),
                PcurveTransfer::Done(stored) => is_res_edge |= stored,
            }
        }
        Ok(PcurveTransfer::Done(is_res_edge))
    }

    /// The p-curves of the other section on `fac2`, copied, reversed for an opposite
    /// section and brought to the reference range; None for a seam in manifold mode, a
    /// null `c2d` when the face has none or the range change fails.
    fn other_pcurve(
        &self,
        edge2: &Edge,
        fac2: &Face,
        sec_forward: bool,
        (first, last): (f64, f64),
    ) -> Result<Option<OtherPcurve>, Failure> {
        let (surface, location) = BRepTool::surface_with_location(fac2)?;
        let is_seam = self.is_closed_surface(&surface, edge2, &location)?
            && BRepTool::is_closed_on_face(edge2, fac2)?;
        let mut other = OtherPcurve {
            face: fac2.clone(),
            surface,
            location,
            is_seam,
            c2d: Handle::default(),
            c2d_seam: Handle::default(),
        };
        let mut seam = Handle::default();
        if is_seam {
            if !self.my_nonmanifold {
                return Ok(None);
            }
            seam = BRepTool::curve_on_face(&Edge::unchecked(edge2.reversed()), fac2)?.pcurve;
        }
        let found = BRepTool::curve_on_face(edge2, fac2)?;
        let (mut first_old, mut last_old) = (found.first, found.last);
        if found.pcurve.is_null() && seam.is_null() {
            return Ok(Some(other));
        }
        if !seam.is_null() {
            let mut c = copied(&seam)?;
            if !sec_forward {
                (c, first_old, last_old) = reversed_pcurve(c, (first_old, last_old))?;
            }
            other.c2d_seam = self.same_range_pcurve(&c, (first_old, last_old), (first, last));
        }
        // A seam p-curve without its partner is a null dereference natively.
        let mut c = copied(&found.pcurve)?;
        if !sec_forward {
            (c, first_old, last_old) = reversed_pcurve(c, (first_old, last_old))?;
        }
        other.c2d = self.same_range_pcurve(&c, (first_old, last_old), (first, last));
        Ok(Some(other))
    }

    /// The loop over the faces of the reference section (`itf1`): each stores its
    /// p-curves on the merged edge, then the other p-curve is stored on the other
    /// surface (as a seam pair when the two p-curves are far apart on the same closed
    /// surface), until a seam is made.
    #[allow(clippy::too_many_arguments)]
    fn store_on_faces(
        &self,
        edge1: &Edge,
        edge2: &Edge,
        faces1: &rustycad_tkernel::ncollection::List<rustycad_tkbrep::topods::Shape>,
        edge: &Edge,
        other: &OtherPcurve,
        sec_forward: bool,
        range: (f64, f64),
    ) -> Result<PcurveTransfer, Failure> {
        let mut stored = false;
        for face in faces1.iter() {
            let fac1 = topods::face(face)?;
            let Some((c2d1, ori)) = self.store_reference(edge1, &fac1, edge)? else {
                return Ok(PcurveTransfer::Rejected);
            };
            let Some(c2d1) = c2d1 else { continue };
            let Some(is_seam) = self.seam_across(edge2, edge, &fac1, &c2d1, other, range)? else {
                continue;
            };
            stored = true;
            self.store_other(edge, edge2, other, &c2d1, (is_seam, ori), sec_forward)?;
            if is_seam {
                break;
            }
        }
        Ok(PcurveTransfer::Done(stored))
    }

    /// The p-curves of the reference section on `fac1` stored on the merged edge (a
    /// pair on a seam, in non-manifold mode only); None rejects the merge, an inner
    /// None skips the face (no p-curve). The orientation of the reference in the face
    /// comes along.
    #[allow(clippy::type_complexity)]
    fn store_reference(
        &self,
        edge1: &Edge,
        fac1: &Face,
        edge: &Edge,
    ) -> Result<Option<(Option<Handle<dyn Geom2dCurve>>, Orientation)>, Failure> {
        let builder = BRepBuilder::default();
        let (surf1, loc1) = BRepTool::surface_with_location(fac1)?;
        let is_seam1 = self.is_closed_surface(&surf1, edge1, &loc1)?
            && BRepTool::is_closed_on_face(edge1, fac1)?;
        let c2d1 = BRepTool::curve_on_face(edge1, fac1)?.pcurve;
        let mut ori = edge1.orientation();
        if fac1.orientation() == Orientation::Reversed {
            ori = top_abs::reverse(ori);
        }
        let mut c2d11: Handle<dyn Geom2dCurve> = Handle::default();
        if is_seam1 {
            if !self.my_nonmanifold {
                return Ok(None);
            }
            c2d11 = BRepTool::curve_on_face(&Edge::unchecked(edge1.reversed()), fac1)?.pcurve;
            let pair = if ori == Orientation::Forward {
                (&c2d1, &c2d11)
            } else {
                (&c2d11, &c2d1)
            };
            builder.update_edge_pcurves_on_face(edge, pair, fac1, 0.0)?;
        } else {
            builder.update_edge_pcurve_on_face(edge, &c2d1, fac1, 0.0)?;
        }
        if c2d1.is_null() && c2d11.is_null() {
            return Ok(Some((None, ori)));
        }
        Ok(Some((Some(c2d1), ori)))
    }

    /// On the same surface at the same location, closed along the section: whether
    /// the two p-curves are more than 0.75 of the period apart (a seam); None skips a
    /// face on which the merged edge is already closed without making a seam.
    fn seam_across(
        &self,
        edge2: &Edge,
        edge: &Edge,
        fac1: &Face,
        c2d1: &Handle<dyn Geom2dCurve>,
        other: &OtherPcurve,
        (first, last): (f64, f64),
    ) -> Result<Option<bool>, Failure> {
        let (surf1, loc1) = BRepTool::surface_with_location(fac1)?;
        if other.surface != surf1 || other.location.is_different(&loc1) {
            return Ok(Some(false));
        }
        let u_closed = self.is_u_closed_surface(&other.surface, edge2, &other.location)?;
        let v_closed = self.is_v_closed_surface(&other.surface, edge2, &other.location)?;
        if !u_closed && !v_closed {
            return Ok(Some(false));
        }
        let c1 = c2d1
            .get()
            .ok_or_else(|| null_object("SameParameterEdge: null p-curve"))?;
        let c2 = other
            .c2d
            .get()
            .ok_or_else(|| null_object("SameParameterEdge: null p-curve"))?;
        let p1n = c1.value(std_max(first, c1.first_parameter()))?;
        let p21n = c2.value(std_max(first, c2.first_parameter()))?;
        let p22n = c2.value(std_min(last, c2.last_parameter()))?;
        let dist: f64 = std_min(p1n.distance(p21n), p1n.distance(p22n));
        let s = other
            .surface
            .get()
            .ok_or_else(|| null_object("SameParameterEdge: null surface"))?;
        let (u1, u2, v1, v2) = s.bounds();
        let is_seam = (u_closed && dist > 0.75 * (u2 - u1).abs())
            || (v_closed && dist > 0.75 * (v2 - v1).abs());
        if !is_seam && BRepTool::is_closed_on_face(edge, fac1)? {
            return Ok(None);
        }
        Ok(Some(is_seam))
    }

    /// Stores the other p-curve on its surface: with the reference one as a seam pair,
    /// as the pair of the other section's seam, or alone.
    fn store_other(
        &self,
        edge: &Edge,
        edge2: &Edge,
        other: &OtherPcurve,
        c2d1: &Handle<dyn Geom2dCurve>,
        (is_seam, ori): (bool, Orientation),
        sec_forward: bool,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let (s, l, tol) = (&other.surface, &other.location, precision::confusion());
        if is_seam {
            let pair = if ori == Orientation::Forward {
                (c2d1, &other.c2d)
            } else {
                (&other.c2d, c2d1)
            };
            return builder.update_edge_pcurves(edge, pair, s, l, tol, None);
        }
        if !other.is_seam {
            return builder.update_edge_pcurve(edge, &other.c2d, s, l, tol, None);
        }
        let mut init_ori = edge2.orientation();
        if other.face.orientation() == Orientation::Reversed {
            init_ori = top_abs::reverse(init_ori);
        }
        if !sec_forward {
            init_ori = top_abs::reverse(init_ori);
        }
        let pair = if init_ori == Orientation::Forward {
            (&other.c2d, &other.c2d_seam)
        } else {
            (&other.c2d_seam, &other.c2d)
        };
        builder.update_edge_pcurves(edge, pair, s, l, tol, None)
    }
}
