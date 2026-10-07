//! HLRTopoBRep_DSFiller.cxx: InsertFace, the edges of the contour lines of a face
//! and the union of their coinciding vertices.
use super::{DSFiller, arc_edge};
use crate::contap::{ContapContour, ContapIType, ContapLine};
use crate::hlr_topo_brep::Data;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_proj_lib::curve2d_range_tolerance;

/// The 3D curve, the p-curve and the range of one contour edge; `None` for a
/// walking line with too few points.
pub(super) type ContourCurves = Option<(Handle<dyn GeomCurve>, Handle<dyn Geom2dCurve>, f64, f64)>;

impl DSFiller {
    /// InsertFace(FI, F, FO, DS, withPCurve) (private): the outlines on restrictions
    /// and the internal outlines of the contour of `f`, then the union of the
    /// coinciding vertices of the internal outlines.
    pub(super) fn insert_face(
        _fi: i32,
        f: &Face,
        fo: &ContapContour,
        ds: &mut Data,
        with_p_curve: bool,
    ) -> Result<(), Failure> {
        let tol = BRepTool::face_tolerance(f)?;
        ds.add_int_l(f);
        ds.add_out_l(f);
        let mut ends = (Vertex::default(), Vertex::default());
        for cur_line in 1..=fo.nb_lines()? {
            let line = fo.line(cur_line)?;
            if line.type_contour() == ContapIType::Restriction {
                Self::restriction_line(line, f, tol, &mut ends, ds)?;
            } else {
                Self::contour_line(line, f, tol, &mut ends, ds, with_p_curve)?;
            }
        }
        Self::unite_internal_vertices(f, tol, ds)
    }

    // OutLine on restriction: the edge is an outline; its internal points that
    // match both of its end vertices are inserted on it.
    fn restriction_line(
        line: &ContapLine,
        f: &Face,
        tol: f64,
        (vf, vl): &mut (Vertex, Vertex),
        ds: &mut Data,
    ) -> Result<(), Failure> {
        let e = arc_edge(&line.arc()?)?;
        ds.add_out_l(f).append(Shape::from(e.clone()));
        (*vf, *vl) = topexp::vertices(&e, false);
        // insert the Internal points.
        for cur_point in 1..=line.nb_vertex() {
            let p = line.vertex(cur_point)?;
            if !p.is_internal() {
                continue;
            }
            let on_first = p
                .value()
                .is_equal(&BRepTool::pnt(vf)?, BRepTool::vertex_tolerance(vf)?);
            if on_first
                && p.value()
                    .is_equal(&BRepTool::pnt(vl)?, BRepTool::vertex_tolerance(vl)?)
            {
                Self::insert_vertex(&p, tol, &e, ds)?;
            }
        }
        Ok(())
    }

    // An internal outline: an edge between each pair of consecutive vertices of
    // the line whose parameters differ by more than PConfusion.
    fn contour_line(
        line: &ContapLine,
        f: &Face,
        tol: f64,
        (vf, vl): &mut (Vertex, Vertex),
        ds: &mut Data,
        with_p_curve: bool,
    ) -> Result<(), Failure> {
        let nb_points = line.nb_vertex();
        for cur_point in 1..=nb_points {
            let pf = line.vertex(cur_point)?;
            *vf = if pf.is_internal() && cur_point != 1 {
                vl.clone()
            } else {
                Self::make_vertex(&pf, tol, ds)?
            };
            let par_f = pf.parameter_on_line();
            if cur_point >= nb_points {
                continue;
            }
            let pl = line.vertex(cur_point + 1)?;
            *vl = Self::make_vertex(&pl, tol, ds)?;
            let par_l = pl.parameter_on_line();
            if par_l - par_f <= precision::p_confusion() {
                continue;
            }
            if let Some(curves) = Self::contour_curves(line, f, (par_f, par_l), with_p_curve)? {
                let e = contour_edge(curves, f, tol, vf, vl)?;
                if !e.is_null() {
                    ds.add_int_l(f).append(Shape::from(e));
                }
            }
        }
        Ok(())
    }

    // The curves of the part [parF, parL] of a line, by its type.
    fn contour_curves(
        line: &ContapLine,
        f: &Face,
        (first, last): (f64, f64),
        with_p_curve: bool,
    ) -> Result<ContourCurves, Failure> {
        match line.type_contour() {
            ContapIType::Lin => {
                let c: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(line.line()?)).cast();
                let c2d = if with_p_curve {
                    let s = BRepTool::surface(f)?;
                    curve2d_range_tolerance(&c, (first, last), &s, 1e-7)?.0
                } else {
                    Handle::default()
                };
                Ok(Some((c, c2d, first, last)))
            }
            ContapIType::Circle => {
                let c: Handle<dyn GeomCurve> =
                    Handle::new(GeomCircle::from_circ(line.circle()?)).cast();
                let c2d = if with_p_curve {
                    curve2d_range_tolerance(&c, (first, last), &located_surface(f)?, 1e-7)?.0
                } else {
                    Handle::default()
                };
                Ok(Some((c, c2d, first, last)))
            }
            ContapIType::Walking => Self::walking_curves(line, (first, last), with_p_curve),
            ContapIType::Restriction => Err(Failure::with_type(
                "Standard_ProgramError",
                "HLRTopoBRep_DSFiller::InsertFace : Restriction",
                "",
            )),
        }
    }

    // Correction of internal outlines: unite coinciding vertices. Each pair of
    // outlines (the second from the first one on) exchanges the vertices of the
    // second that lie within `tol` of a vertex of the first.
    fn unite_internal_vertices(f: &Face, tol: f64, ds: &mut Data) -> Result<(), Failure> {
        let sq_tol = tol * tol;
        let int_l: Vec<Shape> = ds.add_int_l(f).iter().cloned().collect();
        for (i1, first) in int_l.iter().enumerate() {
            let mut an_int_line = topods::edge(first)?;
            an_int_line.set_orientation(Orientation::Forward);
            let (a0, a1) = topexp::vertices(&an_int_line, false);
            let mut a_ver = [a0, a1];
            for second in &int_l[i1..] {
                let mut an_int_line2 = topods::edge(second)?;
                an_int_line2.set_orientation(Orientation::Forward);
                if an_int_line2.is_same(&an_int_line) {
                    continue;
                }
                unite_pair(&mut a_ver, &mut an_int_line2, sq_tol)?;
            }
        }
        Ok(())
    }
}

// The surface of the face, transformed by its location when it is not the identity.
fn located_surface(f: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (s, loc) = BRepTool::surface_with_location(f)?;
    if loc.is_identity() {
        return Ok(s);
    }
    let surface = s
        .get()
        .ok_or_else(|| super::null_object("the face surface"))?;
    Ok(surface
        .transformed(loc.transformation())?
        .cast::<dyn GeomSurface>())
}

// The edge of a contour part between `vf` (made FORWARD) and `vl` (made REVERSED),
// with its p-curve on the face when there is one.
fn contour_edge(
    (c, c2d, first, last): (Handle<dyn GeomCurve>, Handle<dyn Geom2dCurve>, f64, f64),
    f: &Face,
    tol: f64,
    vf: &mut Vertex,
    vl: &mut Vertex,
) -> Result<Edge, Failure> {
    let b = BRepBuilder::default();
    let mut e = Edge::default();
    b.make_edge_with_curve(&mut e, &c, tol)?;
    vf.set_orientation(Orientation::Forward);
    vl.set_orientation(Orientation::Reversed);
    b.add(&mut e, &*vf)?;
    b.add(&mut e, &*vl)?;
    b.range(&e, first, last, false)?;
    if !c2d.is_null() {
        b.update_edge_pcurve_on_face(&e, &c2d, f, BRepTool::face_tolerance(f)?)?;
    }
    Ok(e)
}

// The two vertices of the first outline against those of the second: a vertex of
// the second within `sq_tol` of one of the first is replaced by it (with the
// orientation of its slot). The second vertex of a closed first outline is skipped.
fn unite_pair(
    a_ver: &mut [Vertex; 2],
    an_int_line2: &mut Edge,
    sq_tol: f64,
) -> Result<(), Failure> {
    let (b0, b1) = topexp::vertices(an_int_line2, false);
    let a_ver2 = [b0, b1];
    for i in 0..2 {
        if i == 1 && a_ver[0].is_same(&a_ver[1]) {
            continue;
        }
        let pnt1 = BRepTool::pnt(&a_ver[i])?;
        for (j, v2) in a_ver2.iter().enumerate() {
            if a_ver[i].is_same(v2) {
                continue;
            }
            if pnt1.square_distance(&BRepTool::pnt(v2)?) > sq_tol {
                continue;
            }
            let a_bb = BRepBuilder::default();
            a_bb.remove(an_int_line2, v2)?;
            a_ver[i].set_orientation(if j == 0 {
                Orientation::Forward
            } else {
                Orientation::Reversed
            });
            a_bb.add(an_int_line2, &a_ver[i])?;
        }
    }
    Ok(())
}

mod walking;
