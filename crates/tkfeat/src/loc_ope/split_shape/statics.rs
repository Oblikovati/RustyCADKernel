// Rust translation of the static functions of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_SplitShape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The classification of wires in faces and the choice of the next edge or section
//! wire at a vertex, by the 2D directions of the p-curves.
use crate::errors::null_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve as ProjectPointOnCurve2d;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
use std::f64::consts::PI;

/// The p-curve of `e` on `f`, which must exist, and its range.
fn pcurve(
    e: &Edge,
    f: &Face,
    member: &str,
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let pc = BRepTool::curve_on_face(e, f)?;
    if pc.pcurve.is_null() {
        return Err(null_object(member, "edge without p-curve on the face"));
    }
    Ok((pc.pcurve, pc.first, pc.last))
}

fn curve_ref<'a>(
    c: &'a Handle<dyn Geom2dCurve>,
    member: &str,
) -> Result<&'a dyn Geom2dCurve, Failure> {
    c.get().ok_or_else(|| null_object(member, "null p-curve"))
}

/// `SameUV(P1, P2, theBAS)`: the points are less than half a period apart along each
/// periodic direction.
pub(super) fn same_uv(
    p1: Point2,
    p2: Point2,
    the_bas: &BRepAdaptorSurface,
) -> Result<bool, Failure> {
    let mut is_same = true;
    if the_bas.is_u_periodic()? {
        is_same = (p1.x() - p2.x()).abs() < the_bas.u_period()? * 0.5;
    }
    if the_bas.is_v_periodic()? {
        is_same = is_same && (p1.y() - p2.y()).abs() < the_bas.v_period()? * 0.5;
    }
    Ok(is_same)
}

/// `IsInside(F, W1, W2)`: whether the first vertex of the first edge of `w1` lies in the
/// region bounded by `w2` on the surface of `f` (outside it when `w2` bounds a hole);
/// false when that edge has no p-curve on `f`.
pub(super) fn is_inside_wire(f: &Face, w1: &Wire, w2: &Wire) -> Result<bool, Failure> {
    // Attention, c`est tres boeuf !!!!
    let b = BRepBuilder::default();
    let mut new_face = topods::face(&f.empty_copied()?)?;
    new_face.set_orientation(Orientation::Forward);
    b.add(&mut new_face, w2)?;
    let classif = BRepTopAdaptorFClass2d::new(&new_face, precision::p_confusion())?;
    // le wire donne defini un trou
    let reversed = classif.perform_infinite_point()? == State::In;
    let edg = topods::edge(Explorer::new(w1, ShapeEnum::Edge, ShapeEnum::Shape)?.current())?;
    let vtx = topods::vertex(Explorer::new(&edg, ShapeEnum::Vertex, ShapeEnum::Shape)?.current())?;
    let prm = BRepTool::parameter(&vtx, &edg)?;
    let pc = BRepTool::curve_on_face(&edg, f)?;
    let Some(c2d) = pc.pcurve.get() else {
        return Ok(false);
    };
    let state = classif.perform(c2d.value(prm)?, true)?;
    Ok(if reversed {
        state == State::Out
    } else {
        state == State::In
    })
}

/// The parameter at which an edge is classified: the middle of a finite range, else 0,
/// or one unit inside the finite bound.
fn sample_parameter(f: f64, l: f64) -> f64 {
    let (neg, pos) = (
        precision::is_negative_infinite(f),
        precision::is_positive_infinite(l),
    );
    match (neg, pos) {
        (false, false) => (f + l) / 2.,
        (true, true) => 0.,
        (true, false) => l - 1.,
        (false, true) => f + 1.,
    }
}

/// Whether more of 10 points of the p-curve of an edge ON the face boundary are OUT
/// than IN or ON.
fn mostly_out(
    classif: &BRepTopAdaptorFClass2d,
    c2d: &dyn Geom2dCurve,
    f: f64,
    l: f64,
) -> Result<bool, Failure> {
    let nb_pnt = 10;
    let (mut nb_out, mut nb_in, mut nb_on) = (0, 0, 0);
    for j in 1..=nb_pnt {
        // check neighbouring point
        let prm = f + (l - f) / f64::from(nb_pnt) * f64::from(j - 1);
        match classif.perform(c2d.value(prm)?, true)? {
            State::Out => nb_out += 1,
            State::In => nb_in += 1,
            _ => nb_on += 1,
        }
    }
    Ok(nb_out > nb_in + nb_on)
}

/// `IsInside(F, W)`: no edge of `w` has its middle OUT of `f` (an edge ON the boundary
/// counts as out when most of 10 of its points are).
pub(super) fn is_inside(f: &Face, w: &Wire) -> Result<bool, Failure> {
    // Attention, c`est tres boeuf !!!!
    for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edg = topods::edge(&e)?;
        let pc = BRepTool::curve_on_face(&edg, f)?;
        let (fp, lp) = (pc.first, pc.last);
        let c2d = curve_ref(&pc.pcurve, "LocOpe_SplitShape::IsInside")?;
        let classif = BRepTopAdaptorFClass2d::new(f, precision::p_confusion())?;
        let stat = classif.perform(c2d.value(sample_parameter(fp, lp))?, true)?;
        if stat == State::Out {
            return Ok(false);
        }
        if stat == State::On && mostly_out(&classif, c2d, fp, lp)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `GetDirection(theEdge, theFace, thePnt, theDir, isFirstEnd)`: the end point of the
/// p-curve of the oriented edge (its start when `is_first_end`) and the direction of
/// the edge there, by a chord of 1% of the range.
pub(super) fn get_direction(
    the_edge: &Edge,
    the_face: &Face,
    is_first_end: bool,
) -> Result<(Point2, Vector2), Failure> {
    let (c, a_first, a_last) = pcurve(the_edge, the_face, "LocOpe_SplitShape::GetDirection")?;
    let c2d = curve_ref(&c, "LocOpe_SplitShape::GetDirection")?;
    let an_or = the_edge.orientation();
    let take_first = (an_or == Orientation::Forward && is_first_end)
        || (an_or == Orientation::Reversed && !is_first_end);
    let dpar = (a_last - a_first) * 0.01;
    let (pnt, mut dir) = if take_first {
        let pnt = c2d.d0(a_first)?;
        (pnt, Vector2::between(pnt, c2d.value(a_first + dpar)?))
    } else {
        let pnt = c2d.d0(a_last)?;
        (pnt, Vector2::between(c2d.value(a_last - dpar)?, pnt))
    };
    if an_or == Orientation::Reversed {
        dir.reverse();
    }
    Ok((pnt, dir))
}

/// `ChoixUV(Last, F, Poss, theResEdge, plst, dlst)`: among the possible edges starting
/// at `plst`, the one turning most to the left of `dlst` (`last` itself least);
/// `plst` and `dlst` move to its end. None when no edge starts there.
pub(super) fn choix_uv(
    last: &Edge,
    f: &Face,
    poss: &IndexedMapOfShape,
    plst: &mut Point2,
    dlst: &mut Vector2,
) -> Result<Option<Edge>, Failure> {
    let surf = BRepAdaptorSurface::with_face(f, false)?; // no restriction
    surf.d0(plst.x(), plst.y())?;
    let ref2d = Direction2::from_vector(*dlst)?;
    let (mut imin, mut angmax) = (0, -PI);
    for index in 1..=poss.extent() {
        let an_edge = topods::edge(poss.find_key(index)?)?;
        let (p2d, v2d) = get_direction(&an_edge, f, true)?;
        if !same_uv(*plst, p2d, &surf)? {
            continue;
        }
        surf.d0(p2d.x(), p2d.y())?;
        let ang = if !last.is_same(&an_edge) {
            ref2d.angle(Direction2::from_vector(v2d)?)
        } else {
            -PI
        };
        if ang > angmax {
            imin = index;
            angmax = ang;
        }
    }
    if imin == 0 {
        return Ok(None);
    }
    let the_res_edge = topods::edge(poss.find_key(imin)?)?;
    (*plst, *dlst) = get_direction(&the_res_edge, f, false)?;
    Ok(Some(the_res_edge))
}

/// The first edge of `s` with an end at `v`, and the orientation `at_first` when it is
/// its first vertex (`at_last` when its last); the last edge explored and `default`
/// when none.
fn edge_at(
    s: &Shape,
    v: &Vertex,
    at_first: Orientation,
    at_last: Orientation,
    default: Orientation,
) -> Result<(Edge, Orientation), Failure> {
    let mut found = Edge::default();
    for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        found = topods::edge(&e)?;
        let (v1, v2) = topexp::vertices(&found, false);
        if v1.is_same(v) {
            return Ok((found, at_first));
        }
        if v2.is_same(v) {
            return Ok((found, at_last));
        }
    }
    Ok((found, default))
}

/// `ChooseDirection(RefDir, RefVertex, theFace, Ldirs)`: among the shapes of `ldirs`
/// leaving `ref_vertex`, the one making the smallest angle (in `[0, 2 pi)`) from the
/// incoming direction of `ref_dir`.
pub(super) fn choose_direction(
    ref_dir: &Shape,
    ref_vertex: &Vertex,
    the_face: &Face,
    ldirs: &List<Shape>,
) -> Result<Shape, Failure> {
    let member = "LocOpe_SplitShape::ChooseDirection";
    let (ref_edge, mut an_or) = edge_at(
        ref_dir,
        ref_vertex,
        Orientation::Reversed,
        Orientation::Forward,
        Orientation::Forward,
    )?;
    let (ref_curve, ref_first, ref_last) = pcurve(&ref_edge, the_face, member)?;
    let ref_par = if an_or == Orientation::Forward {
        ref_last
    } else {
        ref_first
    };
    let (_, mut ref_vec) = curve_ref(&ref_curve, member)?.d1(ref_par)?;
    if an_or == Orientation::Forward {
        ref_vec.reverse();
    }
    let mut min_angle = f64::MAX;
    let mut target_dir = Shape::default();
    for a_shape in ldirs.iter() {
        let (an_edge, or) = edge_at(
            a_shape,
            ref_vertex,
            Orientation::Forward,
            Orientation::Reversed,
            an_or,
        )?;
        an_or = or;
        let (a_curve, a_first, a_last) = pcurve(&an_edge, the_face, member)?;
        let a_par = if an_or == Orientation::Forward {
            a_first
        } else {
            a_last
        };
        let (_, mut a_vec) = curve_ref(&a_curve, member)?.d1(a_par)?;
        if an_or == Orientation::Reversed {
            a_vec.reverse();
        }
        let mut an_angle = a_vec.angle(ref_vec)?;
        if an_angle < 0. {
            an_angle += 2. * PI;
        }
        if an_angle < min_angle {
            min_angle = an_angle;
            target_dir = a_shape.clone();
        }
    }
    Ok(target_dir)
}

/// `checkOverlapping(theEdge1, theEdge2, theFace)`: whether three inner points of the
/// p-curve of the second edge lie on the first (within the 2D resolution of the summed
/// tolerances) with parallel tangents (1 degree).
pub(super) fn check_overlapping(
    the_edge1: &Edge,
    the_edge2: &Edge,
    the_face: &Face,
) -> Result<bool, Failure> {
    let member = "LocOpe_SplitShape::checkOverlapping";
    let an_ad_s = BRepAdaptorSurface::with_face(the_face, false)?;
    let max_tol = BRepTool::edge_tolerance(the_edge1)? + BRepTool::edge_tolerance(the_edge2)?;
    let max_tol2d = an_ad_s
        .u_resolution(max_tol)?
        .max(an_ad_s.v_resolution(max_tol)?);
    let tol_ang = PI / 180.;
    let pc1 = BRepTool::curve_on_face(the_edge1, the_face)?;
    let pc2 = BRepTool::curve_on_face(the_edge2, the_face)?;
    let crv2 = curve_ref(&pc2.pcurve, member)?;
    let nb_p = 4;
    let a_step = (pc2.last - pc2.first) / f64::from(nb_p);
    let mut proj = ProjectPointOnCurve2d::new();
    for j in 1..nb_p {
        let par2 = pc2.first + a_step * f64::from(j);
        let (p2d, v2) = crv2.d1(par2)?;
        proj.init_range(p2d, &pc1.pcurve, pc1.first, pc1.last)?;
        // check intermediate points
        if proj.nb_points() == 0 || proj.lower_distance()? > max_tol2d {
            return Ok(false);
        }
        let par1 = proj.lower_distance_parameter()?;
        let (_, v1) = curve_ref(&pc1.pcurve, member)?.d1(par1)?;
        if !v1.is_parallel(v2, tol_ang)? {
            return Ok(false);
        }
    }
    Ok(true)
}
