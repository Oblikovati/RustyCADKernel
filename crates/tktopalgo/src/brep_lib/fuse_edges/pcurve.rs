// Rust translation of BRepLib_FuseEdges::UpdatePCurve (BRepLib_FuseEdges.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curves on surface of a fused edge, from those of the first edge of its chain
//! (concatenated with those of the other edges when the first one is too short).
use super::BRepLibFuseEdges;
use crate::brep_lib::same_parameter::same_parameter_edge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool, EdgeCurveOnSurface};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom2d_convert::CompCurveToBSplineCurve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;

/// The basis of a trimmed p-curve, else the p-curve.
fn untrimmed(curve: Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    match curve.cast::<Geom2dTrimmedCurve>().get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => curve,
    }
}

impl BRepLibFuseEdges {
    /// `UpdatePCurve`: each curve on surface of the old edge whose surface is that of one
    /// of its faces is set on the new edge (both curves of a seam, or one extended by
    /// the others of the chain when shorter than the new range), with the edge range
    /// adapted to it; false when the p-curves of the chain do not join. A rebuilt
    /// p-curve makes the new edge go through SameParameter.
    pub(super) fn update_pcurve(
        &self,
        old_edge: &Edge,
        new_edge: &mut Edge,
        chain: &List<Shape>,
    ) -> Result<bool, Failure> {
        let faces = self.my_map_edg_lst_fac.find_from_key(old_edge)?;
        let mut rebuilt: bool = false;
        let mut index: i32 = 1;
        let mut found = BRepTool::curve_on_surface_by_index(old_edge, index)?;
        while found.pcurve.is_some() {
            if let Some(face) = face_on(faces, &found.surface)?
                && !update_one(old_edge, new_edge, &face, index, chain, &mut rebuilt)?
            {
                return Ok(false);
            }
            // get next pcurve
            index += 1;
            found = BRepTool::curve_on_surface_by_index(old_edge, index)?;
        }
        if rebuilt {
            // force same parameter
            BRepBuilder::default().same_parameter(new_edge, false)?;
            same_parameter_edge(new_edge, BRepTool::edge_tolerance(new_edge)?)?;
        }
        Ok(true)
    }
}

/// The first face of the list on the surface (the same handle), if any.
fn face_on(
    faces: &List<Shape>,
    surface: &Handle<dyn GeomSurface>,
) -> Result<Option<Face>, Failure> {
    for face in faces.iter() {
        let (s, _) = BRepTool::surface_with_location(&Face::unchecked(face.clone()))?;
        if s == *surface {
            return Ok(Some(Face::unchecked(face.clone())));
        }
    }
    Ok(None)
}

/// One curve on surface of `UpdatePCurve` (read again on the FORWARD old edge): both
/// curves of a seam, or the p-curve (extended by the chain when shorter than the new
/// range); then, unless a p-curve was rebuilt, the new edge range moved onto the old
/// p-curve range. False when the chain p-curves do not join.
fn update_one(
    old_edge: &Edge,
    new_edge: &Edge,
    face: &Face,
    index: i32,
    chain: &List<Shape>,
    rebuilt: &mut bool,
) -> Result<bool, Failure> {
    let builder = BRepBuilder::default();
    let (ef, el) = BRepTool::range(new_edge)?;
    let forward = Edge::unchecked(old_edge.oriented(Orientation::Forward));
    let mut found: EdgeCurveOnSurface = BRepTool::curve_on_surface_by_index(&forward, index)?;
    let tol: f64 = BRepTool::edge_tolerance(new_edge)?;
    if BRepTool::is_closed_on_surface(old_edge, &found.surface, &found.location)? {
        let reversed = Edge::unchecked(forward.reversed());
        let face_forward = Face::unchecked(face.oriented(Orientation::Forward));
        let seam = BRepTool::curve_on_face(&reversed, &face_forward)?;
        (found.first, found.last) = (seam.first, seam.last);
        let (c1, c2) = (untrimmed(found.pcurve.clone()), untrimmed(seam.pcurve));
        builder.update_edge_pcurves(
            new_edge,
            (&c1, &c2),
            &found.surface,
            &found.location,
            tol,
            None,
        )?;
    } else {
        let mut curv2d = untrimmed(found.pcurve.clone());
        let c = curv2d.get().expect("a p-curve");
        let length: f64 = c.last_parameter() - c.first_parameter();
        if length + 2. * epsilon(length) < el - ef {
            let Some(joined) = joined_pcurve(&curv2d, &found, chain, (ef, el))? else {
                // cannot merge pcurves
                return Ok(false);
            };
            curv2d = joined;
            *rebuilt = true;
        }
        builder.update_edge_pcurve(
            new_edge,
            &curv2d,
            &found.surface,
            &found.location,
            tol,
            None,
        )?;
    }
    // the old pcurve range is cf,cl. The new 3d edge range is ef,el. if we want
    // the pcurve to be samerange we must adapt the parameter of the edge. In general
    // cases cf=ef and cl=el expect for periodic curve if the new edge is going over
    // the value 0.
    if !*rebuilt {
        let (cf, cl) = (found.first, found.last);
        let (first, last) = if old_edge.orientation() == Orientation::Reversed {
            (cl - el + ef, cl)
        } else {
            (cf, cf + el - ef)
        };
        builder.range(new_edge, first, last, false)?;
    }
    Ok(true)
}

/// A p-curve made bounded (a trimmed curve on the range when it is not).
fn bounded(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<dyn Geom2dBoundedCurve>, Failure> {
    let as_bounded = curve.cast::<dyn Geom2dBoundedCurve>();
    if as_bounded.is_some() {
        return Ok(as_bounded);
    }
    Ok(Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast())
}

/// The p-curve concatenated with the p-curves of the other chain edges on the same
/// surface, reparametrized to the new range when it differs; None when one does not
/// join.
fn joined_pcurve(
    curv2d: &Handle<dyn Geom2dCurve>,
    found: &EdgeCurveOnSurface,
    chain: &List<Shape>,
    (ef, el): (f64, f64),
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    let first = bounded(curv2d, found.first, found.last)?;
    let mut concat =
        CompCurveToBSplineCurve::with_curve(&first, ParameterisationType::TgtThetaOver2)?;
    for shape in chain.iter().skip(1) {
        let e = BRepTool::curve_on_surface(
            &Edge::unchecked(shape.clone()),
            &found.surface,
            &found.location,
        )?;
        let bc = bounded(&e.pcurve, e.first, e.last)?;
        if !concat.add(&bc, precision::p_confusion(), false)? {
            return Ok(None);
        }
    }
    let joined: Handle<Geom2dBSplineCurve> = concat.bspline_curve();
    let bs = joined.get().expect("a B-spline");
    // check that new curve 2d is same range
    if (bs.first_parameter() - ef).abs() > precision::p_confusion()
        || (bs.last_parameter() - el).abs() > precision::p_confusion()
    {
        let mut knots = bs.knots();
        bsplclib::reparametrize(ef, el, &mut knots)?;
        bs.set_knots(&knots)?;
    }
    Ok(Some(joined.cast()))
}
