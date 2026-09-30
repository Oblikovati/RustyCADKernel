// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_Framework.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_Framework`: the constraint frame of the approximation, the nodes
//! and the iso-curves of the cutting grid.
use super::iso::AdvApp2VarIso;
use super::node::AdvApp2VarNode;
use rustycad_tkernel::ncollection::{Array1, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::Xy;

/// A strip of isos: the V-isos of one U interval, or the U-isos of one V interval.
pub type AdvApp2VarIsoStrip = Sequence<Handle<AdvApp2VarIso>>;

/// The nodes (row by row in V), the V-isos of each U strip (`UFrontier`) and the
/// U-isos of each V strip (`VFrontier`) (`AdvApp2Var_Framework`).
///
/// `let frame = AdvApp2VarFramework::new(nodes, u_strips, v_strips); frame.first_node(IsoType::IsoU, 2, 1)`.
#[derive(Clone, Default)]
pub struct AdvApp2VarFramework {
    node_constraints: Sequence<Handle<AdvApp2VarNode>>,
    u_constraints: Sequence<AdvApp2VarIsoStrip>,
    v_constraints: Sequence<AdvApp2VarIsoStrip>,
}

/// The `Standard_OutOfRange` of a failed lookup, with the searched values.
fn not_found(member: &str, detail: String) -> Failure {
    Failure::out_of_range(&format!("AdvApp2Var_Framework::{member}: {detail}"))
}

/// The first strip whose first iso does not span `[first, last]`, stopping at the
/// last strip (`findStripByBounds`).
fn find_strip_by_bounds(
    constraints: &Sequence<AdvApp2VarIsoStrip>,
    first: f64,
    last: f64,
) -> usize {
    let mut index: usize = 1;
    while index < constraints.len() && !first_iso_spans(&constraints[index], first, last) {
        index += 1;
    }
    index
}

fn first_iso_spans(strip: &AdvApp2VarIsoStrip, first: f64, last: f64) -> bool {
    strip
        .first()
        .is_some_and(|iso| iso.t0() == first && iso.t1() == last)
}

/// The first of the `nb_iso` first isos at `constant`, or `nb_iso + 1`
/// (`findIsoByConstant`): natively the bound is the other direction's strip count,
/// so the strip's last iso is returned whatever its constant.
fn find_iso_by_constant(strip: &AdvApp2VarIsoStrip, nb_iso: usize, constant: f64) -> usize {
    let mut index: usize = 1;
    while index <= nb_iso && strip.value(index).map(|iso| iso.constante()).ok() != Some(constant) {
        index += 1;
    }
    index
}

impl AdvApp2VarFramework {
    /// The frame of `frame` nodes and the iso strips (`AdvApp2Var_Framework(Frame,
    /// UFrontier, VFrontier)`); the handles are shared with the caller.
    /// `AdvApp2VarFramework::new(nodes, u_strips, v_strips)`.
    pub fn new(
        frame: Sequence<Handle<AdvApp2VarNode>>,
        u_frontier: Sequence<AdvApp2VarIsoStrip>,
        v_frontier: Sequence<AdvApp2VarIsoStrip>,
    ) -> Self {
        Self {
            node_constraints: frame,
            u_constraints: u_frontier,
            v_constraints: v_frontier,
        }
    }

    /// The first iso not approximated, V-isos of the U strips first, with its
    /// indices `(iso, strip)`; `None` when all are (`FirstNotApprox`).
    /// `frame.first_not_approx()`.
    pub fn first_not_approx(&self) -> Option<(Handle<AdvApp2VarIso>, i32, i32)> {
        for constraints in [&self.u_constraints, &self.v_constraints] {
            for (i, strip) in (1..).zip(constraints.iter()) {
                for (j, iso) in (1..).zip(strip.iter()) {
                    if !iso.is_approximated() {
                        return Some((iso.clone(), j, i));
                    }
                }
            }
        }
        None
    }

    /// The index of the node at the start of an iso (`FirstNode`); `frame.first_node(IsoType::IsoU, 2, 1)`.
    pub fn first_node(&self, iso_type: IsoType, index_iso: i32, index_strip: i32) -> i32 {
        let nb_iso_in_v: i32 = self.u_constraints.length() + 1;
        if iso_type == IsoType::IsoU {
            return nb_iso_in_v * (index_strip - 1) + index_iso;
        }
        nb_iso_in_v * (index_iso - 1) + index_strip
    }

    /// The index of the node at the end of an iso (`LastNode`); `frame.last_node(IsoType::IsoU, 2, 1)`.
    pub fn last_node(&self, iso_type: IsoType, index_iso: i32, index_strip: i32) -> i32 {
        let nb_iso_in_v: i32 = self.u_constraints.length() + 1;
        if iso_type == IsoType::IsoU {
            return nb_iso_in_v * index_strip + index_iso;
        }
        nb_iso_in_v * (index_iso - 1) + index_strip + 1
    }

    /// Replaces the iso `(index_iso, index_strip)` of the strips of its type
    /// (`ChangeIso`); Standard_OutOfRange outside them. `frame.change_iso(1, 1, &iso)?`.
    pub fn change_iso(
        &mut self,
        index_iso: i32,
        index_strip: i32,
        iso: &Handle<AdvApp2VarIso>,
    ) -> Result<(), Failure> {
        let strips = if iso.iso_type() == IsoType::IsoV {
            &mut self.u_constraints
        } else {
            &mut self.v_constraints
        };
        let strip = strips.change_value(index_strip.max(0) as usize)?;
        strip.set_value(index_iso.max(0) as usize, iso.clone())
    }

    /// The node of index `index_node` (`Node(IndexNode)`); `frame.node(1)?`.
    pub fn node(&self, index_node: i32) -> Result<&Handle<AdvApp2VarNode>, Failure> {
        self.node_constraints.value(index_node.max(0) as usize)
    }

    /// The node at `(u, v)`, or the last node when none is there (`Node(U, V)`);
    /// Standard_OutOfRange for an empty frame. `frame.node_at(0.5, 0.0)?`.
    pub fn node_at(&self, u: f64, v: f64) -> Result<&Handle<AdvApp2VarNode>, Failure> {
        let found = self.node_constraints.iter().find(|node| {
            let coord: Xy = node.coord();
            coord.x() == u && coord.y() == v
        });
        found
            .or_else(|| self.node_constraints.last())
            .ok_or_else(|| not_found("Node", format!("no node at ({u}, {v}) in an empty frame")))
    }

    /// The U-iso at `u` of the V strip `[v0, v1]` (`IsoU`); Standard_OutOfRange when
    /// the lookup runs past the strip. `frame.iso_u(20.0, 0.0, 1.0)?`.
    pub fn iso_u(&self, u: f64, v0: f64, v1: f64) -> Result<&Handle<AdvApp2VarIso>, Failure> {
        let strip = self
            .v_constraints
            .value(find_strip_by_bounds(&self.v_constraints, v0, v1))?;
        let index: usize = find_iso_by_constant(strip, self.u_constraints.len(), u);
        strip
            .value(index)
            .map_err(|_| not_found("IsoU", format!("no iso U = {u} on [{v0}, {v1}]")))
    }

    /// The V-iso at `v` of the U strip `[u0, u1]` (`IsoV`); `frame.iso_v(1.0, 2.0, 40.0)?`.
    pub fn iso_v(&self, u0: f64, u1: f64, v: f64) -> Result<&Handle<AdvApp2VarIso>, Failure> {
        let strip = self
            .u_constraints
            .value(find_strip_by_bounds(&self.u_constraints, u0, u1))?;
        let index: usize = find_iso_by_constant(strip, self.v_constraints.len(), v);
        strip
            .value(index)
            .map_err(|_| not_found("IsoV", format!("no iso V = {v} on [{u0}, {u1}]")))
    }

    /// The polynomial of the U-iso `index_iso` of V strip `index_strip` (`UEquation`);
    /// `frame.u_equation(1, 1)?`.
    pub fn u_equation(
        &self,
        index_iso: i32,
        index_strip: i32,
    ) -> Result<Option<Array1<'static, f64>>, Failure> {
        let strip = self.v_constraints.value(index_strip.max(0) as usize)?;
        Ok(strip.value(index_iso.max(0) as usize)?.polynom())
    }

    /// The polynomial of the V-iso `index_iso` of U strip `index_strip` (`VEquation`);
    /// `frame.v_equation(1, 1)?`.
    pub fn v_equation(
        &self,
        index_iso: i32,
        index_strip: i32,
    ) -> Result<Option<Array1<'static, f64>>, Failure> {
        let strip = self.u_constraints.value(index_strip.max(0) as usize)?;
        Ok(strip.value(index_iso.max(0) as usize)?.polynom())
    }

    pub(super) fn nodes_mut(&mut self) -> &mut Sequence<Handle<AdvApp2VarNode>> {
        &mut self.node_constraints
    }

    pub(super) fn strips_mut(
        &mut self,
    ) -> (
        &mut Sequence<AdvApp2VarIsoStrip>,
        &mut Sequence<AdvApp2VarIsoStrip>,
    ) {
        (&mut self.u_constraints, &mut self.v_constraints)
    }
}
