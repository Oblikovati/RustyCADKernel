//! HLRBRep_PolyAlgo.cxx: UpdateOutLines, UpdateEdgesBiPoints and UpdatePolyData.
use super::PolyState;
use super::orient::{Arrays, data, node_h, seg, set_tri, tri};
use crate::hlr_algo::{
    BiPoint, BiPointFace, BiPointPoints, PolyData, PolyHidingData, PolyInternalData, PolyMask,
    TriangleData,
};
use rustycad_tkernel::ncollection::{HArray1, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Xyz;

const SIDE: i32 = PolyMask::FMskSide as i32;
const BACK: i32 = PolyMask::FMskBack as i32;
const FLAT: i32 = PolyMask::FMskFlat as i32;
const OR_BACK: i32 = PolyMask::FMskOrBack as i32;
const HIDING: i32 = PolyMask::FMskHiding as i32;

/// Whether the edge between two triangles is on the outline: one turned to the back
/// and the other not, or one on the side (not flat) next to a front one.
fn is_outline(f1: i32, f2: i32) -> bool {
    let (side1, side2) = (f1 & SIDE != 0, f2 & SIDE != 0);
    if !side1 && !side2 {
        (f1 & BACK) != (f2 & BACK)
    } else if side1 && side2 {
        false
    } else if side1 {
        f1 & FLAT == 0 && f2 & BACK == 0
    } else {
        f2 & FLAT == 0 && f1 & BACK == 0
    }
}

/// The outline bit of the side `(pd, pf)` of a triangle (its nodes reversed when it
/// is turned to the back and not on the side).
fn outline_bit(t: &TriangleData, pd: i32, pf: i32) -> i32 {
    let (mut tn1, tn2, mut tn3) = (t.node1, t.node2, t.node3);
    if t.flags & SIDE == 0 && t.flags & OR_BACK != 0 {
        std::mem::swap(&mut tn1, &mut tn3);
    }
    let is = |a: i32, b: i32| (a == pd && b == pf) || (a == pf && b == pd);
    if is(tn1, tn2) {
        PolyMask::EMskOutLin1 as i32
    } else if is(tn2, tn3) {
        PolyMask::EMskOutLin2 as i32
    } else if is(tn3, tn1) {
        PolyMask::EMskOutLin3 as i32
    } else {
        0
    }
}

impl PolyState {
    /// UpdateOutLines(List, PID): on the faces with an outline, each inner segment
    /// between triangles on both sides of the outline marks their sides and becomes
    /// an internal outline segment of the face.
    pub(super) fn update_outlines(
        &self,
        list: &mut List<BiPoint>,
        pids: &[Handle<PolyInternalData>],
    ) -> Result<(), Failure> {
        for (face_iter, pid) in pids.iter().enumerate().skip(1) {
            let Some(pid) = pid.get() else {
                continue;
            };
            if !pid.int_out_l() {
                continue;
            }
            let a = Arrays::of(pid);
            for seg_iter in 1..=pid.nb_pi_seg() {
                let psg = seg(&a.s, seg_iter)?;
                let (it1, it2) = (psg.conex1, psg.conex2);
                if it1 == 0 || it2 == 0 || it1 == it2 {
                    continue;
                }
                let (mut t1, mut t2) = (tri(&a.t, it1)?, tri(&a.t, it2)?);
                if !is_outline(t1.flags, t2.flags) {
                    continue;
                }
                let (pd, pf) = (psg.lst_sg1, psg.lst_sg2);
                t1.flags |= outline_bit(&t1, pd, pf);
                set_tri(&a.t, it1, t1)?;
                t2.flags |= outline_bit(&t2, pd, pf);
                set_tri(&a.t, it2, t2)?;
                let p1 = data(&node_h(&a.n, pd)?)?.point;
                let p2 = data(&node_h(&a.n, pf)?)?.point;
                let (mut ti1, mut ti2) = (p1, p2);
                self.ti.apply_xyz(&mut ti1, false);
                self.ti.apply_xyz(&mut ti2, false);
                let f = face_iter as i32;
                let face = BiPointFace {
                    conex: f,
                    pt1: pd,
                    pt2: pf,
                };
                let pts = BiPointPoints {
                    pnt1: ti1,
                    pnt2: ti2,
                    pnt_p1: p1,
                    pnt_p2: p2,
                };
                list.append(BiPoint::with_faces_flags(pts, f, [face, face], 12));
            }
        }
        Ok(())
    }

    /// UpdateEdgesBiPoints(List, PID, closed): each segment between two faces is hidden
    /// on a closed shell when its triangles are both turned away (or on the side), and
    /// is an outline when they lie on both sides of the outline.
    pub(super) fn update_edges_bipoints(
        &self,
        list: &mut List<BiPoint>,
        pids: &[Handle<PolyInternalData>],
        is_closed: bool,
    ) -> Result<(), Failure> {
        let pid_at = |i: i32| -> Result<&PolyInternalData, Failure> {
            pids.get(i as usize).and_then(|h| h.get()).ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "HLRBRep_PolyAlgo::UpdateEdgesBiPoints: a face without internal data",
                )
            })
        };
        for bp in list.iter_mut() {
            let ind = *bp.indices();
            if ind.face_conex1 == 0 || ind.face_conex2 == 0 {
                continue;
            }
            let (a1, a2) = (
                Arrays::of(pid_at(ind.face_conex1)?),
                Arrays::of(pid_at(ind.face_conex2)?),
            );
            let n11 = node_h(&a1.n, ind.face1_pt1)?;
            let n21 = node_h(&a2.n, ind.face2_pt1)?;
            let (_, itri1, _) = self.triangles(ind.face1_pt1, ind.face1_pt2, &n11, &a1.s)?;
            let (_, itri2, _) = self.triangles(ind.face2_pt1, ind.face2_pt2, &n21, &a2.s)?;
            if itri1 == 0 || itri2 == 0 {
                continue;
            }
            if ind.face_conex1 == ind.face_conex2 && itri1 == itri2 {
                continue;
            }
            let (f1, f2) = (tri(&a1.t, itri1)?.flags, tri(&a2.t, itri2)?.flags);
            let hidden = |a: i32, b: i32| {
                (a & BACK != 0 || a & SIDE != 0) && (b & BACK != 0 || b & SIDE != 0)
            };
            if is_closed && hidden(f1, f2) {
                bp.set_hidden(true);
            }
            bp.set_out_line(is_outline(f1, f2));
        }
        Ok(())
    }

    /// UpdatePolyData(PD, PID, closed): the polyhedral data of each face: its node
    /// points, its triangles (turned to the eye, the hiding ones flagged) and room for
    /// the hiding data.
    pub(super) fn update_poly_data(
        &self,
        pds: &[Handle<PolyData>],
        pids: &[Handle<PolyInternalData>],
        is_closed: bool,
    ) -> Result<(), Failure> {
        for (face_iter, pid) in pids.iter().enumerate().skip(1) {
            let Some(pid) = pid.get() else {
                continue;
            };
            let (nb_nodes, nb_tris) = (pid.nb_pi_nod(), pid.nb_t_data());
            let a = Arrays::of(pid);
            let mut nodes = rustycad_tkernel::ncollection::Array1::<Xyz>::with_bounds(1, nb_nodes)
                .map_err(|e| e.into_failure("HLRBRep_PolyAlgo::UpdatePolyData"))?;
            for i in 1..=nb_nodes {
                *nodes
                    .change_value(i)
                    .map_err(|e| e.into_failure("HLRBRep_PolyAlgo::UpdatePolyData"))? =
                    data(&node_h(&a.n, i)?)?.point;
            }
            let mut trian =
                rustycad_tkernel::ncollection::Array1::<TriangleData>::with_bounds(1, nb_tris)
                    .map_err(|e| e.into_failure("HLRBRep_PolyAlgo::UpdatePolyData"))?;
            let mut nb_hidden = 0;
            for i in 1..=nb_tris {
                let mut ot = tri(&a.t, i)?;
                if ot.flags & SIDE == 0 {
                    if ot.flags & OR_BACK != 0 {
                        std::mem::swap(&mut ot.node1, &mut ot.node3);
                        ot.flags |= BACK;
                    } else {
                        ot.flags &= !BACK;
                    }
                }
                set_tri(&a.t, i, ot)?;
                let mut nt = ot;
                if nt.flags & SIDE == 0 && (nt.flags & BACK == 0 || !is_closed) {
                    nt.flags |= HIDING;
                    nb_hidden += 1;
                } else {
                    nt.flags &= !HIDING;
                }
                *trian
                    .change_value(i)
                    .map_err(|e| e.into_failure("HLRBRep_PolyAlgo::UpdatePolyData"))? = nt;
            }
            let hp_dat = if nb_hidden > 0 {
                Handle::new(
                    HArray1::<PolyHidingData>::with_bounds(1, nb_hidden)
                        .map_err(|e| e.into_failure("HLRBRep_PolyAlgo::UpdatePolyData"))?,
                )
            } else {
                Handle::default()
            };
            let pd = pds.get(face_iter).and_then(|h| h.get()).ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "HLRBRep_PolyAlgo::UpdatePolyData: a face without polyhedral data",
                )
            })?;
            pd.set_h_nodes(Handle::new(HArray1::from_owned(nodes)));
            pd.set_h_t_data(Handle::new(HArray1::from_owned(trian)));
            pd.set_h_ph_dat(hp_dat);
            pd.set_face_index(face_iter as i32);
        }
        Ok(())
    }
}
