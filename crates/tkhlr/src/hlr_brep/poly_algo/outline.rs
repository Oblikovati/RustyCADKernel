//! HLRBRep_PolyAlgo.cxx: InsertOnOutLine and CheckFrBackTriangles.
use super::orient::{Arrays, NodeH, add_flag, data, node_h, pin, seg, tri};
use super::{Frame, NMSK_EDGE, NMSK_MOVE, NMSK_OUT_L, NMSK_VERT, PolyState};
use crate::hlr_algo::{BiPoint, PolyInternalData, PolyMask};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Xyz;

const SIDE: i32 = PolyMask::FMskSide as i32;
const FR_BACK: i32 = PolyMask::FMskFrBack as i32;

impl PolyState {
    /// InsertOnOutLine(PID): on every face, the inner segments whose nodes look at the
    /// eye from opposite sides get a node on the outline (or their near node is moved
    /// onto it); the faces with an outline are marked.
    pub(super) fn insert_on_outline(
        &mut self,
        pids: &[Handle<PolyInternalData>],
    ) -> Result<(), Failure> {
        for (face_iter, pid) in pids.iter().enumerate().skip(1) {
            let Some(pid) = pid.get() else {
                continue;
            };
            let face = topods::face(self.f_map.find_key(face_iter)?)?;
            self.b_surf.initialize(&face, false)?;
            let (surf, loc) = BRepTool::surface_with_location(&face)?;
            self.g_surf = surf;
            let mut tt = loc.transformation();
            tt.pre_multiply(self.proj.transformation());
            self.tt = Frame::of(&tt)?;
            let mut a = Arrays::of(pid);
            let mut other = Arrays::none();
            let mut is_int_out_l = false;
            for seg_iter in 1..=pid.nb_pi_seg() {
                if self.insert_on_segment(pid, &mut a, &mut other, seg_iter)? {
                    is_int_out_l = true;
                }
            }
            if is_int_out_l {
                pid.set_int_out_l(true);
            }
        }
        Ok(())
    }

    // One inner segment; true when it is on the outline or crosses it.
    fn insert_on_segment(
        &self,
        pid: &PolyInternalData,
        a: &mut Arrays,
        other: &mut Arrays,
        seg_iter: i32,
    ) -> Result<bool, Failure> {
        let sg = seg(&a.s, seg_iter)?;
        if sg.conex1 == 0 || sg.conex2 == 0 {
            return Ok(false);
        }
        let (ip1, ip2) = (sg.lst_sg1, sg.lst_sg2);
        let (n1, n2) = (node_h(&a.n, ip1)?, node_h(&a.n, ip2)?);
        let (d1, d2) = (data(&n1)?, data(&n2)?);
        if pin(&n1)?.indices().flag & NMSK_OUT_L != 0 && pin(&n2)?.indices().flag & NMSK_OUT_L != 0
        {
            return Ok(true);
        }
        let tol = self.tol_angular;
        if !((d1.scal >= tol && d2.scal <= -tol) || (d2.scal >= tol && d1.scal <= -tol)) {
            return Ok(false);
        }
        let Some((coef3, m_p1)) = self.new_node(&d1, &d2) else {
            return Ok(true);
        };
        let p3 = self.outline_point(&d1, &d2, coef3)?;
        let mut ins = true;
        if m_p1 && pin(&n1)?.indices().flag & NMSK_EDGE == 0 && coef3 < self.tol_sta {
            ins = false;
            self.change_node((ip1, ip2), &n1, &n2, coef3, p3, true, a)?;
            pin(&n1)?.data().scal = 0.;
            add_flag(&n1, NMSK_OUT_L)?;
        }
        if ins && !m_p1 && pin(&n2)?.indices().flag & NMSK_EDGE == 0 && coef3 > self.tol_end {
            ins = false;
            self.change_node((ip1, ip2), &n1, &n2, coef3, p3, false, a)?;
            pin(&n2)?.data().scal = 0.;
            add_flag(&n2, NMSK_OUT_L)?;
        }
        if ins {
            let (d1, d2) = (data(&n1)?, data(&n2)?);
            let ip3 = pid.add_node(
                &d1,
                &d2,
                &mut a.n,
                &mut other.n,
                coef3,
                p3.x(),
                p3.y(),
                p3.z(),
            )?;
            let n3 = node_h(&a.n, ip3)?;
            pid.update_links_split(
                ip1,
                ip2,
                ip3,
                &mut a.t,
                &mut other.t,
                &mut a.s,
                &mut other.s,
                &mut a.n,
                &mut other.n,
            )?;
            self.update_around_node(ip3, &n3, a)?;
            pin(&n3)?.data().scal = 0.;
            add_flag(&n3, NMSK_OUT_L)?;
        }
        Ok(true)
    }

    // The outline point at `coef3` between two nodes: on the surface at the
    // interpolated parameters, else on the chord, in the view frame.
    fn outline_point(
        &self,
        d1: &crate::hlr_algo::NodeData,
        d2: &crate::hlr_algo::NodeData,
        coef3: f64,
    ) -> Result<Xyz, Failure> {
        let (u3, v3) = self.uv_node(d1, d2, coef3);
        let mut p = if let Some(s) = self.g_surf.get() {
            Xyz::from(s.value(u3, v3)?)
        } else {
            // Simple averaging - this could be improved.
            let coef2 = 1.0 - coef3;
            coef2 * d1.point + coef3 * d2.point
        };
        self.tt.apply_xyz(&mut p, false);
        Ok(p)
    }

    /// CheckFrBackTriangles(List, PID): up to four times, each triangle turned to the
    /// back but not on the side has its farthest movable node pushed across the
    /// opposite side in the view plane; the segments ending at moved nodes follow.
    pub(super) fn check_fr_back_triangles(
        &self,
        list: &mut List<BiPoint>,
        pids: &[Handle<PolyInternalData>],
    ) -> Result<(), Failure> {
        let mut is_modif = true;
        let mut i_loop = 0;
        let mut shifts = [[0.; 2]; 3];
        while is_modif && i_loop < 4 {
            i_loop += 1;
            is_modif = false;
            let mut is_fr_back_in_list = false;
            for pid in pids.iter().skip(1) {
                let Some(pid) = pid.get() else {
                    continue;
                };
                let a = Arrays::of(pid);
                for tri_iter in 1..=pid.nb_t_data() {
                    let t = tri(&a.t, tri_iter)?;
                    if t.flags & SIDE != 0 || t.flags & FR_BACK == 0 {
                        continue;
                    }
                    is_modif = true;
                    let nodes = [
                        node_h(&a.n, t.node1)?,
                        node_h(&a.n, t.node2)?,
                        node_h(&a.n, t.node3)?,
                    ];
                    let moved = self.push_farthest(
                        &nodes,
                        [t.node1, t.node2, t.node3],
                        i_loop,
                        &mut shifts,
                        &a,
                    )?;
                    is_fr_back_in_list |= moved;
                }
            }
            if is_fr_back_in_list {
                self.follow_moved_nodes(list, pids)?;
                for pid in pids.iter().skip(1) {
                    let Some(pid) = pid.get() else {
                        continue;
                    };
                    let nodes = pid.pi_nod();
                    for i in 1..=pid.nb_pi_nod() {
                        pin(&node_h(&nodes, i)?)?.indices().flag &= !NMSK_MOVE;
                    }
                }
            }
        }
        Ok(())
    }

    // The shift of each movable node of a triangle across its opposite side (the
    // native `X1, Y1` ... keep their values between triangles), then the node with the
    // longest opposite side moved; true when one moved.
    fn push_farthest(
        &self,
        nodes: &[NodeH; 3],
        ids: [i32; 3],
        i_loop: i32,
        shifts: &mut [[f64; 2]; 3],
        a: &Arrays,
    ) -> Result<bool, Failure> {
        let pts = [
            data(&nodes[0])?.point,
            data(&nodes[1])?.point,
            data(&nodes[2])?.point,
        ];
        let mut dist = [0.; 3];
        for k in 0..3 {
            let flag = pin(&nodes[k])?.indices().flag;
            let movable = (flag & NMSK_EDGE == 0 || i_loop > 1)
                && (flag & NMSK_OUT_L == 0 || i_loop > 1)
                && flag & NMSK_VERT == 0;
            if !movable {
                continue;
            }
            let (b, c) = (pts[(k + 1) % 3], pts[(k + 2) % 3]);
            let (dx, dy) = (c.x() - b.x(), c.y() - b.y());
            let d = (dx * dx + dy * dy).sqrt();
            let (nx, ny) = (-dy / d, dx / d);
            let (dx, dy) = (pts[k].x() - b.x(), pts[k].y() - b.y());
            let mut dd = -(dx * nx + dy * ny);
            if dd < 0. {
                dd -= d * 0.01;
            } else {
                dd += d * 0.01;
            }
            shifts[k] = [nx * dd, ny * dd];
            dist[k] = d;
        }
        let pick = if dist[0] > dist[1] && dist[0] > dist[2] {
            0
        } else if dist[1] > dist[2] && dist[1] > dist[0] {
            1
        } else if dist[2] > dist[0] && dist[2] > dist[1] {
            2
        } else {
            return Ok(false);
        };
        let n = &nodes[pick];
        {
            let mut d = pin(n)?.data();
            let p = d.point;
            d.point = Xyz::new(p.x() + shifts[pick][0], p.y() + shifts[pick][1], p.z());
        }
        add_flag(n, NMSK_MOVE)?;
        self.update_around_node(ids[pick], n, a)?;
        Ok(true)
    }

    // The segment ends at moved nodes take their new points, and the node of the
    // other face at the same end follows in the view plane.
    fn follow_moved_nodes(
        &self,
        list: &mut List<BiPoint>,
        pids: &[Handle<PolyInternalData>],
    ) -> Result<(), Failure> {
        let (mut a1, mut a2) = (Arrays::none(), Arrays::none());
        let arrays = |i: i32| -> Result<Arrays, Failure> {
            let p = pids.get(i as usize).and_then(|h| h.get()).ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "HLRBRep_PolyAlgo::CheckFrBackTriangles: a face without internal data",
                )
            })?;
            Ok(Arrays::of(p))
        };
        for bp in list.iter_mut() {
            let ind = *bp.indices();
            if ind.face_conex1 != 0 {
                a1 = arrays(ind.face_conex1)?;
            }
            if ind.face_conex2 != 0 {
                a2 = if ind.face_conex1 == ind.face_conex2 {
                    Arrays {
                        t: a1.t.clone(),
                        s: a1.s.clone(),
                        n: a1.n.clone(),
                    }
                } else {
                    arrays(ind.face_conex2)?
                };
            }
            if ind.face_conex1 != 0 {
                self.follow_end(
                    bp,
                    (&a1, ind.face1_pt1),
                    (ind.face_conex2 != 0).then_some((&a2, ind.face2_pt1)),
                    true,
                )?;
                self.follow_end(
                    bp,
                    (&a1, ind.face1_pt2),
                    (ind.face_conex2 != 0).then_some((&a2, ind.face2_pt2)),
                    false,
                )?;
            }
            if ind.face_conex2 != 0 {
                a2.n = arrays(ind.face_conex2)?.n;
                self.follow_end(
                    bp,
                    (&a2, ind.face2_pt1),
                    (ind.face_conex1 != 0).then_some((&a1, ind.face1_pt1)),
                    true,
                )?;
                self.follow_end(
                    bp,
                    (&a2, ind.face2_pt2),
                    (ind.face_conex1 != 0).then_some((&a1, ind.face1_pt2)),
                    false,
                )?;
            }
        }
        Ok(())
    }

    // One end of a segment at a moved node `(arrays, index)`: the segment point and
    // its model point, and the node of the other face moved in the view plane.
    fn follow_end(
        &self,
        bp: &mut BiPoint,
        (a, i): (&Arrays, i32),
        other: Option<(&Arrays, i32)>,
        first: bool,
    ) -> Result<(), Failure> {
        let n = node_h(&a.n, i)?;
        if pin(&n)?.indices().flag & NMSK_MOVE == 0 {
            return Ok(());
        }
        let p = data(&n)?.point;
        let mut ti = p;
        self.ti.apply_xyz(&mut ti, false);
        let pts = bp.points_mut();
        if first {
            (pts.pnt1, pts.pnt_p1) = (ti, p);
        } else {
            (pts.pnt2, pts.pnt_p2) = (ti, p);
        }
        if let Some((oa, oi)) = other {
            let on = node_h(&oa.n, oi)?;
            {
                let mut d = pin(&on)?.data();
                let q = d.point;
                d.point = Xyz::new(p.x(), p.y(), q.z());
            }
            self.update_around_node(oi, &on, oa)?;
        }
        Ok(())
    }
}
