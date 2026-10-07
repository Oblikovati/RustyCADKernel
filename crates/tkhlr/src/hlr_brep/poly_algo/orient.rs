//! HLRBRep_PolyAlgo.cxx: the node and triangle helpers (UpdateAroundNode,
//! OrientTriangle, ChangeNode, Triangles, NewNode, UVNode, CheckDegeneratedSegment).
//!
//! The native members take references to the indices and data of nodes, which may
//! be one node; the port takes the node handles and reads or writes each field
//! through a short lock in the native order, so a write through one name is seen
//! through the other.
use super::{NMSK_FUCK, NMSK_OUT_L, PolyState};
use crate::hlr_algo::{
    NodeArray, NodeData, PolyInternalData, PolyInternalNode, PolyInternalSegment, PolyMask,
    SegmentArray, TriangleArray, TriangleData,
};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Xy, Xyz};

pub(super) type NodeH = Handle<PolyInternalNode>;

const SIDE: i32 = PolyMask::FMskSide as i32;
const BACK: i32 = PolyMask::FMskBack as i32;
const FLAT: i32 = PolyMask::FMskFlat as i32;
const ON_OUT_L: i32 = PolyMask::FMskOnOutL as i32;
const OR_BACK: i32 = PolyMask::FMskOrBack as i32;
const FR_BACK: i32 = PolyMask::FMskFrBack as i32;

fn null(what: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!(
            "HLRBRep_PolyAlgo: a null {what}; expected the internal data of a triangulated face"
        ),
    )
}

fn array<T: Send + Sync + 'static>(h: &Handle<HArray1<T>>) -> Result<&HArray1<T>, Failure> {
    h.get().ok_or_else(|| null("array"))
}

fn range(member: &'static str) -> impl Fn(rustycad_tkernel::ncollection::ArrayError) -> Failure {
    move |e| e.into_failure(member)
}

/// The node behind a handle.
pub(super) fn pin(h: &NodeH) -> Result<&PolyInternalNode, Failure> {
    h.get().ok_or_else(|| null("node"))
}

/// The node `i` of the node array.
pub(super) fn node_h(nodes: &NodeArray, i: i32) -> Result<NodeH, Failure> {
    let n = array(nodes)?
        .array1()
        .value(i)
        .map_err(range("HLRBRep_PolyAlgo"))?
        .clone();
    if n.is_null() {
        return Err(null("node"));
    }
    Ok(n)
}

/// A copy of the data of a node.
pub(super) fn data(h: &NodeH) -> Result<NodeData, Failure> {
    Ok(*pin(h)?.data())
}

/// The flags of a node.
pub(super) fn flag(h: &NodeH) -> Result<i32, Failure> {
    Ok(pin(h)?.indices().flag)
}

/// Adds the bits to the flags of a node.
pub(super) fn add_flag(h: &NodeH, bits: i32) -> Result<(), Failure> {
    pin(h)?.indices().flag |= bits;
    Ok(())
}

/// A copy of the triangle `i`.
pub(super) fn tri(t: &TriangleArray, i: i32) -> Result<TriangleData, Failure> {
    Ok(*array(t)?
        .array1()
        .value(i)
        .map_err(range("HLRBRep_PolyAlgo"))?)
}

/// Writes the triangle `i`.
pub(super) fn set_tri(t: &TriangleArray, i: i32, td: TriangleData) -> Result<(), Failure> {
    *array(t)?
        .change_array1()
        .change_value(i)
        .map_err(range("HLRBRep_PolyAlgo"))? = td;
    Ok(())
}

/// A copy of the segment `i`.
pub(super) fn seg(s: &SegmentArray, i: i32) -> Result<PolyInternalSegment, Failure> {
    Ok(*array(s)?
        .array1()
        .value(i)
        .map_err(range("HLRBRep_PolyAlgo"))?)
}

/// The triangle, segment and node arrays of a face (`TData`, `PISeg`, `PINod`), the
/// native array pointers that the growing members redirect.
pub(super) struct Arrays {
    pub(super) t: TriangleArray,
    pub(super) s: SegmentArray,
    pub(super) n: NodeArray,
}

impl Arrays {
    pub(super) fn of(pid: &PolyInternalData) -> Self {
        Self {
            t: pid.t_data(),
            s: pid.pi_seg(),
            n: pid.pi_nod(),
        }
    }

    /// No arrays (the native null pointers).
    pub(super) fn none() -> Self {
        Self {
            t: Handle::default(),
            s: Handle::default(),
            n: Handle::default(),
        }
    }
}

pub(super) fn lerp_xy(c2: f64, a: Xy, c1: f64, b: Xy) -> Xy {
    c2 * a + c1 * b
}

impl PolyState {
    /// UpdateAroundNode(iNode, Nod1Indices, TData, PISeg, PINod): orients the
    /// triangles of the segments around the node.
    pub(super) fn update_around_node(
        &self,
        i_node: i32,
        nd: &NodeH,
        a: &Arrays,
    ) -> Result<(), Failure> {
        let mut iiii = pin(nd)?.indices().nd_sg;
        while iiii != 0 {
            let sg = seg(&a.s, iiii)?;
            if sg.conex1 != 0 {
                self.orient_triangle_at(sg.conex1, a)?;
            }
            if sg.conex2 != 0 {
                self.orient_triangle_at(sg.conex2, a)?;
            }
            iiii = if sg.lst_sg1 == i_node {
                sg.nxt_sg1
            } else {
                sg.nxt_sg2
            };
        }
        Ok(())
    }

    pub(super) fn orient_triangle_at(&self, i_tri: i32, a: &Arrays) -> Result<(), Failure> {
        let mut td = tri(&a.t, i_tri)?;
        let nodes = [
            node_h(&a.n, td.node1)?,
            node_h(&a.n, td.node2)?,
            node_h(&a.n, td.node3)?,
        ];
        self.orient_triangle(&mut td, &nodes)?;
        let mut arr = array(&a.t)?.change_array1();
        arr.change_value(i_tri)
            .map_err(range("HLRBRep_PolyAlgo::OrientTriangle"))?
            .flags = td.flags;
        Ok(())
    }

    /// OrientTriangle(iTri, Triangle, Nod1..3): the side, back, flat and orientation
    /// flags of a triangle from the outline flags and scalars of its nodes and its
    /// orientation in the view.
    pub(super) fn orient_triangle(
        &self,
        t: &mut TriangleData,
        nodes: &[NodeH; 3],
    ) -> Result<(), Failure> {
        let o = [
            flag(&nodes[0])? & NMSK_OUT_L != 0,
            flag(&nodes[1])? & NMSK_OUT_L != 0,
            flag(&nodes[2])? & NMSK_OUT_L != 0,
        ];
        let d = [data(&nodes[0])?, data(&nodes[1])?, data(&nodes[2])?];
        t.flags &= !FLAT;
        t.flags &= !ON_OUT_L;
        if o[0] && o[1] && o[2] {
            t.flags |= SIDE;
            t.flags &= !BACK;
            t.flags |= ON_OUT_L;
        } else {
            let (mut s, mut as_) = (0., 0.);
            for k in 0..3 {
                let sk = d[k].scal;
                let ask = if sk < 0. { -sk } else { sk };
                if !o[k] && (k == 0 || as_ < ask) {
                    s = sk;
                    as_ = ask;
                }
            }
            t.flags &= !SIDE;
            if s > 0. {
                t.flags |= BACK;
            } else {
                t.flags &= !BACK;
            }
            self.orient_by_view(t, &d)?;
        }
        if (t.flags & BACK == 0 && t.flags & OR_BACK != 0)
            || (t.flags & BACK != 0 && t.flags & OR_BACK == 0)
        {
            t.flags |= FR_BACK;
        } else {
            t.flags &= !FR_BACK;
        }
        Ok(())
    }

    // The flat test and the orientation of the triangle plane to the eye.
    fn orient_by_view(&self, t: &mut TriangleData, d: &[NodeData; 3]) -> Result<(), Failure> {
        let flat = |t: &mut TriangleData| {
            t.flags |= FLAT;
            t.flags |= SIDE;
            t.flags &= !BACK;
        };
        let mut d12 = d[1].point - d[0].point;
        let d12_norm = d12.modulus();
        if d12_norm <= 1.0e-10 {
            flat(t);
            return Ok(());
        }
        let mut d23 = d[2].point - d[1].point;
        let d23_norm = d23.modulus();
        if d23_norm < 1.0e-10 {
            flat(t);
            return Ok(());
        }
        let d31_norm = (d[0].point - d[2].point).modulus();
        if d31_norm < 1.0e-10 {
            flat(t);
            return Ok(());
        }
        d12 *= 1. / d12_norm;
        d23 *= 1. / d23_norm;
        let mut dd = d12 ^ d23;
        let d_norm = dd.modulus();
        if d_norm < 1.0e-5 {
            flat(t);
            return Ok(());
        }
        let mut o = if self.proj.perspective() {
            dd *= 1. / d_norm;
            dd.z() * self.proj.focus()? - dd * d[0].point
        } else {
            dd.z() / d_norm
        };
        if o < 0. {
            t.flags |= OR_BACK;
            o = -o;
        } else {
            t.flags &= !OR_BACK;
        }
        if o < 1.0e-10 {
            t.flags |= SIDE;
            t.flags &= !BACK;
        }
        Ok(())
    }

    /// ChangeNode(ip1, ip2, Nod1, Nod2, coef1, X3, Y3, Z3, first, ...): moves the first
    /// (or the second) node of a segment to the point at `coef1` with interpolated
    /// parameters, scalar and normal, then orients the triangles around it.
    #[allow(clippy::too_many_arguments)] // The native member takes these values.
    pub(super) fn change_node(
        &self,
        ip: (i32, i32),
        n1: &NodeH,
        n2: &NodeH,
        coef1: f64,
        p3: Xyz,
        is_first: bool,
        a: &Arrays,
    ) -> Result<(), Failure> {
        let coef2 = 1.0 - coef1;
        let (moved, i_moved) = if is_first { (n1, ip.0) } else { (n2, ip.1) };
        pin(moved)?.data().point = p3;
        let (d1, d2) = (data(n1)?, data(n2)?);
        let uv = lerp_xy(coef2, d1.uv, coef1, d2.uv);
        pin(moved)?.data().uv = uv;
        let (d1, d2) = (data(n1)?, data(n2)?);
        let scal = d1.scal * coef2 + d2.scal * coef1;
        pin(moved)?.data().scal = scal;
        let (d1, d2) = (data(n1)?, data(n2)?);
        let xyz = coef2 * d1.normal + coef1 * d2.normal;
        let norm = xyz.modulus();
        let normal = if norm > 0. {
            (1. / norm) * xyz
        } else {
            Xyz::new(1., 0., 0.)
        };
        pin(moved)?.data().normal = normal;
        self.update_around_node(i_moved, moved, a)
    }

    /// Triangles(ip1, ip2, Nod1Indices, PISeg, iTri1, iTri2): the two triangles of the
    /// segment `(ip1, ip2)` around the node `ip1`.
    pub(super) fn triangles(
        &self,
        ip1: i32,
        ip2: i32,
        nd1: &NodeH,
        s: &SegmentArray,
    ) -> Result<(bool, i32, i32), Failure> {
        let mut iiii = pin(nd1)?.indices().nd_sg;
        while iiii != 0 {
            let sg = seg(s, iiii)?;
            if sg.lst_sg1 == ip1 {
                if sg.lst_sg2 == ip2 {
                    return Ok((true, sg.conex1, sg.conex2));
                }
                iiii = sg.nxt_sg1;
            } else {
                if sg.lst_sg1 == ip2 {
                    return Ok((true, sg.conex1, sg.conex2));
                }
                iiii = sg.nxt_sg2;
            }
        }
        Ok((false, 0, 0))
    }

    /// NewNode(Nod1, Nod2, coef1, moveP1): the zero of the scalar between two nodes of
    /// opposite scalars beyond half the angular tolerance: its coefficient and whether
    /// it is nearer the first node.
    pub(super) fn new_node(&self, d1: &NodeData, d2: &NodeData) -> Option<(f64, bool)> {
        let tol_ang = self.tol_angular * 0.5;
        if (d1.scal >= tol_ang && d2.scal <= -tol_ang)
            || (d2.scal >= tol_ang && d1.scal <= -tol_ang)
        {
            let mut coef1 = d1.scal / (d2.scal - d1.scal);
            if coef1 < 0. {
                coef1 = -coef1;
            }
            return Some((coef1, coef1 < 0.5));
        }
        None
    }

    /// UVNode(Nod1, Nod2, coef1, U3, V3): the interpolated surface parameters.
    pub(super) fn uv_node(&self, d1: &NodeData, d2: &NodeData, coef1: f64) -> (f64, f64) {
        let coef2 = 1.0 - coef1;
        let uv3 = lerp_xy(coef2, d1.uv, coef1, d2.uv);
        (uv3.x(), uv3.y())
    }

    /// CheckDegeneratedSegment(Nod1, Nod2): marks the nodes of a degenerated segment,
    /// on the outline when their scalars have opposite signs.
    pub(super) fn check_degenerated_segment(&self, n1: &NodeH, n2: &NodeH) -> Result<(), Failure> {
        add_flag(n1, NMSK_FUCK)?;
        add_flag(n2, NMSK_FUCK)?;
        let (s1, s2) = (data(n1)?.scal, data(n2)?.scal);
        let tol = self.tol_angular;
        if (s1 >= tol && s2 <= -tol) || (s2 >= tol && s1 <= -tol) {
            pin(n1)?.data().scal = 0.;
            add_flag(n1, NMSK_OUT_L)?;
            pin(n2)?.data().scal = 0.;
            add_flag(n2, NMSK_OUT_L)?;
        }
        Ok(())
    }
}
