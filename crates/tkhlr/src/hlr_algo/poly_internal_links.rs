// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_PolyInternalData.cxx (the two UpdateLinks) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The segment links of an internal triangulation built from its triangles
//! (poly_internal_split.rs updates them when a node splits a segment).
//!
//! Every node keeps the first segment at it (`nd_sg`); a segment continues the chain
//! of its first end with `nxt_sg1` and of its second end with `nxt_sg2`, and names
//! the triangles on its two sides (`conex1`, `conex2`).
use super::failures::{array_failure, null_handle};
use super::poly_internal_data::PolyInternalData;
use super::poly_internal_node::PolyInternalNode;
use super::poly_records::{PolyInternalSegment, TriangleData};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};

pub(super) const LINKS: &str = "HLRAlgo_PolyInternalData::UpdateLinks";

/// The node at `index` of `nodes` (a null node crashes natively).
pub(super) fn node_at(
    nodes: &Array1<'static, Handle<PolyInternalNode>>,
    index: i32,
) -> Result<Handle<PolyInternalNode>, Failure> {
    let node = nodes.value(index).map_err(array_failure(LINKS))?;
    if node.is_null() {
        return Err(null_handle(LINKS, "a node"));
    }
    Ok(node.clone())
}

/// The node behind a handle checked by [`node_at`].
pub(super) fn node(handle: &Handle<PolyInternalNode>) -> Result<&PolyInternalNode, Failure> {
    handle.get().ok_or_else(|| null_handle(LINKS, "a node"))
}

pub(super) fn segment<'a>(
    segments: &'a mut Array1<'static, PolyInternalSegment>,
    index: i32,
) -> Result<&'a mut PolyInternalSegment, Failure> {
    segments.change_value(index).map_err(array_failure(LINKS))
}

impl PolyInternalData {
    /// UpdateLinks(TData, PISeg, PINod): builds the segments of the triangles in use,
    /// edge by edge (`(Node1, Node2)`, `(Node2, Node3)`, `(Node3, Node1)`): an edge met
    /// again gets its second triangle, a new edge a new segment chained at both ends;
    /// `data.update_links(&mut triangles, &mut segments, &nodes)?`.
    pub fn update_links(
        &self,
        t_data: &mut Array1<'static, TriangleData>,
        pi_seg: &mut Array1<'static, PolyInternalSegment>,
        pi_nod: &Array1<'static, Handle<PolyInternalNode>>,
    ) -> Result<(), Failure> {
        let nb_t_data = self.nb_t_data();
        for i in 1..=nb_t_data {
            let td = *t_data.change_value(i).map_err(array_failure(LINKS))?;
            let nodes = [
                node_at(pi_nod, td.node1)?,
                node_at(pi_nod, td.node2)?,
                node_at(pi_nod, td.node3)?,
            ];
            let edges = [
                (td.node1, td.node2, 0, 1),
                (td.node2, td.node3, 1, 2),
                (td.node3, td.node1, 2, 0),
            ];
            for (n1, n2, a, b) in edges {
                self.link_edge(pi_seg, i, (n1, n2), (node(&nodes[a])?, node(&nodes[b])?))?;
            }
        }
        Ok(())
    }

    /// One edge block of the first UpdateLinks: triangle `i`, edge `(n1, n2)` with the
    /// nodes `(a, b)` at its ends (the same node when the triangle is degenerate).
    fn link_edge(
        &self,
        segments: &mut Array1<'static, PolyInternalSegment>,
        i: i32,
        (n1, n2): (i32, i32),
        (a, b): (&PolyInternalNode, &PolyInternalNode),
    ) -> Result<(), Failure> {
        // One guard at a time: `a` and `b` are the same node on a degenerate triangle.
        let a_first = a.indices().nd_sg;
        let b_first = b.indices().nd_sg;
        let new_seg = if a_first == 0 && b_first == 0 {
            let nb = self.next_segment();
            a.indices().nd_sg = nb;
            b.indices().nd_sg = nb;
            true
        } else {
            let new_seg = if a_first != 0 {
                self.find_or_chain(segments, i, (n1, n2), a_first)?
            } else {
                let nb = self.next_segment();
                a.indices().nd_sg = nb;
                true
            };
            if new_seg {
                self.chain_at_second_end(segments, n2, b)?;
            }
            new_seg
        };
        if new_seg {
            let nb = self.nb_pi_seg();
            *segment(segments, nb)? = PolyInternalSegment {
                lst_sg1: n1,
                lst_sg2: n2,
                nxt_sg1: 0,
                nxt_sg2: 0,
                conex1: i,
                conex2: 0,
            };
        }
        Ok(())
    }

    /// Increments the segment count and returns it.
    pub(super) fn next_segment(&self) -> i32 {
        let mut state = self.state.write();
        state.nb_pi_seg += 1;
        state.nb_pi_seg
    }

    /// Walks the chain of `n1` from `first` for the segment `(n1, n2)`: when found it
    /// gets the second triangle `i`; otherwise a new segment is chained after the last
    /// one walked. Returns whether a segment was created.
    fn find_or_chain(
        &self,
        segments: &mut Array1<'static, PolyInternalSegment>,
        i: i32,
        (n1, n2): (i32, i32),
        first: i32,
    ) -> Result<bool, Failure> {
        let (mut iiii, mut find, mut icsv, mut last) = (first, 0, 0, first);
        while iiii != 0 && find == 0 {
            last = iiii;
            let s = segment(segments, iiii)?;
            if s.lst_sg1 == n1 {
                if s.lst_sg2 == n2 {
                    find = iiii;
                } else {
                    iiii = s.nxt_sg1;
                }
                icsv = 1;
            } else {
                if s.lst_sg1 == n2 {
                    find = iiii;
                } else {
                    iiii = s.nxt_sg2;
                }
                icsv = 2;
            }
        }
        if find != 0 {
            segment(segments, last)?.conex2 = i;
            return Ok(false);
        }
        let nb = self.next_segment();
        let s = segment(segments, last)?;
        if icsv == 1 {
            s.nxt_sg1 = nb;
        } else {
            s.nxt_sg2 = nb;
        }
        Ok(true)
    }

    /// Chains the new segment at the end of the chain of the second end `n2` (node
    /// `b`), or starts that chain.
    fn chain_at_second_end(
        &self,
        segments: &mut Array1<'static, PolyInternalSegment>,
        n2: i32,
        b: &PolyInternalNode,
    ) -> Result<(), Failure> {
        let nb = self.nb_pi_seg();
        let mut iiii = b.indices().nd_sg;
        if iiii == 0 {
            b.indices().nd_sg = nb;
            return Ok(());
        }
        let (mut icsv, mut last) = (0, iiii);
        while iiii != 0 {
            last = iiii;
            let s = segment(segments, iiii)?;
            if s.lst_sg1 == n2 {
                icsv = 1;
                iiii = s.nxt_sg1;
            } else {
                icsv = 2;
                iiii = s.nxt_sg2;
            }
        }
        let s = segment(segments, last)?;
        if icsv == 1 {
            s.nxt_sg1 = nb;
        } else {
            s.nxt_sg2 = nb;
        }
        Ok(())
    }
}
