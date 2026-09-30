// Rust translation of BRepBuilderAPI_Sewing::MergedNearestEdges
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sections to merge with a boundary or section: those joining the same two nodes
//! (or their cutting nodes), filtered by `FindCandidates`.
use super::BRepBuilderAPISewing;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::{IndexedMap, Sequence};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;

impl BRepBuilderAPISewing {
    /// The node of a vertex, or the vertex itself.
    fn node_of(&self, vertex: &Shape) -> Shape {
        match self.my_vertex_node.seek(vertex) {
            Some(node) => node.clone(),
            None => vertex.clone(),
        }
    }

    /// The node and the cutting nodes linked to it (and, for a vertex that is not a
    /// node, their own links too).
    fn connected_nodes(&self, node: &Shape, is_node: bool, nodes: &mut impl FnMut(Shape)) {
        nodes(node.clone());
        let Ok(links) = self.my_cutting_node.find(node) else {
            return;
        };
        for v1 in links.iter() {
            nodes(v1.clone());
            if is_node {
                continue;
            }
            if let Ok(inner) = self.my_cutting_node.find(v1) {
                for vn in inner.iter() {
                    nodes(vn.clone());
                }
            }
        }
    }

    /// Whether a section is rejected: merged already, a cut boundary with a merged
    /// section, or a section whose boundary is not cut any more or merged.
    fn is_rejected_section(&self, sec: &Shape) -> bool {
        if self.my_merged_edges.contains(sec) {
            return true;
        }
        if let Ok(sections) = self.my_bound_sections.find(sec)
            && sections
                .iter()
                .any(|section| self.my_merged_edges.contains(section))
        {
            return true;
        }
        match self.my_section_bound.find(sec) {
            Ok(bnd) => !self.my_bound_sections.is_bound(bnd) || self.my_merged_edges.contains(bnd),
            Err(_) => false,
        }
    }

    /// The sections that join the nodes of the two ends of `edge`, after it.
    fn contiguous_sections(&self, edge: &Shape) -> Result<Sequence<Shape>, Failure> {
        let (no1, no2) = topexp::vertices(&topods::edge(edge)?, false);
        let (no1, no2) = (no1.into_shape(), no2.into_shape());
        let (is_node1, is_node2) = (
            self.my_vertex_node.contains(&no1),
            self.my_vertex_node.contains(&no2),
        );
        let mut map_vert1 = IndexedMapOfShape::default();
        self.connected_nodes(&self.node_of(&no1), is_node1, &mut |v| {
            map_vert1.add(v);
        });
        let mut map_vert2 = MapOfShape::default();
        self.connected_nodes(&self.node_of(&no2), is_node2, &mut |v| {
            map_vert2.add(v);
        });
        let mut seq_edges: Sequence<Shape> = Sequence::default();
        seq_edges.append(edge.clone());
        let mut map_edges = MapOfShape::default();
        map_edges.add(edge.clone());
        for i in 1..=map_vert1.extent() {
            let Ok(sections) = self.my_node_sections.find(map_vert1.find_key(i)?) else {
                continue;
            };
            for sec in sections.iter() {
                if sec.is_same(edge) {
                    continue;
                }
                let (vs1, vs2) = topexp::vertices(&topods::edge(sec)?, false);
                let (vs1n, vs2n) = (self.node_of(&vs1), self.node_of(&vs2));
                let joins = (map_vert1.contains(&vs1n) && map_vert2.contains(&vs2n))
                    || (map_vert1.contains(&vs2n) && map_vert2.contains(&vs1n));
                if joins && map_edges.add(sec.clone()) && !self.is_rejected_section(sec) {
                    seq_edges.append(sec.clone());
                }
            }
        }
        Ok(seq_edges)
    }

    /// The index of the longest section (non-manifold mode; 1 otherwise), which is then
    /// swapped with the first one.
    fn longest_first(&self, seq_edges: &mut Sequence<Shape>) -> Result<i32, Failure> {
        if !self.my_nonmanifold {
            return Ok(1);
        }
        let (mut ind_ref, mut len_ref) = (1, 0.0);
        for i in 1..=seq_edges.len() {
            let (c3d, f, l) = BRepTool::curve(&topods::edge(seq_edges.value(i)?)?)?;
            let adaptor = GeomAdaptorCurve::with_curve(&c3d)?;
            let len = AbscissaPoint::length_between(&adaptor, f, l)?;
            if len > len_ref {
                ind_ref = i;
                len_ref = len;
            }
        }
        if ind_ref != 1 {
            seq_edges.exchange(ind_ref, 1)?;
        }
        Ok(ind_ref as i32)
    }

    /// `MergedNearestEdges(edge, SeqMergedEdge, SeqMergedOri)`: the sections to merge
    /// with `edge` (only the best in manifold mode) and their relative orientations;
    /// false when there is none. In non-manifold mode the longest section is moved to
    /// the front but the reference stays at its index (the original section), as
    /// natively.
    pub(crate) fn merged_nearest_edges(
        &self,
        edge: &Shape,
        seq_merged_edge: &mut Sequence<Shape>,
        seq_merged_ori: &mut Sequence<bool>,
    ) -> Result<bool, Failure> {
        let mut seq_edges = self.contiguous_sections(edge)?;
        if seq_edges.len() <= 1 {
            return Ok(false);
        }
        // CCI60011: find the longest edge.
        let ind_ref = self.longest_first(&mut seq_edges)?;
        let (mut seq_forward, mut seq_candidates) = (Sequence::default(), Sequence::default());
        let mut map_reference: IndexedMap<i32> = IndexedMap::default();
        map_reference.add(ind_ref);
        if !self.find_candidates(
            &seq_edges,
            &mut map_reference,
            &mut seq_candidates,
            &mut seq_forward,
        )? {
            return Ok(false);
        }
        let nb_candidates = seq_candidates.len();
        for i in 1..=nb_candidates {
            let iedge = seq_edges.value(*seq_candidates.value(i)? as usize)?.clone();
            let ori = *seq_forward.value(i)?;
            seq_merged_edge.append(iedge);
            seq_merged_ori.append(ori);
            if !self.my_nonmanifold {
                break;
            }
        }
        Ok(nb_candidates != 0)
    }
}
