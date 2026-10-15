// Rust translation of ChFi3d_Builder::StartSol(Spine, HS, pons, HC, W, SD, isfirst, ons,
// HSref, HCref, RecP, RecS, RecRst, c1obstacle, HSBis, PBis, decroch, Vref) and
// ChFi3d_Builder::SearchFace (ChFi3d_Builder_2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The face a fillet goes on to at the end of a surface data: across the arc of its
//! common point, or along an obstacle.
use super::ChFi3dBuilder;
use super::faces_search::{
    curve_on_surface, is_g1, is_input, is_tangent_to_arc, pcurve_value, search_face_on_v,
};
use crate::chfi_ds::{CommonPoint, SpineHandle, SurfDataHandle, lock};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;

/// The outputs of `StartSol` on an obstacle: `RecP`, `RecS`, `RecRst`, `c1obstacle` and
/// the support face `HSBis` with its point `PBis`.
#[derive(Default)]
pub(crate) struct ObstacleFlags {
    pub rec_p: bool,
    pub rec_s: bool,
    pub rec_rst: bool,
    pub c1obstacle: bool,
    pub hs_bis: Handle<BRepAdaptorSurface>,
    pub p_bis: Point2,
}

/// The in-out arguments of `StartSol` on an obstacle.
pub(crate) struct ObstacleStart<'a> {
    /// `HS`: the new face.
    pub hs: &'a Handle<BRepAdaptorSurface>,
    /// `pons`: the start point on it.
    pub pons: &'a mut Point2,
    /// `HC`: the obstacle on the new face.
    pub hc: &'a mut Handle<BRepAdaptorCurve2d>,
    /// `W`: the parameter on the obstacle.
    pub w: &'a mut f64,
    /// `HSref`: the other face of the obstacle.
    pub hsref: &'a Handle<BRepAdaptorSurface>,
    /// `HCref`: the obstacle on it.
    pub hcref: &'a Handle<BRepAdaptorCurve2d>,
}

/// The edge of a face that is the same as `e`, with its orientation in the face.
pub(super) fn edge_in_face(face: &Face, e: &Edge) -> Result<Edge, Failure> {
    let mut ex = Explorer::new(face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while ex.more() {
        if ex.current().is_same(e.as_ref()) {
            return topods::edge(ex.current());
        }
        ex.next()?;
    }
    Ok(Edge::default())
}

/// An edge of `face` other than `except` through the vertex `v`, tangent across to
/// another face (`edgereg` and `facereg`) or not (returned).
pub(super) fn next_edge_at(
    face: &Face,
    except: &Edge,
    v: &Vertex,
    ef_map: &crate::chfi_ds::Map,
    reg: &mut (Edge, Face),
) -> Result<Edge, Failure> {
    let mut fv = Face::default();
    let mut ex1 = Explorer::new(face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while ex1.more() {
        let cured = topods::edge(ex1.current())?;
        ex1.next()?;
        if cured.is_same(except) {
            continue;
        }
        let mut found = false;
        let mut ex2 = Explorer::new(cured.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)?;
        while ex2.more() && !found {
            if ex2.current().is_same(v.as_ref()) {
                if is_g1(ef_map, &cured, face, &mut fv)? {
                    *reg = (cured.clone(), fv.clone());
                } else {
                    found = true;
                }
            }
            ex2.next()?;
        }
        if found {
            return Ok(cured);
        }
    }
    Ok(Edge::default())
}

impl ChFi3dBuilder {
    /// `SearchFace(Spine, Pc, FRef, FVoi)`: the face tangent to `fref` across the arc (or
    /// at the vertex) of the common point.
    pub(crate) fn search_face(
        &self,
        spine: Option<&SpineHandle>,
        pc: &CommonPoint,
        fref: &Face,
        fvoi: &mut Face,
    ) -> Result<bool, Failure> {
        if !pc.is_on_arc() {
            return Ok(false);
        }
        *fvoi = Face::default();
        if !pc.is_vertex() {
            return is_g1(&self.my_ef_map, pc.arc()?, fref, fvoi);
        }
        if pc.has_vector() {
            let mut fbis = Face::default();
            return Ok(search_face_on_v(
                pc,
                fref,
                &self.my_ve_map,
                &self.my_ef_map,
                fvoi,
                &mut fbis,
            )? > 0);
        }
        let vtx = pc.vertex()?.clone();
        let sp = spine.ok_or_else(|| Failure::with_message("Standard_NullObject", "SearchFace"))?;
        let (_, mut vec_spine) = lock(sp).d1(pc.parameter())?;
        let mut find_face = is_input(vec_spine, &vtx, fref)?;
        if find_face {
            vec_spine.reverse();
            find_face = is_input(vec_spine, &vtx, fref)?;
        }
        if find_face {
            *fvoi = fref.clone();
            return Ok(true);
        }
        for s in self.my_ve_map.find_from_key(vtx.as_ref())?.iter() {
            if find_face {
                break;
            }
            let e = topods::edge(s)?;
            let trouve = self
                .my_ef_map
                .find_from_key(e.as_ref())?
                .iter()
                .any(|f| f.is_same(fref.as_ref()));
            if trouve {
                find_face = is_g1(&self.my_ef_map, &e, fref, fvoi)?;
            }
            if find_face {
                find_face = self.spine_edge_on_face(sp, &vtx, fvoi)?;
            }
        }
        Ok(false)
    }

    /// An edge of the spine at the vertex lying on the face.
    pub(super) fn spine_edge_on_face(
        &self,
        sp: &SpineHandle,
        vtx: &Vertex,
        fvoi: &Face,
    ) -> Result<bool, Failure> {
        let nb = lock(sp).nb_edges();
        let mut trouve = false;
        let mut ie = 1;
        while ie <= nb && !trouve {
            let e = lock(sp).edges(ie)?.clone();
            if topexp::first_vertex(&e, false).is_same(vtx)
                || topexp::last_vertex(&e, false).is_same(vtx)
            {
                trouve = self
                    .my_ef_map
                    .find_from_key(e.as_ref())?
                    .iter()
                    .any(|f| f.is_same(fvoi.as_ref()));
            }
            ie += 1;
        }
        Ok(trouve)
    }

    /// The obstacle continued at the vertex of the common point: the next sharp edge of
    /// the reference face (or of its tangent neighbour), its faces and the start on it.
    pub(super) fn obstacle_at_vertex(
        &self,
        sd: &SurfDataHandle,
        (isfirst, ons): (bool, i32),
        cp: &CommonPoint,
        st: &mut ObstacleStart<'_>,
        flags: &mut ObstacleFlags,
    ) -> Result<bool, Failure> {
        let vcp = cp.vertex()?.clone();
        let ehc = st.hc.edge();
        let mut fref = if st.hsref.is_null() {
            Face::default()
        } else {
            st.hsref.face()
        };
        if vcp.is_null()
            || ehc.is_null()
            || fref.is_null()
            || st.hsref.is_null()
            || st.hcref.is_null()
        {
            return Ok(false);
        }
        let mut bidface = fref.clone();
        bidface.set_orientation(Orientation::Forward);
        let mut reg = (Edge::default(), Face::default());
        let mut newedge = next_edge_at(&bidface, &ehc, &vcp, &self.my_ef_map, &mut reg)?;
        let mut fv = Face::default();
        if newedge.is_null() {
            let (v1, v2) = topexp::vertices(&ehc, false);
            if v1.is_same(&v2) {
                let (w1, w2) = (
                    BRepTool::parameter(&v1, &ehc)?,
                    BRepTool::parameter(&v2, &ehc)?,
                );
                let pww = {
                    let g = lock(sd);
                    let fi = g.interference(ons);
                    if fi.pcurve_on_face().is_null() {
                        g.get_2d_point(isfirst, ons)
                    } else {
                        pcurve_value(fi.pcurve_on_face(), fi.parameter(isfirst))?
                    }
                };
                let (p1, p2) = (st.hc.value(w1)?, st.hc.value(w2)?);
                if p1.distance(pww) > p2.distance(pww) {
                    *st.w = w1;
                    *st.pons = p1;
                } else {
                    *st.w = w2;
                    *st.pons = p2;
                }
                flags.rec_p = true;
                flags.c1obstacle = true;
                return Ok(true);
            } else if !reg.0.is_null() {
                fref = reg.1.clone();
                st.hsref.initialize(&fref, true)?;
                newedge = self.first_sharp_edge(&fref, &reg.0, &vcp, &mut fv)?;
            }
        }
        if newedge.is_null() {
            return Err(Failure::new(
                "StartSol : chain is not possible, new obstacle not found",
            ));
        }
        if is_g1(&self.my_ef_map, &newedge, &fref, &mut fv)? {
            return Err(Failure::new(
                "StartSol : chain is not possible, config non processed",
            ));
        }
        if fv.is_null() {
            return Err(Failure::new(
                "StartSol : chain is not possible, new obstacle not found",
            ));
        }
        if curve_on_surface(&newedge, &fref)?.is_null() {
            return Ok(false);
        }
        let mut newface = fv.clone();
        newface.set_orientation(Orientation::Forward);
        let in_face = edge_in_face(&newface, &newedge)?;
        if !in_face.is_null() {
            newedge = in_face;
        }
        if curve_on_surface(&newedge, &fv)?.is_null() {
            return Ok(false);
        }
        st.hs.initialize(&fv, true)?;
        *st.w = BRepTool::parameter(&vcp, &newedge)?;
        st.hcref.initialize(&newedge, &fref)?;
        st.hc.initialize(&newedge, &fv)?;
        *st.pons = st.hc.value(*st.w)?;
        flags.rec_p = true;
        flags.c1obstacle = true;
        Ok(true)
    }

    /// The last edge of `face` other than `except` through `v` that is not tangent across
    /// (`fv` left at the last neighbour examined).
    pub(super) fn first_sharp_edge(
        &self,
        face: &Face,
        except: &Edge,
        v: &Vertex,
        fv: &mut Face,
    ) -> Result<Edge, Failure> {
        let mut newedge = Edge::default();
        let mut ex1 = Explorer::new(face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ex1.more() && newedge.is_null() {
            let cured = topods::edge(ex1.current())?;
            ex1.next()?;
            if cured.is_same(except) {
                continue;
            }
            let mut ex2 = Explorer::new(cured.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)?;
            while ex2.more() {
                if ex2.current().is_same(v.as_ref()) && !is_g1(&self.my_ef_map, &cured, face, fv)? {
                    newedge = cured.clone();
                }
                ex2.next()?;
            }
        }
        Ok(newedge)
    }

    /// `prepareDefaultReturn`: the start on the face `f` at the parameter of the common
    /// point on its arc.
    pub(super) fn default_start(
        f: &Face,
        cp: &CommonPoint,
        st: &mut ObstacleStart<'_>,
    ) -> Result<bool, Failure> {
        let pc = curve_on_surface(cp.arc()?, f)?;
        if pc.is_null() {
            return Ok(false);
        }
        st.hs.initialize(f, true)?;
        *st.w = cp.parameter_on_arc()?;
        *st.pons = pcurve_value(&pc, *st.w)?;
        Ok(true)
    }

    /// The start across the arc of the common point onto the face `fv` tangent to `f`.
    pub(super) fn start_on_tangent_face(
        &self,
        f: &Face,
        mut fv: Face,
        cp: &CommonPoint,
        st: &mut ObstacleStart<'_>,
        flags: &mut ObstacleFlags,
    ) -> Result<bool, Failure> {
        let an_arc = cp.arc()?.clone();
        if fv.is_null() {
            Self::default_start(f, cp, st)?;
            return Ok(false);
        }
        st.hs.initialize(&fv, true)?;
        flags.rec_s = true;
        if cp.is_vertex() {
            let mut aux = Face::default();
            let nb = search_face_on_v(cp, f, &self.my_ve_map, &self.my_ef_map, &mut fv, &mut aux)?;
            *st.pons = BRepTool::parameters(cp.vertex()?, &fv)?;
            st.hs.initialize(&fv, true)?;
            if nb >= 2 {
                flags.hs_bis = Handle::new(BRepAdaptorSurface::with_face(&aux, true)?);
                flags.p_bis = BRepTool::parameters(cp.vertex()?, &aux)?;
            }
            return Ok(true);
        }
        let a_pcurve = if !fv.is_same(f) {
            fv.set_orientation(Orientation::Forward);
            let newedge = edge_in_face(&fv, &an_arc)?;
            if is_tangent_to_arc(cp, 0.1)? {
                let pc = curve_on_surface(&an_arc, f)?;
                if pc.is_null() {
                    Self::default_start(f, cp, st)?;
                    return Ok(false);
                }
                flags.hs_bis = Handle::new(BRepAdaptorSurface::with_face(f, true)?);
                flags.p_bis = pcurve_value(&pc, cp.parameter_on_arc()?)?;
            }
            if newedge.is_null() {
                Self::default_start(f, cp, st)?;
                return Ok(false);
            }
            curve_on_surface(&newedge, &fv)?
        } else {
            let mut newedge = an_arc.clone();
            newedge.reverse();
            fv.set_orientation(Orientation::Forward);
            curve_on_surface(&newedge, &fv)?
        };
        if a_pcurve.is_null() {
            Self::default_start(f, cp, st)?;
            return Ok(false);
        }
        *st.pons = pcurve_value(&a_pcurve, cp.parameter_on_arc()?)?;
        Ok(true)
    }
}
