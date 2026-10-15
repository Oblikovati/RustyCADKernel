// Rust translation of ChFi3d_Builder::StartSol(Spine, HS, pons, HC, W, SD, isfirst, ons,
// HSref, HCref, RecP, RecS, RecRst, c1obstacle, HSBis, PBis, decroch, Vref) and
// ChFi3d_Builder::SearchFace (ChFi3d_Builder_2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The start of the walking on an obstacle or in the extension of the spine.
use super::ChFi3dBuilder;
use super::faces_search::{curve_on_surface, pcurve_value};
use super::start_obstacle::ObstacleFlags;
use super::start_obstacle::ObstacleStart;
use super::start_obstacle::edge_in_face;
use crate::chfi_ds::{CommonPoint, SpineHandle, SurfDataHandle, lock};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl ChFi3dBuilder {
    /// The start on the obstacle `fv` of a sharp arc of the common point (unless the arc
    /// ends at `vref`).
    pub(super) fn start_on_obstacle(
        f: &Face,
        fv: &Face,
        cp: &CommonPoint,
        vref: &Vertex,
        st: &mut ObstacleStart<'_>,
        flags: &mut ObstacleFlags,
    ) -> Result<bool, Failure> {
        let an_arc = cp.arc()?.clone();
        flags.c1obstacle = true;
        if !vref.is_null() {
            let mut ex = Explorer::new(an_arc.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)?;
            while ex.more() {
                if ex.current().is_same(vref.as_ref()) {
                    flags.c1obstacle = false;
                    break;
                }
                ex.next()?;
            }
        }
        if !flags.c1obstacle || st.hsref.is_null() || st.hcref.is_null() {
            Self::default_start(f, cp, st)?;
            return Ok(false);
        }
        st.hs.initialize(fv, true)?;
        st.hsref.initialize(f, true)?;
        *st.w = cp.parameter_on_arc()?;
        *st.hc = Handle::new(BRepAdaptorCurve2d::default());
        let mut newface = fv.clone();
        newface.set_orientation(Orientation::Forward);
        let newedge = edge_in_face(&newface, &an_arc)?;
        if newedge.is_null() {
            Self::default_start(f, cp, st)?;
            return Ok(false);
        }
        st.hc.initialize(&newedge, fv)?;
        *st.pons = st.hc.value(*st.w)?;
        st.hcref.initialize(&an_arc, f)?;
        if cp.is_vertex() {
            flags.rec_p = true;
        } else {
            flags.rec_rst = true;
        }
        Ok(true)
    }

    /// The common point of a surface data on its arc with the other one too, in the
    /// extension zone: the start on the face at the other parameter.
    pub(super) fn start_in_extension(
        &self,
        sd: &SurfDataHandle,
        (isfirst, ons): (bool, i32),
        f: &Face,
        st: &mut ObstacleStart<'_>,
        flags: &mut ObstacleFlags,
    ) -> Result<Option<bool>, Failure> {
        let g = lock(sd);
        let notons = if ons == 1 { 2 } else { 1 };
        let (cp, cpbis) = (g.vertex(isfirst, ons), g.vertex(isfirst, notons));
        if !cp.is_on_arc() || !cpbis.is_on_arc() {
            return Ok(None);
        }
        let ts = g.interference(ons).parameter(isfirst);
        let tns = g.interference(notons).parameter(isfirst);
        let tolesp = self.tolesp;
        let is_extend = if isfirst {
            ts - tns > 100. * tolesp
        } else {
            tns - ts > 100. * tolesp
        };
        if is_extend && !cp.point().is_equal(&cpbis.point(), 0.) {
            let pc = g.interference(ons).pcurve_on_face().clone();
            if pc.is_null() {
                return Ok(Some(false));
            }
            st.hs.initialize(f, true)?;
            flags.rec_s = false;
            *st.pons = pcurve_value(&pc, tns)?;
            return Ok(Some(false));
        }
        Ok(None)
    }

    /// `StartSol(Spine, HS, pons, HC, W, SD, isfirst, ons, HSref, HCref, RecP, RecS,
    /// RecRst, c1obstacle, HSBis, PBis, decroch, Vref)`: the face and the point the fillet
    /// goes on from at the end of `sd` on its face `ons`, and its obstacle.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start_sol_obstacle(
        &self,
        spine: &SpineHandle,
        sd: &SurfDataHandle,
        (isfirst, ons): (bool, i32),
        st: &mut ObstacleStart<'_>,
        decroch: bool,
        vref: &Vertex,
    ) -> Result<(bool, ObstacleFlags), Failure> {
        let mut flags = ObstacleFlags::default();
        let fref = if st.hsref.is_null() {
            Face::default()
        } else {
            st.hsref.face()
        };
        let f = topods::face(self.ds.shape(lock(sd).index(ons), true))?;
        if f.is_null() {
            return Ok((false, flags));
        }
        if let Some(r) = self.start_in_extension(sd, (isfirst, ons), &f, st, &mut flags)? {
            return Ok((r, flags));
        }
        let cp = lock(sd).vertex(isfirst, ons).clone();
        if cp.is_vertex() && !st.hc.is_null() && !decroch {
            let r = self.obstacle_at_vertex(sd, (isfirst, ons), &cp, st, &mut flags)?;
            return Ok((r, flags));
        }
        if cp.is_on_arc() && !st.hc.is_null() && !decroch {
            *st.w = cp.parameter_on_arc()?;
            flags.c1obstacle = true;
            return Ok((true, flags));
        }
        *st.hc = Handle::default();
        if !cp.is_on_arc() {
            st.hs.initialize(&f, true)?;
            let g = lock(sd);
            let fi = g.interference(ons);
            *st.pons = if fi.pcurve_on_face().is_null() {
                g.get_2d_point(isfirst, ons)
            } else {
                pcurve_value(fi.pcurve_on_face(), fi.parameter(isfirst))?
            };
            return Ok((true, flags));
        }
        let an_arc = cp.arc()?.clone();
        if an_arc.is_null() {
            return Ok((false, flags));
        }
        if decroch {
            let pc = if fref.is_null() {
                Handle::default()
            } else {
                curve_on_surface(&an_arc, &fref)?
            };
            if pc.is_null() {
                Self::default_start(&f, &cp, st)?;
                return Ok((false, flags));
            }
            st.hs.initialize(&fref, true)?;
            *st.w = cp.parameter_on_arc()?;
            *st.pons = pcurve_value(&pc, *st.w)?;
            flags.rec_s = true;
            return Ok((true, flags));
        }
        let mut fv = Face::default();
        if self.search_face(Some(spine), &cp, &f, &mut fv)? {
            let r = self.start_on_tangent_face(&f, fv, &cp, st, &mut flags)?;
            return Ok((r, flags));
        }
        if !fv.is_null() {
            let r = Self::start_on_obstacle(&f, &fv, &cp, vref, st, &mut flags)?;
            return Ok((r, flags));
        }
        Self::default_start(&f, &cp, st)?;
        Ok((false, flags))
    }
}
