//! HLRBRep_PolyAlgo.cxx: the three Interpolation and the three MoveOrInsertPoint
//! members, which cut an edge segment where the outline crosses it.
use super::orient::{Arrays, NodeH, add_flag, data, node_h, pin};
use super::{NMSK_EDGE, NMSK_OUT_L, NMSK_VERT, PolyState};
use crate::hlr_algo::{BiPoint, BiPointFace, BiPointPoints, NodeArray, PolyInternalData};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Xyz;

/// The two ends of the current edge segment: in the view frame (`X1`...), in the
/// model (`XTI1`...) and their edge parameters.
pub(super) struct Ends {
    pub(super) p1: Xyz,
    pub(super) p2: Xyz,
    pub(super) ti1: Xyz,
    pub(super) ti2: Xyz,
    pub(super) u1: f64,
    pub(super) u2: f64,
}

/// One face along the edge: its index, internal data and arrays.
pub(super) struct Side {
    pub(super) i: i32,
    pub(super) pid: Handle<PolyInternalData>,
    pub(super) a: Arrays,
}

/// The two nodes of the face ending the segment, with their indices.
#[derive(Clone)]
pub(super) struct SegNodes {
    pub(super) n1: NodeH,
    pub(super) n2: NodeH,
    pub(super) ip1: i32,
    pub(super) ip2: i32,
}

/// A point where the outline cuts the segment (`X3`, `XTI3`, `coef3`, `U3`, `insP3`,
/// `mP3P1`).
#[derive(Clone, Copy, Default)]
pub(super) struct Cut {
    pub(super) p: Xyz,
    pub(super) ti: Xyz,
    pub(super) coef: f64,
    pub(super) u: f64,
    pub(super) ins: bool,
    pub(super) m_p1: bool,
}

fn null_pid() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "HLRBRep_PolyAlgo: a null internal data; expected a triangulated face",
    )
}

fn face(s: &Side, ip1: i32, ip2: i32) -> BiPointFace {
    BiPointFace {
        conex: s.i,
        pt1: ip1,
        pt2: ip2,
    }
}

fn points(ti1: Xyz, ti2: Xyz, p1: Xyz, p2: Xyz) -> BiPointPoints {
    BiPointPoints {
        pnt1: ti1,
        pnt2: ti2,
        pnt_p1: p1,
        pnt_p2: p2,
    }
}

// The segment between two points on one or two faces.
fn bipoint(
    e: i32,
    pts: BiPointPoints,
    f1: BiPointFace,
    f2: Option<BiPointFace>,
    flag: i32,
) -> BiPoint {
    match f2 {
        Some(f2) => BiPoint::with_faces_flags(pts, e, [f1, f2], flag),
        None => BiPoint::with_face_flags(pts, e, f1, flag),
    }
}

/// The edge parameter of a node on the edge `e` (`PCu1` or `PCu2`) set to `u`.
fn set_edge_parameter(n: &NodeH, e: i32, u: f64) -> Result<(), Failure> {
    let ind = *pin(n)?.indices();
    if ind.edg1 == e {
        pin(n)?.data().p_cu1 = u;
    } else if ind.edg2 == e {
        pin(n)?.data().p_cu2 = u;
    }
    Ok(())
}

// A node moved onto the outline: its point, edge parameter, zero scalar, outline flag.
fn moved_on_outline(n: &NodeH, e: i32, cut: &Cut) -> Result<(), Failure> {
    pin(n)?.data().point = cut.p;
    set_edge_parameter(n, e, cut.u)?;
    pin(n)?.data().scal = 0.;
    add_flag(n, NMSK_OUT_L)
}

// A node inserted on the edge: its edge, parameter, zero scalar and flags.
fn inserted_on_edge(n: &NodeH, e: i32, u: f64) -> Result<(), Failure> {
    pin(n)?.indices().edg1 = e;
    {
        let mut d = pin(n)?.data();
        d.p_cu1 = u;
        d.scal = 0.;
    }
    add_flag(n, NMSK_OUT_L)?;
    add_flag(n, NMSK_EDGE)
}

impl PolyState {
    /// Interpolation(U1, U2, Nod1, Nod2, X3...): where the scalar of the two nodes
    /// changes sign, the point of the edge curve at the interpolated parameter.
    pub(super) fn interpolate(
        &self,
        u1: f64,
        u2: f64,
        n1: &NodeH,
        n2: &NodeH,
    ) -> Result<Cut, Failure> {
        let (d1, d2) = (data(n1)?, data(n2)?);
        let Some((coef, m_p1)) = self.new_node(&d1, &d2) else {
            return Ok(Cut::default());
        };
        let u = u1 + (u2 - u1) * coef;
        let ti = Xyz::from(self.b_curv.value(u)?);
        let mut p = ti;
        self.t.apply_xyz(&mut p, false);
        Ok(Cut {
            p,
            ti,
            coef,
            u,
            ins: true,
            m_p1,
        })
    }

    /// Interpolation(List, ..., e, U1, U2, Nod11, Nod12, i1p1, i1p2, i1, pid1, ...): the
    /// segment of an edge on one face, cut where the outline crosses it.
    pub(super) fn interpolation_one(
        &self,
        list: &mut List<BiPoint>,
        ends: &mut Ends,
        e: i32,
        s1: &mut Side,
        k1: &SegNodes,
    ) -> Result<(), Failure> {
        let cut = self.interpolate(ends.u1, ends.u2, &k1.n1, &k1.n2)?;
        self.move_or_insert(list, ends, e, (s1, k1), None, cut, 0)
    }

    /// Interpolation(List, ..., e, U1, U2, rg, face 1, face 2): the segment of an edge
    /// between two faces (the flag of its regularity), cut where the outline of either
    /// face crosses it.
    #[allow(clippy::too_many_arguments)] // The native member takes the two faces.
    pub(super) fn interpolation_two(
        &self,
        list: &mut List<BiPoint>,
        ends: &mut Ends,
        e: i32,
        flag: i32,
        (s1, k1): (&mut Side, &SegNodes),
        (s2, k2): (&mut Side, &SegNodes),
    ) -> Result<(), Failure> {
        let c3 = self.interpolate(ends.u1, ends.u2, &k1.n1, &k1.n2)?;
        let c4 = self.interpolate(ends.u1, ends.u2, &k2.n1, &k2.n2)?;
        if !(c3.ins || c4.ins) {
            let pts = points(ends.ti1, ends.ti2, ends.p1, ends.p2);
            list.prepend(bipoint(
                e,
                pts,
                face(s1, k1.ip1, k1.ip2),
                Some(face(s2, k2.ip1, k2.ip2)),
                flag,
            ));
            return Ok(());
        }
        if !c4.ins {
            self.move_or_insert(list, ends, e, (s1, k1), Some((s2, k2)), c3, flag)
        } else if !c3.ins || (c4.coef - c3.coef).abs() < self.tol_sta {
            self.move_or_insert(list, ends, e, (s2, k2), Some((s1, k1)), c4, flag)
        } else if c4.coef < c3.coef {
            self.move_or_insert_two(list, ends, e, (s2, k2), (s1, k1), (c4, c3), flag)
        } else {
            self.move_or_insert_two(list, ends, e, (s1, k1), (s2, k2), (c3, c4), flag)
        }
    }

    // The first node of the segment moved to the cut (`P1 ---> P3`), or the second:
    // ChangeNode on each face (in reverse order with `swap`), the ends, then on each
    // face the node on the outline and the triangles around it.
    #[allow(clippy::too_many_arguments)] // The native blocks share these values.
    fn move_end(
        &self,
        ends: &mut Ends,
        e: i32,
        sides: [(&Side, &SegNodes); 2],
        n: usize,
        cut: &Cut,
        first: bool,
        swap: bool,
    ) -> Result<(), Failure> {
        let order: Vec<usize> = if swap {
            (0..n).rev().collect()
        } else {
            (0..n).collect()
        };
        for &i in &order {
            let (s, k) = sides[i];
            self.change_node((k.ip1, k.ip2), &k.n1, &k.n2, cut.coef, cut.p, first, &s.a)?;
        }
        if first {
            (ends.p1, ends.ti1, ends.u1) = (cut.p, cut.ti, cut.u);
        } else {
            (ends.p2, ends.ti2, ends.u2) = (cut.p, cut.ti, cut.u);
        }
        for (s, k) in sides.iter().take(n) {
            let (node, ip) = if first {
                (&k.n1, k.ip1)
            } else {
                (&k.n2, k.ip2)
            };
            moved_on_outline(node, e, cut)?;
            self.update_around_node(ip, node, &s.a)?;
        }
        Ok(())
    }

    /// MoveOrInsertPoint(List, ..., face 1 [, face 2], X3, ..., insP3, mP3P1, flag): the
    /// cut moves the nearer node of the segment when it is close to it (and the node is
    /// not a vertex), else a node is inserted on each face and the segment is split.
    #[allow(clippy::too_many_arguments)] // The native member takes these values.
    pub(super) fn move_or_insert(
        &self,
        list: &mut List<BiPoint>,
        ends: &mut Ends,
        e: i32,
        (s1, k1): (&mut Side, &SegNodes),
        s2: Option<(&mut Side, &SegNodes)>,
        cut: Cut,
        flag: i32,
    ) -> Result<(), Failure> {
        let mut ins3 = cut.ins;
        let mut dummy = Side {
            i: 0,
            pid: Handle::default(),
            a: Arrays::none(),
        };
        let empty = k1.clone();
        let (s2, k2, n) = match s2 {
            Some((s, k)) => (s, k, 2),
            None => (&mut dummy, &empty, 1),
        };
        if ins3
            && cut.m_p1
            && pin(&k1.n1)?.indices().flag & NMSK_VERT == 0
            && cut.coef < self.tol_sta
        {
            ins3 = false;
            self.move_end(ends, e, [(&*s1, k1), (&*s2, k2)], n, &cut, true, false)?;
            if let Some(first) = list.first_mut() {
                let p = first.points_mut();
                p.pnt_p2 = cut.p;
                p.pnt2 = cut.ti;
            }
        }
        if ins3
            && !cut.m_p1
            && pin(&k1.n2)?.indices().flag & NMSK_VERT == 0
            && cut.coef > self.tol_end
        {
            ins3 = false;
            self.move_end(ends, e, [(&*s1, k1), (&*s2, k2)], n, &cut, false, false)?;
        }
        let s2_face = s2.i;
        let f2 = |ip1, ip2| {
            (n == 2).then_some(BiPointFace {
                conex: s2_face,
                pt1: ip1,
                pt2: ip2,
            })
        };
        if !ins3 {
            let pts = points(ends.ti1, ends.ti2, ends.p1, ends.p2);
            list.prepend(bipoint(
                e,
                pts,
                face(s1, k1.ip1, k1.ip2),
                f2(k2.ip1, k2.ip2),
                flag,
            ));
            return Ok(());
        }
        let i1p3 = self.add_cut_node(s1, k1, &mut s2.a.n, &cut)?;
        let i2p3 = if n == 2 {
            self.add_cut_node(s2, k2, &mut s1.a.n, &cut)?
        } else {
            0
        };
        let n13 = node_h(&s1.a.n, i1p3)?;
        let n23 = if n == 2 {
            Some(node_h(&s2.a.n, i2p3)?)
        } else {
            None
        };
        inserted_on_edge(&n13, e, cut.u)?;
        if let Some(n23) = &n23 {
            inserted_on_edge(n23, e, cut.u)?;
        }
        split_links(s1, s2, (k1.ip1, k1.ip2, i1p3))?;
        if n == 2 {
            split_links(s2, s1, (k2.ip1, k2.ip2, i2p3))?;
        }
        self.update_around_node(i1p3, &n13, &s1.a)?;
        if let Some(n23) = &n23 {
            self.update_around_node(i2p3, n23, &s2.a)?;
        }
        let pts = points(ends.ti1, cut.ti, ends.p1, cut.p);
        list.prepend(bipoint(
            e,
            pts,
            face(s1, k1.ip1, i1p3),
            f2(k2.ip1, i2p3),
            flag,
        ));
        let pts = points(cut.ti, ends.ti2, cut.p, ends.p2);
        list.prepend(bipoint(
            e,
            pts,
            face(s1, i1p3, k1.ip2),
            f2(i2p3, k2.ip2),
            flag,
        ));
        Ok(())
    }

    // AddNode on the face of a cut (the node array of the other face follows it).
    fn add_cut_node(
        &self,
        s: &mut Side,
        k: &SegNodes,
        other_nodes: &mut NodeArray,
        cut: &Cut,
    ) -> Result<i32, Failure> {
        let (d1, d2) = (data(&k.n1)?, data(&k.n2)?);
        let pid = s.pid.get().ok_or_else(null_pid)?;
        pid.add_node(
            &d1,
            &d2,
            &mut s.a.n,
            other_nodes,
            cut.coef,
            cut.p.x(),
            cut.p.y(),
            cut.p.z(),
        )
    }

    /// MoveOrInsertPoint(List, ..., face 1, face 2, X3 ..., X4 ...): two cuts of the
    /// segment in this order (the first on face 1, the second on face 2): the near
    /// ends move, then one cut or both are inserted on both faces.
    #[allow(clippy::too_many_arguments)] // The native member takes these values.
    pub(super) fn move_or_insert_two(
        &self,
        list: &mut List<BiPoint>,
        ends: &mut Ends,
        e: i32,
        (s1, k1): (&mut Side, &SegNodes),
        (s2, k2): (&mut Side, &SegNodes),
        (c3, c4): (Cut, Cut),
        flag: i32,
    ) -> Result<(), Failure> {
        let (mut ins3, mut ins4) = (c3.ins, c4.ins);
        if ins3 && c3.m_p1 && pin(&k1.n1)?.indices().flag & NMSK_VERT == 0 && c3.coef < self.tol_sta
        {
            ins3 = false;
            self.move_end(ends, e, [(&*s1, k1), (&*s2, k2)], 2, &c3, true, false)?;
            if let Some(first) = list.first_mut() {
                let p = first.points_mut();
                p.pnt_p2 = c3.p;
                p.pnt2 = c3.ti;
            }
        }
        if ins4
            && !c4.m_p1
            && pin(&k1.n2)?.indices().flag & NMSK_VERT == 0
            && c4.coef > self.tol_end
        {
            ins4 = false;
            // ChangeNode on face 2 first, then the nodes of face 1 and face 2.
            self.move_end(ends, e, [(&*s1, k1), (&*s2, k2)], 2, &c4, false, true)?;
        }
        if !(ins3 || ins4) {
            let pts = points(ends.ti1, ends.ti2, ends.p1, ends.p2);
            list.prepend(bipoint(
                e,
                pts,
                face(s1, k1.ip1, k1.ip2),
                Some(face(s2, k2.ip1, k2.ip2)),
                flag,
            ));
            return Ok(());
        }
        if !ins4 {
            return self.move_or_insert(list, ends, e, (s1, k1), Some((s2, k2)), c3, flag);
        }
        if !ins3 {
            return self.move_or_insert(list, ends, e, (s2, k2), Some((s1, k1)), c4, flag);
        }
        self.insert_two_cuts(list, ends, e, (s1, k1), (s2, k2), (c3, c4), flag)
    }

    // Both cuts inserted on both faces, the segment split in three.
    #[allow(clippy::too_many_arguments)] // The native member takes these values.
    fn insert_two_cuts(
        &self,
        list: &mut List<BiPoint>,
        ends: &Ends,
        e: i32,
        (s1, k1): (&mut Side, &SegNodes),
        (s2, k2): (&mut Side, &SegNodes),
        (c3, c4): (Cut, Cut),
        flag: i32,
    ) -> Result<(), Failure> {
        let i1p3 = self.add_cut_node(s1, k1, &mut s2.a.n, &c3)?;
        let i2p3 = self.add_cut_node(s2, k2, &mut s1.a.n, &c3)?;
        let i1p4 = self.add_cut_node(s1, k1, &mut s2.a.n, &c4)?;
        let i2p4 = self.add_cut_node(s2, k2, &mut s1.a.n, &c4)?;
        let (n13, n14) = (node_h(&s1.a.n, i1p3)?, node_h(&s1.a.n, i1p4)?);
        let (n23, n24) = (node_h(&s2.a.n, i2p3)?, node_h(&s2.a.n, i2p4)?);
        inserted_on_edge(&n13, e, c3.u)?;
        inserted_on_edge(&n23, e, c3.u)?;
        inserted_on_edge(&n14, e, c4.u)?;
        inserted_on_edge(&n24, e, c4.u)?;
        split_links(s1, s2, (k1.ip1, k1.ip2, i1p3))?;
        split_links(s2, s1, (k2.ip1, k2.ip2, i2p3))?;
        split_links(s2, s1, (i2p3, k2.ip2, i2p4))?;
        split_links(s1, s2, (i1p3, k1.ip2, i1p4))?;
        self.update_around_node(i1p3, &n13, &s1.a)?;
        self.update_around_node(i2p3, &n23, &s2.a)?;
        self.update_around_node(i1p4, &n14, &s1.a)?;
        self.update_around_node(i2p4, &n24, &s2.a)?;
        let f = |s: &Side, a, b| face(s, a, b);
        list.prepend(bipoint(
            e,
            points(ends.ti1, c3.ti, ends.p1, c3.p),
            f(s1, k1.ip1, i1p3),
            Some(f(s2, k2.ip1, i2p3)),
            flag,
        ));
        list.prepend(bipoint(
            e,
            points(c3.ti, c4.ti, c3.p, c4.p),
            f(s1, i1p3, i1p4),
            Some(f(s2, i2p3, i2p4)),
            flag,
        ));
        list.prepend(bipoint(
            e,
            points(c4.ti, ends.ti2, c4.p, ends.p2),
            f(s1, i1p4, k1.ip2),
            Some(f(s2, i2p4, k2.ip2)),
            flag,
        ));
        Ok(())
    }
}

// UpdateLinks(ip1, ip2, ip3, ...) on the face `s`, the arrays of `other` following.
fn split_links(
    s: &mut Side,
    other: &mut Side,
    (ip1, ip2, ip3): (i32, i32, i32),
) -> Result<(), Failure> {
    let pid = s.pid.get().ok_or_else(null_pid)?;
    pid.update_links_split(
        ip1,
        ip2,
        ip3,
        &mut s.a.t,
        &mut other.a.t,
        &mut s.a.s,
        &mut other.a.s,
        &mut s.a.n,
        &mut other.a.n,
    )
}
