// Rust translation of the file statics getCurveOnSurface, isTangentToArc, BonVoisin,
// Projection, IsInput, IsG1, SearchFaceOnV and ChFi3d_BuildPlane
// (ChFi3d_Builder_2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The search of the faces a fillet goes on to at the end of its contour.
use crate::chfi_ds::{CommonPoint, Map, SurfDataHandle, lock};
use crate::chfi3d::{inter_plane_edge, is_tangent_faces};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_lprop::BRepLPropSLProps;
use rustycad_tkbrep::brep_tools::{WireExplorer, is_really_closed};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::extrema::{ExtPC, LocateExtPC};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3, Transform3, Vector3, Xyz};
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// `getCurveOnSurface(theEdge, theFace)`: the pcurve of the edge on the face (null for a
/// null edge or face).
pub(super) fn curve_on_surface(
    edge: &Edge,
    face: &Face,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if edge.is_null() || face.is_null() {
        return Ok(Handle::default());
    }
    Ok(BRepTool::curve_on_face(edge, face)?.pcurve)
}

/// The value of a pcurve (Standard_NullObject for a null one).
pub(super) fn pcurve_value(pc: &Handle<dyn Geom2dCurve>, w: f64) -> Result<Point2, Failure> {
    pc.get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "ChFi3d_Builder: null pcurve"))?
        .value(w)
}

/// `isTangentToArc(aCommonPoint, anAngularTolerance)`: the vector of the common point is
/// parallel to its arc.
pub(super) fn is_tangent_to_arc(cp: &CommonPoint, angular_tolerance: f64) -> Result<bool, Failure> {
    if !cp.has_vector() {
        return Ok(false);
    }
    let (arc_curve, _, _) = BRepTool::curve(cp.arc()?)?;
    let (_, arc_tangent) = arc_curve
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "isTangentToArc"))?
        .d1(cp.parameter_on_arc()?)?;
    cp.vector()?.is_parallel(arc_tangent, angular_tolerance)
}

/// The pcurve of `newe` on the face `ff` when the edge is the cured edge's neighbour,
/// with the seam on a face closed by it.
fn neighbour_pcurve(
    newe: &Edge,
    ff: &Face,
    isclosed: bool,
    isreallyclosed: bool,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if !(isclosed && !isreallyclosed) {
        return curve_on_surface(newe, ff);
    }
    let mut fff = ff.clone();
    fff.set_orientation(Orientation::Forward);
    let mut ex2 = Explorer::new(fff.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while ex2.more() {
        if newe.is_same(ex2.current()) {
            let e = topods::edge(ex2.current())?;
            return curve_on_surface(&e, &fff);
        }
        ex2.next()?;
    }
    Ok(Handle::default())
}

/// The parameters on the neighbour face reached at `winter` on the edge `ecur` from the
/// face `f` (`issame`: the neighbour is `f` across its seam).
#[allow(clippy::too_many_arguments)]
fn bon_voisin_point(
    point: Point3,
    hs: &BRepAdaptorSurface,
    hc: &BRepAdaptorCurve,
    ff: &Face,
    ecur: &Edge,
    winter: f64,
    issame: bool,
    (isclosed, isreallyclosed): (bool, bool),
) -> Result<(f64, f64), Failure> {
    let mut newe = ecur.clone();
    newe.set_orientation(Orientation::Forward);
    hs.initialize(ff, true)?;
    let pc = neighbour_pcurve(&newe, ff, isclosed, isreallyclosed)?;
    let mut xy = pcurve_value(&pc, winter)?;
    if issame {
        let d1 = hs.d1(xy.x(), xy.y())?;
        let nors = d1.d1u.crossed(d1.d1v);
        let (cpt, cd) = hc.d1(winter)?;
        let vref = Vector3::between(point, cpt);
        let mut fff = ff.clone();
        fff.set_orientation(Orientation::Forward);
        if vref.dot(nors.crossed(cd)) < 0. {
            newe.set_orientation(Orientation::Reversed);
        }
        xy = pcurve_value(&curve_on_surface(&newe, &fff)?, winter)?;
    }
    Ok((xy.x(), xy.y()))
}

/// `BonVoisin(Point, HS, F, plane, cured, XDep, YDep, EFMap, tol3d)`: the face tangent to
/// `f` across the edge nearest to the start point cut by the plane, `hs` set on it and
/// the start parameters moved there; false when there is none.
#[allow(clippy::too_many_arguments)]
pub(super) fn bon_voisin(
    point: Point3,
    hs: &BRepAdaptorSurface,
    f: &Face,
    plane: &Handle<dyn Adaptor3dSurface>,
    cured: &Edge,
    dep: (&mut f64, &mut f64),
    ef_map: &Map,
    tol3d: f64,
) -> Result<bool, Failure> {
    let papp = hs.value(*dep.0, *dep.1)?;
    let mut dist = f64::MAX;
    let mut found = false;
    let mut ex = Explorer::new(f.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while ex.more() {
        let ecur = topods::edge(ex.current())?;
        ex.next()?;
        if ecur.is_same(cured) {
            continue;
        }
        let hc = Handle::new(BRepAdaptorCurve::with_edge(&ecur)?);
        let tolc = hc.resolution(tol3d)?;
        let hca: Handle<dyn Adaptor3dCurve> = hc.clone().cast();
        let Some(winter) = inter_plane_edge(plane, &hca, true, tolc)? else {
            continue;
        };
        let ndist = hc.value(winter)?.square_distance(&papp);
        if ndist >= dist {
            continue;
        }
        let isclosed = BRepTool::is_closed_on_face(&ecur, f)?;
        let isreallyclosed = is_really_closed(&ecur, f)?;
        for s in ef_map.find_from_key(ecur.as_ref())?.iter() {
            let ff = topods::face(s)?;
            let issame = ff.is_same(f);
            let istg = is_tangent_faces(&ecur, &ff, f, Continuity::G1)?;
            if (!issame || isreallyclosed) && istg {
                found = true;
                dist = ndist;
                let (x, y) = bon_voisin_point(
                    point,
                    hs,
                    &hc,
                    &ff,
                    &ecur,
                    winter,
                    issame,
                    (isclosed, isreallyclosed),
                )?;
                *dep.0 = x;
                *dep.1 = y;
                break;
            }
        }
    }
    Ok(found)
}

/// `Projection(PExt, P, C, W, Tol)`: the parameter of the curve nearest to `p`, refined
/// from `w` locally then globally; false when nothing nearer than `w` is found.
pub(super) fn projection(
    pext: &mut ExtPC<'_>,
    p: Point3,
    c: &dyn Adaptor3dCurve,
    w: &mut f64,
    tol: f64,
) -> Result<bool, Failure> {
    let mut dist2 = c.value(*w)?.square_distance(&p);
    if dist2 < tol * tol {
        return Ok(true);
    }
    let mut ok = false;
    let ext = LocateExtPC::with_curve(p, c, *w, tol / 10.)?;
    if ext.is_done() {
        let par = ext.point()?.parameter();
        let daux2 = c.value(par)?.square_distance(&p);
        if daux2 < dist2 {
            *w = par;
            dist2 = daux2;
            ok = true;
            if dist2 < tol * tol {
                return Ok(true);
            }
        }
    }
    pext.perform(p)?;
    if pext.is_done() {
        for ii in 1..=pext.nb_ext()? {
            if pext.square_distance(ii)? < dist2 {
                dist2 = pext.square_distance(ii)?;
                *w = pext.point(ii)?.parameter();
                ok = true;
            }
        }
    }
    Ok(ok)
}

/// The tangents of the two edges of a face at the vertex `ve`, outgoing (at most two).
fn tangents_at(ve: &Vertex, fa: &Face) -> Result<(usize, [Vector3; 2], Point3), Failure> {
    let mut vec3d = [Vector3::default(); 2];
    let mut point = Point3::default();
    let mut trouve = 0;
    let mut face_exp = Explorer::new(fa.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)?;
    while face_exp.more() && trouve < 2 {
        let w = topods::wire(face_exp.current())?;
        let mut wire_exp = WireExplorer::new();
        wire_exp.init(&w)?;
        trouve = 0;
        while wire_exp.more() && trouve < 2 {
            let e = wire_exp.current().clone();
            let (vf, vl) = topexp::vertices(&e, false);
            if vf.is_same(ve) || vl.is_same(ve) {
                let (p, mut v) =
                    BRepAdaptorCurve::with_edge(&e)?.d1(BRepTool::parameter(ve, &e)?)?;
                if !vf.is_same(ve) {
                    v.reverse();
                }
                point = p;
                vec3d[trouve] = v;
                trouve += 1;
            }
            wire_exp.next()?;
        }
        face_exp.next()?;
    }
    Ok((trouve, vec3d, point))
}

/// `IsInput(Vec, Ve, Fa)`: the direction `vec` enters the face at its vertex `ve`
/// (between the tangents of its two edges there).
pub(super) fn is_input(vec: Vector3, ve: &Vertex, fa: &Face) -> Result<bool, Failure> {
    let (trouve, vec3d, point) = tangents_at(ve, fa)?;
    if trouve < 2 {
        return Ok(false);
    }
    let normal = vec3d[0].crossed(vec3d[1]);
    if normal.square_magnitude() < precision::confusion() {
        return vec.is_parallel(vec3d[0], precision::confusion());
    }
    let mut amax = vec3d[1].angle_with_ref(vec3d[0], normal)?;
    let amin = if amax < 0. {
        let a = amax;
        amax = 0.;
        a
    } else {
        0.
    };
    let axe = Axis3::new(
        point,
        Direction3::from_vector(normal)?,
        Direction3::from_vector(vec3d[0])?,
    )?;
    let mut transf = Transform3::default();
    transf.set_transformation_to(axe);
    let mut coord: Xyz = *vec.xyz();
    transf.transforms(&mut coord);
    coord.set_z(0.);
    transf.invert()?;
    transf.transforms(&mut coord);
    let the_proj = Vector3::from(coord);
    let angle = the_proj.angle_with_ref(vec3d[0], normal)?;
    Ok(angle >= amin && angle <= amax)
}

/// `IsG1(TheMap, E, FRef, FVoi)`: a face tangent to `fref` across `e` (`fref` itself
/// across a seam), set in `fvoi`.
pub(super) fn is_g1(
    the_map: &Map,
    e: &Edge,
    fref: &Face,
    fvoi: &mut Face,
) -> Result<bool, Failure> {
    for s in the_map.find_from_key(e.as_ref())?.iter() {
        let f = topods::face(s)?;
        if !f.is_same(fref) {
            *fvoi = f;
            if is_tangent_faces(e, fref, fvoi, Continuity::G1)? {
                return Ok(true);
            }
        }
    }
    let mut orient: Option<Orientation> = None;
    let mut ex = Explorer::new(fref.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while ex.more() {
        let ed = topods::edge(ex.current())?;
        ex.next()?;
        if !ed.is_same(e) {
            continue;
        }
        match orient {
            None => orient = Some(ed.orientation()),
            Some(o) if ed.orientation() == rustycad_tkg3d::top_abs::reverse(o) => {
                *fvoi = fref.clone();
                return is_tangent_faces(e, fref, fref, Continuity::G1);
            }
            _ => {}
        }
    }
    Ok(false)
}

/// `SearchFaceOnV(Pc, FRef, VEMap, EFMap, F1, F2)`: the faces the vector of the common
/// point enters at its vertex: `fref` itself, or the faces tangent to it at the vertex.
pub(super) fn search_face_on_v(
    pc: &CommonPoint,
    fref: &Face,
    ve_map: &Map,
    ef_map: &Map,
    f1: &mut Face,
    f2: &mut Face,
) -> Result<i32, Failure> {
    let (vec, vtx) = (pc.vector()?, pc.vertex()?.clone());
    let mut find_face = is_input(vec, &vtx, fref)?;
    if find_face {
        find_face = is_input(vec.reversed(), &vtx, fref)?;
    }
    if find_face {
        *f1 = fref.clone();
        return Ok(1);
    }
    let mut num = 0;
    let mut fvoi = Face::default();
    for s in ve_map.find_from_key(vtx.as_ref())?.iter() {
        if num >= 2 {
            break;
        }
        let e = topods::edge(s)?;
        let mut trouve = ef_map
            .find_from_key(e.as_ref())?
            .iter()
            .any(|f| f.is_same(fref.as_ref()));
        if trouve {
            trouve = is_g1(ef_map, &e, fref, &mut fvoi)?;
        }
        if trouve {
            trouve = is_input(vec, &vtx, &fvoi)?;
        }
        if trouve {
            if num == 0 {
                *f1 = fvoi.clone();
            } else {
                *f2 = fvoi.clone();
            }
            num += 1;
        }
    }
    Ok(num)
}

/// `ChFi3d_BuildPlane(DStr, HS, pons, SD, isfirst, ons)`: the tangent plane of the face at
/// the end of the fillet on its arc, `hs` set on it and `pons` at its origin
/// (Standard_Failure when the normal is not defined there).
pub(super) fn build_plane(
    dstr: &TopOpeBRepDSDataStructure,
    hs: &BRepAdaptorSurface,
    pons: &mut Point2,
    sd: &SurfDataHandle,
    isfirst: bool,
    ons: i32,
) -> Result<(), Failure> {
    let g = lock(sd);
    let f = topods::face(dstr.shape(g.index(ons), true))?;
    if f.is_null() {
        return Ok(());
    }
    let cp = g.vertex(isfirst, ons);
    if cp.is_on_arc() {
        let hc = curve_on_surface(cp.arc()?, &f)?;
        let uv = pcurve_value(&hc, cp.parameter_on_arc()?)?;
        let mut prop = BRepLPropSLProps::with_parameters(hs, uv.x(), uv.y(), 1, 1.0e-12)?;
        if prop.is_normal_defined()? {
            let pln: Handle<dyn GeomSurface> =
                Handle::new(GeomPlane::from_point(prop.value(), prop.normal()?)?).cast();
            let mut new_f = BRepLibMakeFace::from_surface(&pln, precision::confusion())?.face()?;
            new_f.set_orientation(f.orientation());
            *pons = Point2::new(0., 0.);
            hs.initialize(&new_f, true)?;
            return Ok(());
        }
    }
    Err(Failure::new("ChFi3d_BuildPlane : echec ."))
}

/// A plane of the guide at `w`, normal to it, as a surface adaptor.
pub(super) fn guide_plane(p: Point3, v: Vector3) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    let pl: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_point(p, Direction3::from_vector(v)?)?).cast();
    Ok(Handle::new(GeomAdaptorSurface::with_surface(&pl)?).cast())
}
