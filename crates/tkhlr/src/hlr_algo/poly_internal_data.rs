// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_PolyInternalData.hxx/.lxx/.cxx (the class, AddNode, the Inc/Dec members and Dump) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_PolyInternalData: the growable internal triangulation of a face.
use super::failures::{array_failure, null_handle};
use super::poly_internal_node::{NodeData, PolyInternalNode};
use super::poly_records::{PolyInternalSegment, TriangleData};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::storage::format_general;
use rustycad_tkmath::gp::{Xy, Xyz};
use std::fmt;

/// The shared triangle array of an internal triangulation.
pub type TriangleArray = Handle<HArray1<TriangleData>>;
/// The shared segment array of an internal triangulation.
pub type SegmentArray = Handle<HArray1<PolyInternalSegment>>;
/// The shared node array of an internal triangulation.
pub type NodeArray = Handle<HArray1<Handle<PolyInternalNode>>>;

pub(super) struct PolyInternalState {
    pub(super) nb_t_data: i32,
    pub(super) nb_pi_seg: i32,
    pub(super) nb_pi_nod: i32,
    mx_t_data: i32,
    mx_pi_seg: i32,
    mx_pi_nod: i32,
    int_out_l: bool,
    planar: bool,
    t_data: TriangleArray,
    pi_seg: SegmentArray,
    pi_nod: NodeArray,
}

/// HLRAlgo_PolyInternalData: the triangles, the segments and the nodes of the internal
/// triangulation of a face, kept in shared arrays indexed from 0 that double when they
/// are full. The counts say how many entries are used (from index 1).
///
/// The native members that grow the arrays take pointers to array pointers and
/// redirect them to the new arrays; the port takes `&mut` handles to the shared
/// arrays, compared by identity (docs/hlr-design.md section 5). The array entries
/// that the native code leaves uninitialized are zero here.
///
/// ```
/// use rustycad_tkhlr::hlr_algo::PolyInternalData;
/// let data = PolyInternalData::new(3, 1).unwrap();
/// assert_eq!((data.nb_pi_nod(), data.nb_t_data(), data.nb_pi_seg()), (3, 1, 0));
/// let (mut t1, mut t2) = (data.t_data(), data.t_data());
/// data.inc_t_data(&mut t1, &mut t2).unwrap();
/// assert_eq!(data.nb_t_data(), 2);
/// assert_eq!(t1.get().unwrap().array1().upper(), 2);
/// ```
pub struct PolyInternalData {
    base: StandardTransient,
    pub(super) state: SharedState<PolyInternalState>,
}
impl_transient!(
    PolyInternalData,
    "HLRAlgo_PolyInternalData",
    StandardTransient,
    base
);

const CREATE: &str = "HLRAlgo_PolyInternalData::HLRAlgo_PolyInternalData";

/// A shared array `0..=upper` of default entries; Standard_RangeError below -1.
fn shared_array<T: Default + Send + Sync + 'static>(
    upper: i32,
    member: &'static str,
) -> Result<Handle<HArray1<T>>, Failure> {
    let array = HArray1::with_bounds(0, upper).map_err(array_failure(member))?;
    Ok(Handle::new(array))
}

/// The array behind a handle, or Standard_NullObject (a null pointer natively).
pub(super) fn array_of<'h, T: Send + Sync + 'static>(
    handle: &'h Handle<HArray1<T>>,
    member: &str,
) -> Result<&'h HArray1<T>, Failure> {
    handle.get().ok_or_else(|| null_handle(member, "the array"))
}

impl PolyInternalData {
    /// HLRAlgo_PolyInternalData(nbNod, nbTri): room for `nb_tri` triangles, `nb_nod`
    /// nodes (all created) and `2 + (3 nb_tri + nb_nod) / 2` segments, with `nb_tri`
    /// triangles and `nb_nod` nodes in use. The native constructor reads node 1 before
    /// its loop, so fewer than one node raises Standard_OutOfRange;
    /// `PolyInternalData::new(4, 2)?`.
    pub fn new(nb_nod: i32, nb_tri: i32) -> Result<Self, Failure> {
        let mx_pi_seg = 2i32.wrapping_add(nb_tri.wrapping_mul(3).wrapping_add(nb_nod) / 2);
        let t_data = shared_array(nb_tri, CREATE)?;
        let pi_seg = shared_array(mx_pi_seg, CREATE)?;
        let pi_nod: NodeArray = shared_array(nb_nod, CREATE)?;
        fill_nodes(&pi_nod, nb_nod)?;
        let state = PolyInternalState {
            nb_t_data: nb_tri,
            nb_pi_seg: 0,
            nb_pi_nod: nb_nod,
            mx_t_data: nb_tri,
            mx_pi_seg,
            mx_pi_nod: nb_nod,
            int_out_l: false,
            planar: false,
            t_data,
            pi_seg,
            pi_nod,
        };
        Ok(Self {
            base: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }

    /// AddNode: appends a node at `(x3, y3, z3)` between two nodes at `coef1` (its
    /// parameters, scalar and normalized normal interpolated; a null normal becomes
    /// `(1, 0, 0)`), growing the node arrays through `pi_nod1`/`pi_nod2` as
    /// `inc_pi_nod` does, and stores it in `pi_nod1`; returns its index;
    /// `data.add_node(&a, &b, &mut n1, &mut n2, 0.5, x, y, z)?`.
    #[allow(clippy::too_many_arguments)] // The native member takes these 8 values.
    pub fn add_node(
        &self,
        nod1: &NodeData,
        nod2: &NodeData,
        pi_nod1: &mut NodeArray,
        pi_nod2: &mut NodeArray,
        coef1: f64,
        x3: f64,
        y3: f64,
        z3: f64,
    ) -> Result<i32, Failure> {
        let coef2 = 1. - coef1;
        self.inc_pi_nod(pi_nod1, pi_nod2)?;
        let ip3 = self.state.read().nb_pi_nod;
        let node = PolyInternalNode::new();
        *node.data() = interpolated_node(nod1, nod2, coef1, coef2, Xyz::new(x3, y3, z3));
        let nodes = array_of(pi_nod1, "HLRAlgo_PolyInternalData::AddNode")?;
        let mut nodes = nodes.change_array1();
        let slot = nodes.change_value(ip3);
        *slot.map_err(array_failure("HLRAlgo_PolyInternalData::AddNode"))? = Handle::new(node);
        Ok(ip3)
    }

    /// IncTData: one more triangle; a full array doubles (entries 1..old size copied)
    /// and `t_data1` (with `t_data2` when both named the same array) is redirected to
    /// it; `data.inc_t_data(&mut t1, &mut t2)?`.
    pub fn inc_t_data(
        &self,
        t_data1: &mut TriangleArray,
        t_data2: &mut TriangleArray,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.nb_t_data >= state.mx_t_data {
            let grown = doubled(
                &state.t_data,
                state.mx_t_data,
                "HLRAlgo_PolyInternalData::IncTData",
            )?;
            state.mx_t_data = state.mx_t_data.wrapping_mul(2);
            state.t_data = grown.clone();
            redirect(t_data1, t_data2, grown);
        }
        state.nb_t_data += 1;
        Ok(())
    }

    /// IncPISeg: one more segment, growing as `inc_t_data`;
    /// `data.inc_pi_seg(&mut s1, &mut s2)?`.
    pub fn inc_pi_seg(
        &self,
        pi_seg1: &mut SegmentArray,
        pi_seg2: &mut SegmentArray,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.nb_pi_seg >= state.mx_pi_seg {
            let grown = doubled(
                &state.pi_seg,
                state.mx_pi_seg,
                "HLRAlgo_PolyInternalData::IncPISeg",
            )?;
            state.mx_pi_seg = state.mx_pi_seg.wrapping_mul(2);
            state.pi_seg = grown.clone();
            redirect(pi_seg1, pi_seg2, grown);
        }
        state.nb_pi_seg += 1;
        Ok(())
    }

    /// IncPINod: one more node, growing as `inc_t_data` (the node handles are shared
    /// by both arrays). The native copy reads entry 1 of both arrays first, so an empty
    /// node array raises Standard_OutOfRange; `data.inc_pi_nod(&mut n1, &mut n2)?`.
    pub fn inc_pi_nod(
        &self,
        pi_nod1: &mut NodeArray,
        pi_nod2: &mut NodeArray,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.nb_pi_nod >= state.mx_pi_nod {
            const MEMBER: &str = "HLRAlgo_PolyInternalData::IncPINod";
            check_first_entry(&state.pi_nod, MEMBER)?;
            let grown = doubled(&state.pi_nod, state.mx_pi_nod, MEMBER)?;
            check_first_entry(&grown, MEMBER)?;
            state.mx_pi_nod = state.mx_pi_nod.wrapping_mul(2);
            state.pi_nod = grown.clone();
            redirect(pi_nod1, pi_nod2, grown);
        }
        state.nb_pi_nod += 1;
        Ok(())
    }

    /// DecTData: one triangle less; `data.dec_t_data()`.
    pub fn dec_t_data(&self) {
        self.state.write().nb_t_data -= 1;
    }

    /// DecPISeg: one segment less; `data.dec_pi_seg()`.
    pub fn dec_pi_seg(&self) {
        self.state.write().nb_pi_seg -= 1;
    }

    /// DecPINod: one node less; `data.dec_pi_nod()`.
    pub fn dec_pi_nod(&self) {
        self.state.write().nb_pi_nod -= 1;
    }

    /// NbTData: the triangles in use; `data.nb_t_data()`.
    pub fn nb_t_data(&self) -> i32 {
        self.state.read().nb_t_data
    }

    /// NbPISeg: the segments in use; `data.nb_pi_seg()`.
    pub fn nb_pi_seg(&self) -> i32 {
        self.state.read().nb_pi_seg
    }

    /// NbPINod: the nodes in use; `data.nb_pi_nod()`.
    pub fn nb_pi_nod(&self) -> i32 {
        self.state.read().nb_pi_nod
    }

    /// Planar(); `data.planar()`.
    pub fn planar(&self) -> bool {
        self.state.read().planar
    }

    /// Planar(B); `data.set_planar(true)`.
    pub fn set_planar(&self, on: bool) {
        self.state.write().planar = on;
    }

    /// IntOutL(); `data.int_out_l()`.
    pub fn int_out_l(&self) -> bool {
        self.state.read().int_out_l
    }

    /// IntOutL(B); `data.set_int_out_l(true)`.
    pub fn set_int_out_l(&self, on: bool) {
        self.state.write().int_out_l = on;
    }

    /// TData(): the current triangle array (shared); `data.t_data()`.
    pub fn t_data(&self) -> TriangleArray {
        self.state.read().t_data.clone()
    }

    /// PISeg(): the current segment array (shared); `data.pi_seg()`.
    pub fn pi_seg(&self) -> SegmentArray {
        self.state.read().pi_seg.clone()
    }

    /// PINod(): the current node array (shared); `data.pi_nod()`.
    pub fn pi_nod(&self) -> NodeArray {
        self.state.read().pi_nod.clone()
    }

    /// Dump: writes the nodes, the segments and the triangles in use, in the native
    /// `std::cout` layout, to `out`; `data.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn fmt::Write) -> Result<(), Failure> {
        let (nb_nod, nb_seg, nb_tri) = (self.nb_pi_nod(), self.nb_pi_seg(), self.nb_t_data());
        let (nodes, segments, triangles) = (self.pi_nod(), self.pi_seg(), self.t_data());
        const MEMBER: &str = "HLRAlgo_PolyInternalData::Dump";
        let nodes = array_of(&nodes, MEMBER)?.array1();
        for i in 1..=nb_nod {
            let node = nodes.value(i).map_err(array_failure(MEMBER))?;
            let node = node.get().ok_or_else(|| null_handle(MEMBER, "a node"))?;
            let nd_sg = node.indices().nd_sg;
            let point = node.data().point;
            let coordinates: String = [point.x(), point.y(), point.z()]
                .iter()
                .map(|value| format!("{:>20}", format_general(*value, 6)))
                .collect();
            write_line(out, format_args!("Node {i:>6} : {nd_sg:>6}{coordinates}"))?;
        }
        let segments = array_of(&segments, MEMBER)?.array1();
        for i in 1..=nb_seg {
            let s = segments.value(i).map_err(array_failure(MEMBER))?;
            let fields = [
                s.lst_sg1, s.lst_sg2, s.nxt_sg1, s.nxt_sg2, s.conex1, s.conex2,
            ];
            write_line(out, format_args!("Segment {i:>6} : {}", ints(&fields)))?;
        }
        let triangles = array_of(&triangles, MEMBER)?.array1();
        for i in 1..=nb_tri {
            let t = triangles.value(i).map_err(array_failure(MEMBER))?;
            write_line(
                out,
                format_args!("Triangle {i:>6} : {}", ints(&[t.node1, t.node2, t.node3])),
            )?;
        }
        Ok(())
    }
}

/// Integers each right aligned in 6 columns (`std::setw(6)`).
fn ints(values: &[i32]) -> String {
    values.iter().map(|value| format!("{value:>6}")).collect()
}

fn write_line(out: &mut dyn fmt::Write, line: fmt::Arguments<'_>) -> Result<(), Failure> {
    writeln!(out, "{line}")
        .map_err(|_| Failure::new("HLRAlgo_PolyInternalData::Dump: the output refused the text"))
}

/// The constructor loop: a new node at each index `1..=nb_nod`, after the native read
/// of entry 1.
fn fill_nodes(pi_nod: &NodeArray, nb_nod: i32) -> Result<(), Failure> {
    let nodes = array_of(pi_nod, CREATE)?;
    let mut nodes = nodes.change_array1();
    nodes.change_value(1).map_err(array_failure(CREATE))?;
    for i in 1..=nb_nod {
        let slot = nodes.change_value(i).map_err(array_failure(CREATE))?;
        *slot = Handle::new(PolyInternalNode::new());
    }
    Ok(())
}

/// The native read of entry 1 of a node array before the copy loop.
fn check_first_entry(array: &NodeArray, member: &'static str) -> Result<(), Failure> {
    let nodes = array_of(array, member)?.array1();
    nodes.value(1).map(|_| ()).map_err(array_failure(member))
}

/// A new shared array `0..=2 old_size` holding entries `1..=old_size` of `old`.
fn doubled<T: Default + Clone + Send + Sync + 'static>(
    old: &Handle<HArray1<T>>,
    old_size: i32,
    member: &'static str,
) -> Result<Handle<HArray1<T>>, Failure> {
    let grown: Handle<HArray1<T>> = shared_array(old_size.wrapping_mul(2), member)?;
    {
        let source = array_of(old, member)?.array1();
        let target = array_of(&grown, member)?;
        let mut target = target.change_array1();
        for i in 1..=old_size {
            let value = source.value(i).map_err(array_failure(member))?.clone();
            *target.change_value(i).map_err(array_failure(member))? = value;
        }
    }
    Ok(grown)
}

/// The native pointer update: both pointers move when they named the same array,
/// else only the first.
fn redirect<T: Send + Sync + 'static>(
    first: &mut Handle<HArray1<T>>,
    second: &mut Handle<HArray1<T>>,
    grown: Handle<HArray1<T>>,
) {
    if first.object_address() == second.object_address() {
        *second = grown.clone();
    }
    *first = grown;
}

/// The data of a node added between two nodes (AddNode).
fn interpolated_node(
    nod1: &NodeData,
    nod2: &NodeData,
    coef1: f64,
    coef2: f64,
    point: Xyz,
) -> NodeData {
    let uv = Xy::new(
        nod1.uv.x() * coef2 + nod2.uv.x() * coef1,
        nod1.uv.y() * coef2 + nod2.uv.y() * coef1,
    );
    let normal = coef2 * nod1.normal + coef1 * nod2.normal;
    let norm = normal.modulus();
    let normal = if norm > 0. {
        (1. / norm) * normal
    } else {
        Xyz::new(1., 0., 0.)
    };
    NodeData {
        point,
        normal,
        uv,
        scal: nod1.scal * coef2 + nod2.scal * coef1,
        ..NodeData::default()
    }
}
