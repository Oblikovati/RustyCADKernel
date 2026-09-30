// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_PolyInternalData.cxx (the split UpdateLinks) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The split of a segment of an internal triangulation by a new node, and the
//! relinking of the segments around the triangles it splits.
use super::failures::array_failure;
use super::poly_internal_data::{
    NodeArray, PolyInternalData, SegmentArray, TriangleArray, array_of,
};
use super::poly_internal_links::{LINKS, node, node_at, segment};
use super::poly_internal_node::PolyInternalNode;
use super::poly_records::{PolyInternalSegment, TriangleData};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};

impl PolyInternalData {
    /// UpdateLinks(ip1, ip2, ip3, ...): the node `ip3` splits the segment `(ip1, ip2)`:
    /// the segment keeps `(ip1, ip3)` or `(ip3, ip1)`, a new segment `(ip3, ip2)` follows
    /// it, each triangle on the segment is split in two with a new segment from `ip3`
    /// to its opposite node, and the segments around the new triangles get their new
    /// neighbours. The arrays grow through `t_data1`/`t_data2` and `pi_seg1`/`pi_seg2`;
    /// the nodes are read from `pi_nod1` (`pi_nod2` is unused, as natively). When the
    /// segment is not found only the counts change;
    /// `data.update_links_split(1, 2, 5, &mut t1, &mut t2, &mut s1, &mut s2, &mut n1, &mut n2)?`.
    #[allow(clippy::too_many_arguments)] // The native member takes these 9 values.
    pub fn update_links_split(
        &self,
        ip1: i32,
        ip2: i32,
        ip3: i32,
        t_data1: &mut TriangleArray,
        t_data2: &mut TriangleArray,
        pi_seg1: &mut SegmentArray,
        pi_seg2: &mut SegmentArray,
        pi_nod1: &mut NodeArray,
        _pi_nod2: &mut NodeArray,
    ) -> Result<(), Failure> {
        for _ in 0..3 {
            self.inc_pi_seg(pi_seg1, pi_seg2)?;
        }
        self.state.write().nb_pi_seg -= 2;
        for _ in 0..2 {
            self.inc_t_data(t_data1, t_data2)?;
        }
        self.state.write().nb_t_data -= 2;
        let nodes = array_of(pi_nod1, LINKS)?.array1();
        node_at(&nodes, ip1)?;
        let (node2, node3) = (node_at(&nodes, ip2)?, node_at(&nodes, ip3)?);
        let segments = array_of(pi_seg1, LINKS)?;
        let mut segments = segments.change_array1();
        let Some(split) = self.split_segment(&mut segments, (ip1, ip2, ip3), node(&node2)?)? else {
            self.state.write().nb_pi_seg -= 1;
            return Ok(());
        };
        let nb = self.nb_pi_seg();
        *segment(&mut segments, nb)? = PolyInternalSegment {
            lst_sg1: ip3,
            lst_sg2: ip2,
            nxt_sg1: 0,
            nxt_sg2: split.iip2,
            conex1: split.cnx1,
            conex2: split.cnx2,
        };
        node(&node3)?.indices().nd_sg = split.find;
        let triangles = array_of(t_data1, LINKS)?;
        let mut triangles = triangles.change_array1();
        let context = SplitContext {
            ips: (ip1, ip2, ip3),
            nodes: &nodes,
            node3: node(&node3)?,
        };
        self.split_triangles(&mut triangles, &mut segments, &context, split)
    }
}

/// What the walk of the chain of `ip2` found: the split segment, its old next segment
/// at `ip2` and its triangles.
#[derive(Clone, Copy)]
struct SplitSegment {
    find: i32,
    iip2: i32,
    cnx1: i32,
    cnx2: i32,
}

/// The inputs of the triangle split shared by its steps.
struct SplitContext<'a> {
    ips: (i32, i32, i32),
    nodes: &'a Array1<'static, Handle<PolyInternalNode>>,
    node3: &'a PolyInternalNode,
}

impl PolyInternalData {
    /// The walk of the chain of `ip2` for the segment `(ip2, ip1)` or `(ip1, ip2)`:
    /// the found segment ends at `ip3` instead of `ip2` and the new segment takes its
    /// place in the chain of `ip2`.
    fn split_segment(
        &self,
        segments: &mut Array1<'static, PolyInternalSegment>,
        (ip1, ip2, ip3): (i32, i32, i32),
        node2: &PolyInternalNode,
    ) -> Result<Option<SplitSegment>, Failure> {
        let nb = self.nb_pi_seg();
        let (mut iiii, mut iisv, mut icsv, mut previous) = (node2.indices().nd_sg, 0, 0, 0);
        while iiii != 0 {
            let s = segment(segments, iiii)?;
            let first_end = s.lst_sg1 == ip2;
            let found = if first_end {
                s.lst_sg2 == ip1
            } else {
                s.lst_sg1 == ip1
            };
            if !found {
                iisv = iiii;
                icsv = if first_end { 1 } else { 2 };
                iiii = if first_end { s.nxt_sg1 } else { s.nxt_sg2 };
                previous = iisv;
                continue;
            }
            let split = SplitSegment {
                find: iiii,
                iip2: if first_end { s.nxt_sg1 } else { s.nxt_sg2 },
                cnx1: s.conex1,
                cnx2: s.conex2,
            };
            if first_end {
                s.lst_sg1 = ip3;
                s.nxt_sg1 = nb;
            } else {
                s.lst_sg2 = ip3;
                s.nxt_sg2 = nb;
            }
            if iisv == 0 {
                node2.indices().nd_sg = nb;
            } else if icsv == 1 {
                segment(segments, previous)?.nxt_sg1 = nb;
            } else {
                segment(segments, previous)?.nxt_sg2 = nb;
            }
            return Ok(Some(split));
        }
        Ok(None)
    }

    /// The two triangle loops of the second UpdateLinks: split each triangle on the
    /// segment (old index `New[skip]`, new index `New[skip + 2]`), then relink the
    /// segments of the four triangles.
    fn split_triangles(
        &self,
        triangles: &mut Array1<'static, TriangleData>,
        segments: &mut Array1<'static, PolyInternalSegment>,
        context: &SplitContext<'_>,
        split: SplitSegment,
    ) -> Result<(), Failure> {
        let nb_t_data = self.nb_t_data();
        let (cnx1, cnx2) = (split.cnx1, split.cnx2);
        let new = if cnx2 == 0 {
            [cnx1, 0, nb_t_data + 1, 0]
        } else {
            [cnx1, cnx2, nb_t_data + 1, nb_t_data + 2]
        };
        // Natively `itpk[skip]` stays uninitialized when the triangle does not hold the
        // segment; the port reads 0 there (docs/hlr-design.md section 5).
        let mut itpk = [0_i32; 2];
        for skip in 0..2 {
            let (i_old, i_new) = (new[skip], new[skip + 2]);
            if i_old != 0 {
                itpk[skip] =
                    self.split_triangle(triangles, segments, context, (i_old, i_new), itpk[skip])?;
            }
        }
        for (i_tr, &i_new) in new.iter().enumerate() {
            let skip = i_tr % 2;
            if i_new != 0 {
                let triangle = *triangles
                    .change_value(i_new)
                    .map_err(array_failure(LINKS))?;
                let relink = Relink {
                    ip3: context.ips.2,
                    ip4: itpk[skip],
                    i_old: new[skip],
                    i_new,
                };
                relink.triangle(segments, context.nodes, &triangle)?;
            }
        }
        Ok(())
    }

    /// Splits the triangle `i_old` holding the segment `(ip1, ip2)` (in either
    /// direction): it keeps one half, the triangle `i_new` gets the other, and a new
    /// segment joins `ip3` to the opposite node. Returns the opposite node, or
    /// `unset` when the triangle does not hold the segment (nothing changes then).
    fn split_triangle(
        &self,
        triangles: &mut Array1<'static, TriangleData>,
        segments: &mut Array1<'static, PolyInternalSegment>,
        context: &SplitContext<'_>,
        (i_old, i_new): (i32, i32),
        unset: i32,
    ) -> Result<i32, Failure> {
        let old = *triangles
            .change_value(i_old)
            .map_err(array_failure(LINKS))?;
        triangles
            .change_value(i_new)
            .map_err(array_failure(LINKS))?;
        let Some((n_old, n_new, ip4)) = split_nodes(&old, context.ips) else {
            return Ok(unset);
        };
        self.state.write().nb_t_data += 1;
        let node4 = node_at(context.nodes, ip4)?;
        write_nodes(triangles, i_old, n_old)?;
        write_nodes(triangles, i_new, n_new)?;
        let flags = triangles.value(i_old).map_err(array_failure(LINKS))?.flags;
        triangles
            .change_value(i_new)
            .map_err(array_failure(LINKS))?
            .flags = flags;
        let nb = self.next_segment();
        let node4 = node(&node4)?;
        let nxt_sg1 = context.node3.indices().nd_sg;
        let nxt_sg2 = node4.indices().nd_sg;
        *segment(segments, nb)? = PolyInternalSegment {
            lst_sg1: context.ips.2,
            lst_sg2: ip4,
            nxt_sg1,
            nxt_sg2,
            conex1: i_old,
            conex2: i_new,
        };
        context.node3.indices().nd_sg = nb;
        node4.indices().nd_sg = nb;
        Ok(ip4)
    }
}

fn write_nodes(
    triangles: &mut Array1<'static, TriangleData>,
    index: i32,
    nodes: [i32; 3],
) -> Result<(), Failure> {
    let t = triangles
        .change_value(index)
        .map_err(array_failure(LINKS))?;
    [t.node1, t.node2, t.node3] = nodes;
    Ok(())
}

/// The six cases of the native split: where the segment `(ip1, ip2)` lies in the
/// triangle, the nodes of the kept half, of the new half, and the opposite node.
fn split_nodes(
    t: &TriangleData,
    (ip1, ip2, ip3): (i32, i32, i32),
) -> Option<([i32; 3], [i32; 3], i32)> {
    let (n1, n2, n3) = (t.node1, t.node2, t.node3);
    let (mut n_old, mut n_new) = ([n1, n2, n3], [n1, n2, n3]);
    let opposite = if n1 == ip1 && n2 == ip2 {
        (n_old[1], n_new[0]) = (ip3, ip3);
        n3
    } else if n1 == ip2 && n2 == ip1 {
        (n_old[0], n_new[1]) = (ip3, ip3);
        n3
    } else if n2 == ip1 && n3 == ip2 {
        (n_old[2], n_new[1]) = (ip3, ip3);
        n1
    } else if n2 == ip2 && n3 == ip1 {
        (n_old[1], n_new[2]) = (ip3, ip3);
        n1
    } else if n3 == ip1 && n1 == ip2 {
        (n_old[0], n_new[2]) = (ip3, ip3);
        n2
    } else if n3 == ip2 && n1 == ip1 {
        (n_old[2], n_new[0]) = (ip3, ip3);
        n2
    } else {
        return None;
    };
    Some((n_old, n_new, opposite))
}

/// The relinking of one new or old triangle after the split: each of its edges but
/// the new segment `(ip3, ip4)` has its triangle `i_old` replaced by `i_new`.
struct Relink {
    ip3: i32,
    ip4: i32,
    i_old: i32,
    i_new: i32,
}

impl Relink {
    fn triangle(
        &self,
        segments: &mut Array1<'static, PolyInternalSegment>,
        nodes: &Array1<'static, Handle<PolyInternalNode>>,
        t: &TriangleData,
    ) -> Result<(), Failure> {
        for (a, b) in [(t.node1, t.node2), (t.node2, t.node3), (t.node3, t.node1)] {
            let is_new_segment =
                (a == self.ip3 && b == self.ip4) || (b == self.ip3 && a == self.ip4);
            if !is_new_segment {
                self.edge(segments, nodes, a, b)?;
            }
        }
        Ok(())
    }

    /// Walks the chain of `a` for the segment `(a, b)` and moves its triangle.
    fn edge(
        &self,
        segments: &mut Array1<'static, PolyInternalSegment>,
        nodes: &Array1<'static, Handle<PolyInternalNode>>,
        a: i32,
        b: i32,
    ) -> Result<(), Failure> {
        let mut iiii = node(&node_at(nodes, a)?)?.indices().nd_sg;
        while iiii != 0 {
            let s = segment(segments, iiii)?;
            let found = if s.lst_sg1 == a {
                s.lst_sg2 == b
            } else {
                s.lst_sg1 == b
            };
            if !found {
                iiii = if s.lst_sg1 == a { s.nxt_sg1 } else { s.nxt_sg2 };
                continue;
            }
            if s.conex1 == self.i_old {
                s.conex1 = self.i_new;
            } else if s.conex2 == self.i_old {
                s.conex2 = self.i_new;
            }
            return Ok(());
        }
        Ok(())
    }
}
