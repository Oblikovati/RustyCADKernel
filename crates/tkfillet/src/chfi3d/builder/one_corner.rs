// Rust translation of ChFi3d_Builder::PerformOneCorner (ChFi3d_Builder_C1.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of one contour at a vertex: its fillet cut by the face at the end (the
//! "cork"), and, OnSame, the prolongation of the edge of the opposite face.
use super::ChFi3dBuilder;
use super::corner_tools::{extend_surface, inters_update_on_same, update_on_edge};
use crate::chfi_ds::{State, StripeHandle, SurfDataHandle, lock};
use crate::chfi3d::{edge_state, index_of_surf_data};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;

/// The state of a corner of one contour.
pub(super) struct OneCorner {
    pub stripe: StripeHandle,
    pub fd: SurfDataHandle,
    pub isfirst: bool,
    pub onsame: bool,
    pub fv: Face,
    pub fad: Face,
    pub fop: Face,
    pub arcprol: Edge,
    pub arcspine: Edge,
    pub oarcprolv: Orientation,
    pub oarcprolop: Orientation,
    pub hbs: Handle<BRepAdaptorSurface>,
    pub hbop: Handle<BRepAdaptorSurface>,
    pub ifad: i32,
    pub ifop: i32,
    pub p2dbout: Point2,
}

/// The cork curve of a corner: the curve, its pcurves on the fillet and on the face, its
/// range and tolerance, and its split at the seam of the face.
#[derive(Default)]
pub(super) struct Cork {
    pub cc: Handle<dyn GeomCurve>,
    pub ps: Handle<dyn Geom2dCurve>,
    pub pc: Handle<dyn Geom2dCurve>,
    pub udeb: f64,
    pub ufin: f64,
    pub tolreached: f64,
    pub edgecouture: Edge,
    pub intcouture: bool,
    pub par1: f64,
    pub par2: f64,
    pub indpt: i32,
    pub curv: [Handle<dyn GeomCurve>; 2],
    pub icurv: [i32; 2],
}

/// A vertex is a vertex of the shape `s`.
pub(super) fn has_vertex(s: &rustycad_tkbrep::topods::Shape, v: &Vertex) -> Result<bool, Failure> {
    let mut ex = Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    while ex.more() {
        if v.is_same(ex.current()) {
            return Ok(true);
        }
        ex.next()?;
    }
    Ok(false)
}

/// The pcurve of an edge on a face (Standard_ConstructionError when it has none).
pub(super) fn pcurve_of(e: &Edge, f: &Face) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let pc = BRepTool::curve_on_face(e, f)?.pcurve;
    if pc.is_null() {
        return Err(Failure::with_message(
            "Standard_ConstructionError",
            "Failed to get p-curve of edge",
        ));
    }
    Ok(pc)
}

/// The value of a pcurve (Standard_NullObject when null).
pub(super) fn value_2d(pc: &Handle<dyn Geom2dCurve>, t: f64) -> Result<Point2, Failure> {
    pc.get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "pcurve"))?
        .value(t)
}

/// The orientation of the edge `e` in the face `f` (forward), if it is there.
pub(super) fn edge_orientation(f: &Face, e: &Edge) -> Result<Option<Orientation>, Failure> {
    let mut ex = Explorer::new(
        &f.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while ex.more() {
        if e.is_same(ex.current()) {
            return Ok(Some(ex.current().orientation()));
        }
        ex.next()?;
    }
    Ok(None)
}

/// The orientation of the edge `e` in the face `f` (forward), `default` when absent.
pub(super) fn orientation_in(
    f: &Face,
    e: &Edge,
    default: Orientation,
) -> Result<Orientation, Failure> {
    Ok(edge_orientation(f, e)?.unwrap_or(default))
}

impl ChFi3dBuilder {
    /// The surface data of the contour at the vertex, the surplus ones without face at the
    /// end removed.
    pub(super) fn corner_surf_data(
        &self,
        stripe: &StripeHandle,
        vtx: &Vertex,
    ) -> Result<(SurfDataHandle, bool), Failure> {
        let (mut num, sens) = index_of_surf_data(vtx, stripe)?;
        let isfirst = sens == 1;
        let seq = lock(stripe)
            .set_of_surf_data()
            .cloned()
            .ok_or_else(|| Failure::new("PerformOneCorner"))?;
        let mut seq = lock(&seq);
        let surplus = |sd: &SurfDataHandle| {
            let g = lock(sd);
            g.index_of_s1() == 0 || g.index_of_s2() == 0
        };
        if isfirst {
            while (num as usize) < seq.len() && surplus(&seq[num as usize - 1]) {
                seq.remove(num as usize - 1);
            }
        } else {
            while num > 1 && surplus(&seq[num as usize - 1]) {
                seq.remove(num as usize - 1);
                num -= 1;
            }
        }
        let fd = seq
            .get(num as usize - 1)
            .cloned()
            .ok_or_else(|| Failure::out_of_range("NCollection_Sequence::ChangeValue"))?;
        Ok((fd, isfirst))
    }

    /// The face of `arc` other than `fad`.
    pub(super) fn other_face(&self, arc: &Edge, fad: &Face) -> Result<Face, Failure> {
        for f in self.my_ef_map.find_from_key(arc.as_ref())?.iter() {
            if !fad.is_same(f) {
                return topods::face(f);
            }
        }
        Ok(Face::default())
    }

    /// OnSame: the side of the fillet on its arc at the vertex (`IFadArc`), the faces, the
    /// edge to prolong; false when there is none (the corner is then an intersection at
    /// the end).
    pub(super) fn on_same_faces(
        &mut self,
        oc: &mut OneCorner,
        vtx: &Vertex,
    ) -> Result<bool, Failure> {
        let (cv1, cv2) = {
            let g = lock(&oc.fd);
            (
                g.vertex(oc.isfirst, 1).clone(),
                g.vertex(oc.isfirst, 2).clone(),
            )
        };
        if !cv1.is_on_arc() && !cv2.is_on_arc() {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "Corner OnSame : no point on arc",
            ));
        } else if cv1.is_on_arc() && cv2.is_on_arc() {
            let sur1 = has_vertex(cv1.arc()?.as_ref(), vtx)?;
            let sur2 = has_vertex(cv2.arc()?.as_ref(), vtx)?;
            if sur1 && sur2 {
                let e = [cv1.arc()?.clone(), cv2.arc()?.clone(), oc.arcspine.clone()];
                if edge_state(&e, &self.my_ef_map)? != State::OnDiff {
                    oc.ifad = 2;
                }
            } else if sur2 {
                oc.ifad = 2;
            }
        } else if cv2.is_on_arc() {
            oc.ifad = 2;
        }
        oc.ifop = 3 - oc.ifad;
        let mut arcpiv = lock(&oc.fd).vertex(oc.isfirst, oc.ifad).arc()?.clone();
        oc.fad = topods::face(self.ds.shape(lock(&oc.fd).index(oc.ifad), true))?;
        oc.fop = topods::face(self.ds.shape(lock(&oc.fd).index(oc.ifop), true))?;
        oc.fv = self.other_face(&arcpiv, &oc.fad)?;
        let isinface = has_vertex(oc.fv.as_ref(), vtx)?;
        if !isinface && lock(&oc.fd).vertex(oc.isfirst, 3 - oc.ifad).is_on_arc() {
            oc.ifad = 3 - oc.ifad;
            oc.ifop = 3 - oc.ifop;
            arcpiv = lock(&oc.fd).vertex(oc.isfirst, oc.ifad).arc()?.clone();
            oc.fad = topods::face(self.ds.shape(lock(&oc.fd).index(oc.ifad), true))?;
            oc.fop = topods::face(self.ds.shape(lock(&oc.fd).index(oc.ifop), true))?;
            oc.fv = self.other_face(&arcpiv, &oc.fad)?;
        }
        if oc.fv.is_null() {
            return Err(Failure::with_message(
                "StdFail_NotDone",
                "OneCorner : face at end not found",
            ));
        }
        oc.fv.set_orientation(Orientation::Forward);
        oc.fad.set_orientation(Orientation::Forward);
        oc.fop.set_orientation(Orientation::Forward);
        for e in self.my_ve_map.find_from_key(vtx.as_ref())?.iter() {
            if !oc.arcprol.is_null() {
                break;
            }
            if arcpiv.is_same(e) {
                continue;
            }
            let mut ex = Explorer::new(oc.fv.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
            while ex.more() {
                if e.is_same(ex.current()) {
                    oc.arcprol = topods::edge(e)?;
                    oc.oarcprolv = ex.current().orientation();
                    break;
                }
                ex.next()?;
            }
        }
        if oc.arcprol.is_null() {
            return Ok(false);
        }
        oc.oarcprolop = orientation_in(&oc.fop, &oc.arcprol, oc.oarcprolop)?;
        let mut sface = BRepTool::surface(&oc.fv)?;
        let mut prol = 0;
        extend_surface(&mut sface, &mut prol)?;
        let tol = BRepTool::face_tolerance(&oc.fv)?;
        let mut ffv = Face::default();
        BRepBuilder::default().make_face_on_surface(&mut ffv, &sface, tol)?;
        if prol != 0 {
            oc.hbs.initialize(&ffv, false)?;
            self.ds.set_new_surface(oc.fv.as_ref(), &sface);
        } else {
            oc.hbs.initialize(&oc.fv, false)?;
        }
        oc.hbop.initialize(&oc.fop, true)?;
        Ok(true)
    }

    /// The ends of the fillet moved onto the face at the end; false when there is no
    /// intersection (`None` when preparing OnSame stops here).
    pub(super) fn update_corner_ends(
        &mut self,
        oc: &mut OneCorner,
        vtx: &Vertex,
        prepare_on_same: bool,
    ) -> Result<Option<bool>, Failure> {
        let surf = self.ds.surface(lock(&oc.fd).surf()).surface().clone();
        let hgs: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&surf)?).cast();
        let hbs: Handle<dyn Adaptor3dSurface> = oc.hbs.clone().cast();
        let mut wop = lock(&oc.fd).interference(oc.ifad).parameter(oc.isfirst);
        if oc.onsame {
            let save_cp = lock(&oc.fd).vertex(oc.isfirst, oc.ifop).clone();
            let c3df = self
                .ds
                .curve(lock(&oc.fd).interference(oc.ifop).line_index())
                .curve()
                .clone();
            let inters = {
                let mut g = lock(&oc.fd);
                let (mut fi, mut cp) = (
                    g.interference(oc.ifop).clone(),
                    g.vertex(oc.isfirst, oc.ifop).clone(),
                );
                let r = inters_update_on_same(
                    &hgs,
                    &hbs,
                    &c3df,
                    (&oc.fop, &oc.fv, &oc.arcprol, vtx),
                    oc.isfirst,
                    10. * self.tolapp3d,
                    (&mut fi, &mut cp),
                    &mut oc.p2dbout,
                    &mut wop,
                )?;
                *g.change_interference(oc.ifop) = fi;
                *g.change_vertex(oc.isfirst, oc.ifop) = cp;
                r
            };
            let cpad = lock(&oc.fd).vertex(oc.isfirst, oc.ifad).clone();
            let pced: Handle<dyn Adaptor2dCurve2d> =
                Handle::new(BRepAdaptorCurve2d::with_edge_on_face(cpad.arc()?, &oc.fv)?).cast();
            let (f, l) = {
                let g = lock(&oc.fd);
                (
                    g.interference(oc.ifad).first_parameter(),
                    g.interference(oc.ifad).last_parameter(),
                )
            };
            if l - f > 10. * self.tolesp {
                self.update_side_on_edge(&oc.fd, (oc.isfirst, oc.ifad), &hbs, &pced, &hgs)?;
            }
            if prepare_on_same {
                let mut s = save_cp;
                let p = lock(&oc.fd).vertex(oc.isfirst, oc.ifop).point();
                s.set_point(p);
                *lock(&oc.fd).change_vertex(oc.isfirst, oc.ifop) = s;
                return Ok(None);
            }
            return Ok(Some(inters));
        }
        let (cv1, cv2) = {
            let g = lock(&oc.fd);
            (
                g.vertex(oc.isfirst, 1).clone(),
                g.vertex(oc.isfirst, 2).clone(),
            )
        };
        let mut fv = Face::default();
        if !self.find_face_avoid(vtx, (&cv1, &cv2), &mut fv, &oc.fop)? {
            return Ok(Some(false));
        }
        oc.fv = fv;
        oc.hbs.initialize(&oc.fv, true)?;
        for ons in [1, 2] {
            let arc = lock(&oc.fd).vertex(oc.isfirst, ons).arc()?.clone();
            let pced: Handle<dyn Adaptor2dCurve2d> =
                Handle::new(BRepAdaptorCurve2d::with_edge_on_face(&arc, &oc.fv)?).cast();
            self.update_side_on_edge(&oc.fd, (oc.isfirst, ons), &hbs, &pced, &hgs)?;
        }
        Ok(Some(true))
    }

    /// `Update(face, edonface, surf, fi, cp, isfirst)` on the side `ons` of a surface data.
    pub(super) fn update_side_on_edge(
        &self,
        fd: &SurfDataHandle,
        (isfirst, ons): (bool, i32),
        face: &Handle<dyn Adaptor3dSurface>,
        ed: &Handle<dyn Adaptor2dCurve2d>,
        surf: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<bool, Failure> {
        let mut g = lock(fd);
        let (mut fi, mut cp) = (g.interference(ons).clone(), g.vertex(isfirst, ons).clone());
        let r = update_on_edge(face, ed, surf, (&mut fi, &mut cp), isfirst)?;
        *g.change_interference(ons) = fi;
        *g.change_vertex(isfirst, ons) = cp;
        Ok(r)
    }

    /// The point on the fillet of the end of a side (at the other end without curve).
    pub(super) fn fillet_end_point(
        fd: &SurfDataHandle,
        ons: i32,
        isfirst: bool,
    ) -> Result<Point2, Failure> {
        let g = lock(fd);
        let fi = g.interference(ons);
        let t = if fi.line_index() != 0 {
            fi.parameter(isfirst)
        } else {
            fi.parameter(!isfirst)
        };
        value_2d(fi.pcurve_on_surf(), t)
    }
}
