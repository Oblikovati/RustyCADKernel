// Rust translation of ChFi3d_Builder::PerformElement, ChFi3d_Builder::FaceTangency and the
// file statics ReorderFaces, ConcatCurves and MakeOffsetEdge (ChFi3d_Builder_1.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extension of a contour of tangent edges downstream and upstream of its first edge.
use super::ChFi3dBuilder;
use super::element::make_offset_edge;
use super::extremity::tangent_on_vertex;
use crate::chfi_ds::{SpineHandle, State, lock};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use std::f64::consts::PI;

impl ChFi3dBuilder {
    /// The downstream progression of `PerformElement`: the edges joined after the last
    /// vertex; the status of the last end.
    pub(super) fn extend_downstream(
        &mut self,
        spine: &SpineHandle,
        mut ec: Edge,
        vstart: Vertex,
        mut lvec: Vertex,
        mut first_face: Face,
        offset: f64,
    ) -> Result<State, Failure> {
        let ta = self.angular;
        let mut curor = ec.orientation();
        let mut fini = false;
        let mut cur_st = State::Closed;
        while !fini {
            cur_st = State::FreeBoundary;
            let wl = BRepTool::parameter(&lvec, &ec)?;
            let degene_on_ec = tangent_on_vertex(&lvec, &ec, &self.my_ef_map, ta)?;
            let (_, v1) = BRepAdaptorCurve::with_edge(&ec)?.d1(wl)?;
            let nb = lock(spine).nb_edges();
            let edges: Vec<Shape> = self
                .my_ve_map
                .find_from_key(lvec.as_ref())?
                .iter()
                .cloned()
                .collect();
            for s in &edges {
                let ev = topods::edge(s)?;
                if ev.is_same(&ec) || BRepTool::degenerated(&ev)? {
                    continue;
                }
                let (mut fvev, mut lvev) = topexp::vertices(&ev, false);
                let or1 = if lvec.is_same(&lvev) {
                    std::mem::swap(&mut fvev, &mut lvev);
                    Orientation::Reversed
                } else {
                    Orientation::Forward
                };
                let wf = BRepTool::parameter(&fvev, &ev)?;
                let (_, v2) = BRepAdaptorCurve::with_edge(&ev)?.d1(wf)?;
                let av1v2 = v1.angle(v2)?;
                let rev = or1 != curor;
                if self.joins(&ec, &ev, &fvev, &lvec, av1v2, rev, degene_on_ec)? {
                    fini = false;
                    let prev_edge = ec.clone();
                    ec = ev.clone();
                    ec.set_orientation(or1);
                    lvec = lvev.clone();
                    lock(spine).set_edges(&ec);
                    let (cf1, cf2) = self.joined_faces(&ec, &prev_edge, &ev, &first_face)?;
                    if offset > 0. {
                        let (sb1, sb2) = (
                            BRepAdaptorSurface::with_face(&cf1, true)?,
                            BRepAdaptorSurface::with_face(&cf2, true)?,
                        );
                        let mut an_offset_edge = make_offset_edge(&ec, offset, &sb1, &sb2)?;
                        an_offset_edge.set_orientation(or1);
                        lock(spine).set_offset_edges(&an_offset_edge);
                    }
                    first_face = cf1;
                    curor = or1;
                    if vstart.is_same(&lvev) {
                        let e1 = lock(spine).edges(1)?.clone();
                        cur_st = if self.face_tangency(&ev, &e1, &lvev)? {
                            State::Closed
                        } else {
                            State::BreakPoint
                        };
                        fini = true;
                    }
                    break;
                }
                if self.nb_faces(&ev)? > 1 {
                    cur_st = State::BreakPoint;
                }
                fini = (!rev && av1v2 < ta) || (rev && (PI - av1v2) < ta);
            }
            fini = fini || nb == lock(spine).nb_edges();
        }
        Ok(cur_st)
    }

    /// The upstream progression of `PerformElement`: the edges joined before the first
    /// vertex; the status of the first end.
    pub(super) fn extend_upstream(
        &mut self,
        spine: &SpineHandle,
        vstart: Vertex,
        offset: f64,
    ) -> Result<State, Failure> {
        let ta = self.angular;
        let mut ec = lock(spine).edges(1)?.clone();
        let mut first_face =
            topods::face(self.my_edge_first_face.seek(ec.as_ref()).ok_or_else(|| {
                Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
            })?)?;
        let mut curor = ec.orientation();
        let mut fvec = vstart;
        let mut fini = false;
        let mut cur_st = State::FreeBoundary;
        while !fini {
            cur_st = State::FreeBoundary;
            let wl = BRepTool::parameter(&fvec, &ec)?;
            let degene_on_ec = tangent_on_vertex(&fvec, &ec, &self.my_ef_map, ta)?;
            let (_, v1) = BRepAdaptorCurve::with_edge(&ec)?.d1(wl)?;
            let nb = lock(spine).nb_edges();
            let edges: Vec<Shape> = self
                .my_ve_map
                .find_from_key(fvec.as_ref())?
                .iter()
                .cloned()
                .collect();
            for s in &edges {
                let ev = topods::edge(s)?;
                if ev.is_same(&ec) || BRepTool::degenerated(&ev)? {
                    continue;
                }
                let (mut fvev, mut lvev) = topexp::vertices(&ev, false);
                let or1 = if fvec.is_same(&fvev) {
                    std::mem::swap(&mut fvev, &mut lvev);
                    Orientation::Reversed
                } else {
                    Orientation::Forward
                };
                let wf = BRepTool::parameter(&lvev, &ev)?;
                let (_, v2) = BRepAdaptorCurve::with_edge(&ev)?.d1(wf)?;
                let av1v2 = v1.angle(v2)?;
                let rev = or1 != curor;
                if self.joins(&ec, &ev, &lvev, &fvec, av1v2, rev, degene_on_ec)? {
                    let prev_edge = ec.clone();
                    ec = ev.clone();
                    ec.set_orientation(or1);
                    fvec = fvev.clone();
                    lock(spine).put_in_first(&ec);
                    let (cf1, cf2) = self.joined_faces(&ec, &prev_edge, &ev, &first_face)?;
                    if offset > 0. {
                        let (sb1, sb2) = (
                            BRepAdaptorSurface::with_face(&cf1, true)?,
                            BRepAdaptorSurface::with_face(&cf2, true)?,
                        );
                        let mut an_offset_edge = make_offset_edge(&ec, offset, &sb1, &sb2)?;
                        an_offset_edge.set_orientation(or1);
                        lock(spine).put_in_first_offset(&an_offset_edge);
                    }
                    first_face = cf1;
                    curor = or1;
                    break;
                }
                if self.nb_faces(&ev)? > 1 {
                    cur_st = State::BreakPoint;
                }
                fini = (!rev && av1v2 < ta) || (rev && (PI - av1v2) < ta);
            }
            fini = fini || nb == lock(spine).nb_edges();
        }
        Ok(cur_st)
    }
}
