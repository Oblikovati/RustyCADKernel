//! HLRBRep_PolyAlgo.cxx: InitBiPointsWithConnexity, the segments of one edge.
use super::insert::{Ends, SegNodes, Side};
use super::orient::{Arrays, NodeH, add_flag, data, node_h, pin};
use super::{Frame, NMSK_EDGE, NMSK_VERT, PolyState};
use crate::hlr_algo::{BiPoint, BiPointPoints, PolyInternalData};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Xyz;
use rustycad_tkmath::poly::{PolygonOnTriangulation, mesh_purpose};

// The polygon of the edge on the triangulation of the face.
fn polygon_on(edge: &Edge, face: &Face) -> Result<Handle<PolygonOnTriangulation>, Failure> {
    let (tr, loc) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
    BRepTool::polygon_on_triangulation(edge, &tr, &loc)
}

fn side(i: i32, pid: &[Handle<PolyInternalData>]) -> Result<Side, Failure> {
    let h = pid.get(i as usize).cloned().unwrap_or_default();
    let p = h.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "HLRBRep_PolyAlgo::InitBiPointsWithConnexity: a face without internal data",
        )
    })?;
    let a = Arrays::of(p);
    Ok(Side { i, pid: h, a })
}

// The edge parameter of a node: `PCu1` or `PCu2` for the edge, else `u`.
fn edge_parameter(n: &NodeH, e: i32, u: f64) -> Result<f64, Failure> {
    let ind = *pin(n)?.indices();
    let d = data(n)?;
    Ok(if ind.edg1 == e {
        d.p_cu1
    } else if ind.edg2 == e {
        d.p_cu2
    } else {
        u
    })
}

// Records the edge and its parameter on a node of the polygon.
fn set_edge(n: &NodeH, e: i32, pcu: f64) -> Result<(), Failure> {
    let edg1 = pin(n)?.indices().edg1;
    if edg1 == 0 || edg1 == e {
        pin(n)?.indices().edg1 = e;
        pin(n)?.data().p_cu1 = pcu;
    } else {
        pin(n)?.indices().edg2 = e;
        pin(n)?.data().p_cu2 = pcu;
    }
    Ok(())
}

impl PolyState {
    // The next end of the segment: the node point, its model point and parameter.
    fn advance(&self, ends: &mut Ends, n12: &NodeH, e: i32) -> Result<(), Failure> {
        let p = data(n12)?.point;
        ends.p2 = p;
        ends.ti2 = p;
        ends.u2 = edge_parameter(n12, e, ends.u2)?;
        add_flag(n12, NMSK_EDGE)?;
        self.ti.apply_xyz(&mut ends.ti2, false);
        Ok(())
    }

    fn shift(ends: &mut Ends) {
        ends.p1 = ends.p2;
        ends.ti1 = ends.ti2;
        ends.u1 = ends.u2;
    }

    /// InitBiPointsWithConnexity(e, Edge, List, PID, LS, connex): the segments of the
    /// edge `e`: along its polygon on the triangulation of its face (or of its two
    /// faces), cut where the outlines cross it; along its 3D polygon when it has no
    /// face.
    pub(super) fn init_bipoints_with_connexity(
        &mut self,
        i_edge: i32,
        edge: &mut Edge,
        list: &mut List<BiPoint>,
        pid: &[Handle<PolyInternalData>],
        ls: &List<Shape>,
        is_connex: bool,
    ) -> Result<(), Failure> {
        self.b_curv.initialize(edge)?;
        if !is_connex {
            return self.isolated_edge(i_edge, edge, list);
        }
        let faces: Vec<Shape> = ls.iter().cloned().collect();
        match faces.len() {
            1 => self.edge_on_one_face(i_edge, edge, list, pid, &topods::face(&faces[0])?),
            2 => self.edge_on_two_faces(
                i_edge,
                edge,
                list,
                pid,
                (&topods::face(&faces[0])?, &topods::face(&faces[1])?),
            ),
            _ => Ok(()),
        }
    }

    fn edge_on_one_face(
        &self,
        e: i32,
        edge: &Edge,
        list: &mut List<BiPoint>,
        pid: &[Handle<PolyInternalData>],
        f1: &Face,
    ) -> Result<(), Failure> {
        let i1 = self.f_map.find_index(&Shape::from(f1.clone())) as i32;
        let hpol = polygon_on(edge, f1)?;
        let Some(pol) = hpol.get() else {
            return Ok(());
        };
        let mut s1 = side(i1, pid)?;
        self.pc.initialize(edge, f1)?;
        let nodes = pol.nodes();
        let par = pol.parameters();
        let nb_pol = nodes.upper();
        let polv = |k: i32| {
            nodes
                .value(k)
                .copied()
                .map_err(|x| x.into_failure("HLRBRep_PolyAlgo::InitBiPointsWithConnexity"))
        };
        let parv = |k: i32| -> Result<f64, Failure> {
            let a = par.get().ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "HLRBRep_PolyAlgo: a polygon without parameters",
                )
            })?;
            Ok(*a
                .array1()
                .value(k)
                .map_err(|x| x.into_failure("HLRBRep_PolyAlgo::InitBiPointsWithConnexity"))?)
        };
        let n11 = node_h(&s1.a.n, polv(1)?)?;
        let n12_end = node_h(&s1.a.n, polv(nb_pol)?)?;
        add_flag(&n11, NMSK_VERT)?;
        add_flag(&n12_end, NMSK_VERT)?;
        for ipol in 1..=nb_pol {
            set_edge(&node_h(&s1.a.n, polv(ipol)?)?, e, parv(ipol)?)?;
        }
        let mut i1p2 = polv(1)?;
        let mut n11 = n11;
        let mut n12 = n11.clone();
        let mut ends = Ends {
            p1: Xyz::default(),
            p2: Xyz::default(),
            ti1: Xyz::default(),
            ti2: Xyz::default(),
            u1: 0.,
            u2: 0.,
        };
        self.advance(&mut ends, &n12, e)?;
        if polv(1)? == polv(nb_pol)? && self.pc.is_periodic()? {
            ends.u2 -= self.pc.period()?;
        }
        if nb_pol == 2 && BRepTool::degenerated(edge)? {
            self.check_degenerated_segment(&n11, &n12)?;
            self.update_around_node(polv(1)?, &n11, &s1.a)?;
            return self.update_around_node(polv(nb_pol)?, &n12, &s1.a);
        }
        for ipol in 2..=nb_pol {
            let i1p1 = i1p2;
            n11 = n12;
            i1p2 = polv(ipol)?;
            n12 = node_h(&s1.a.n, i1p2)?;
            Self::shift(&mut ends);
            self.advance(&mut ends, &n12, e)?;
            let k1 = SegNodes {
                n1: n11.clone(),
                n2: n12.clone(),
                ip1: i1p1,
                ip2: i1p2,
            };
            self.interpolation_one(list, &mut ends, e, &mut s1, &k1)?;
        }
        Ok(())
    }

    fn edge_on_two_faces(
        &self,
        e: i32,
        edge: &mut Edge,
        list: &mut List<BiPoint>,
        pid: &[Handle<PolyInternalData>],
        (f1, f2): (&Face, &Face),
    ) -> Result<(), Failure> {
        let i1 = self.f_map.find_index(&Shape::from(f1.clone())) as i32;
        let hpol0 = polygon_on(edge, f1)?;
        let i2 = self.f_map.find_index(&Shape::from(f2.clone())) as i32;
        if i1 == i2 {
            edge.reverse();
        }
        let hpol1 = polygon_on(edge, f2)?;
        let rg = BRepTool::continuity_on_faces(edge, f1, f2)?;
        let (Some(pol0), Some(pol1)) = (hpol0.get(), hpol1.get()) else {
            return Ok(());
        };
        let (mut s1, mut s2) = (side(i1, pid)?, side(i2, pid)?);
        self.pc.initialize(edge, f1)?;
        let (nodes1, nodes2, par) = (pol0.nodes(), pol1.nodes(), pol0.parameters());
        let nb_pol1 = nodes1.upper();
        let member = "HLRBRep_PolyAlgo::InitBiPointsWithConnexity";
        let pol1v = |k: i32| nodes1.value(k).copied().map_err(|x| x.into_failure(member));
        let pol2v = |k: i32| nodes2.value(k).copied().map_err(|x| x.into_failure(member));
        let parv = |k: i32| -> Result<f64, Failure> {
            let a = par.get().ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "HLRBRep_PolyAlgo: a polygon without parameters",
                )
            })?;
            Ok(*a.array1().value(k).map_err(|x| x.into_failure(member))?)
        };
        let n11 = node_h(&s1.a.n, pol1v(1)?)?;
        let n12_end = node_h(&s1.a.n, pol1v(nb_pol1)?)?;
        let n21 = node_h(&s2.a.n, pol2v(1)?)?;
        // The native code reads the last node of the second polygon at the size of
        // the first one.
        let n22_end = node_h(&s2.a.n, pol2v(nb_pol1)?)?;
        for n in [&n11, &n12_end, &n21, &n22_end] {
            add_flag(n, NMSK_VERT)?;
        }
        for ipol in 1..=nb_pol1 {
            let n1a = node_h(&s1.a.n, pol1v(ipol)?)?;
            let n2a = node_h(&s2.a.n, pol2v(ipol)?)?;
            let pcu = parv(ipol)?;
            set_edge(&n1a, e, pcu)?;
            set_edge(&n2a, e, pcu)?;
        }
        let (mut i1p2, mut i2p2) = (pol1v(1)?, pol2v(1)?);
        let (mut n11, mut n21) = (n11, n21);
        let (mut n12, mut n22) = (n11.clone(), n21.clone());
        let mut ends = Ends {
            p1: Xyz::default(),
            p2: Xyz::default(),
            ti1: Xyz::default(),
            ti2: Xyz::default(),
            u1: 0.,
            u2: 0.,
        };
        let p = data(&n12)?.point;
        (ends.p2, ends.ti2) = (p, p);
        ends.u2 = edge_parameter(&n12, e, ends.u2)?;
        add_flag(&n12, NMSK_EDGE)?;
        add_flag(&n22, NMSK_EDGE)?;
        self.ti.apply_xyz(&mut ends.ti2, false);
        if pol1v(1)? == pol1v(nb_pol1)? && self.pc.is_periodic()? {
            ends.u2 -= self.pc.period()?;
        }
        if nb_pol1 == 2 && BRepTool::degenerated(edge)? {
            self.check_degenerated_segment(&n11, &n12)?;
            self.check_degenerated_segment(&n21, &n22)?;
            self.update_around_node(pol1v(1)?, &n11, &s1.a)?;
            self.update_around_node(pol1v(nb_pol1)?, &n12, &s1.a)?;
            self.update_around_node(pol2v(1)?, &n21, &s2.a)?;
            return self.update_around_node(pol2v(nb_pol1)?, &n22, &s2.a);
        }
        let flag = i32::from(rg >= Continuity::G1) + if rg >= Continuity::G2 { 2 } else { 0 };
        for ipol in 2..=nb_pol1 {
            let (i1p1, i2p1) = (i1p2, i2p2);
            n11 = n12;
            n21 = n22;
            i1p2 = pol1v(ipol)?;
            n12 = node_h(&s1.a.n, i1p2)?;
            i2p2 = pol2v(ipol)?;
            n22 = node_h(&s2.a.n, i2p2)?;
            Self::shift(&mut ends);
            // The native loop flags the node of face 1 twice and not the one of face 2.
            self.advance(&mut ends, &n12, e)?;
            let k1 = SegNodes {
                n1: n11.clone(),
                n2: n12.clone(),
                ip1: i1p1,
                ip2: i1p2,
            };
            let k2 = SegNodes {
                n1: n21.clone(),
                n2: n22.clone(),
                ip1: i2p1,
                ip2: i2p2,
            };
            self.interpolation_two(list, &mut ends, e, flag, (&mut s1, &k1), (&mut s2, &k2))?;
        }
        Ok(())
    }

    // An edge without face: the segments of its 3D polygon.
    fn isolated_edge(
        &mut self,
        e: i32,
        edge: &Edge,
        list: &mut List<BiPoint>,
    ) -> Result<(), Failure> {
        let (polyg, loc) = BRepTool::polygon3d(edge)?;
        let Some(polyg) = polyg.get() else {
            return Ok(());
        };
        let pol = polyg.nodes();
        let mut tt = loc.transformation();
        tt.pre_multiply(self.proj.transformation());
        self.tt = Frame::of(&tt)?;
        let member = "HLRBRep_PolyAlgo::InitBiPointsWithConnexity";
        let mut p2 = Xyz::from(*pol.value(1).map_err(|x| x.into_failure(member))?);
        self.tt.apply_xyz(&mut p2, false);
        let mut ti2 = p2;
        self.ti.apply_xyz(&mut ti2, false);
        for jpol in 2..=pol.upper() {
            let (p1, ti1) = (p2, ti2);
            p2 = Xyz::from(*pol.value(jpol).map_err(|x| x.into_failure(member))?);
            self.tt.apply_xyz(&mut p2, false);
            ti2 = p2;
            self.ti.apply_xyz(&mut ti2, false);
            let pts = BiPointPoints {
                pnt1: ti1,
                pnt2: ti2,
                pnt_p1: p1,
                pnt_p2: p2,
            };
            list.prepend(BiPoint::with_flags(pts, e, 0));
        }
        Ok(())
    }
}
