// Rust translation of BRepBuilderAPI_Sewing::Merging (BRepBuilderAPI_Sewing.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The merge of the free boundaries: each boundary is merged whole with its nearest
//! sections, or piecewise through its cutting sections, whichever the tolerances
//! favour, and the context records the replacements.
use super::merging_sections::MergeTarget;
use super::replace_edge::replace_edge;
use super::{BRepBuilderAPISewing, SewingShapeIndexedMap, live_reshape};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The outcome of the merges of one boundary.
struct BoundMerge {
    bound_wire: Wire,
    is_prev_split: bool,
    with_bound: SewingShapeIndexedMap,
    with_sections: SewingShapeIndexedMap,
    sections_reshape: Handle<dyn ReShape>,
}

impl BRepBuilderAPISewing {
    /// `Merging(passage, theProgress)`: every boundary not merged yet is merged; a
    /// floating edge only takes its glued nodes. Then the nodes are forgotten and
    /// their count kept.
    pub(crate) fn merging(
        &mut self,
        _passage: bool,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let scope = ProgressScope::with_literal(
            progress,
            "Merging bounds",
            self.my_bound_faces.extent() as f64,
            false,
        );
        let mut i = 1;
        while i <= self.my_bound_faces.extent() && scope.more() {
            let bound = self.my_bound_faces.find_key(i)?.clone();
            if !self.my_merged_edges.contains(&bound) {
                if self.my_bound_faces.find_from_index(i)?.is_empty() {
                    self.merge_floating(&bound)?;
                } else {
                    self.merge_bound(&bound)?;
                }
            }
            i += 1;
            scope.next(1.0);
        }
        self.my_nb_vertices =
            (self.my_vertex_node.extent() + self.my_vertex_node_free.extent()) as i32;
        self.my_node_sections.clear(true);
        self.my_vertex_node.clear(true);
        self.my_vertex_node_free.clear(true);
        self.my_cutting_node.clear(true);
        Ok(())
    }

    /// A free edge: only its vertices are replaced by their nodes.
    fn merge_floating(&mut self, bound: &Shape) -> Result<(), Failure> {
        let reshape = live_reshape(&self.my_re_shape)?;
        let (no1, no2) = topexp::vertices(&topods::edge(bound)?, false);
        for no in [no1.into_shape(), no2.into_shape()] {
            let Some(node) = self.my_vertex_node_free.seek(&no) else {
                continue;
            };
            if !no.is_same(node) {
                reshape.replace(&no, &node.oriented(no.orientation()))?;
            }
        }
        self.my_merged_edges.add(bound.clone());
        Ok(())
    }

    /// The wire of the cutting sections of a boundary, and whether one of them is
    /// merged already.
    fn bound_wire(&self, bound: &Shape) -> Result<(Wire, bool), Failure> {
        let mut wire = Wire::default();
        let mut is_prev_split = false;
        let Ok(sections) = self.my_bound_sections.find(bound) else {
            return Ok((wire, is_prev_split));
        };
        let b = BRepBuilder::default();
        b.make_wire(&mut wire);
        wire.set_orientation(bound.orientation());
        for section in sections.iter() {
            b.add(&mut wire, section)?;
            if self.my_merged_edges.contains(section) {
                is_prev_split = true;
            }
        }
        Ok((wire, is_prev_split))
    }

    /// The merges of one boundary: whole (unless a section was merged before), and
    /// section by section; the sections are not merged when one finds nothing while
    /// the whole boundary was merged.
    fn merges_of(&self, bound: &Shape) -> Result<BoundMerge, Failure> {
        let (bound_wire, is_prev_split) = self.bound_wire(bound)?;
        let mut merge = BoundMerge {
            bound_wire,
            is_prev_split,
            with_bound: SewingShapeIndexedMap::default(),
            with_sections: SewingShapeIndexedMap::default(),
            sections_reshape: Handle::new(BRepToolsReShape::new()).cast(),
        };
        if !is_prev_split {
            let mut target = MergeTarget {
                merged: &mut merge.with_bound,
                loc_reshape: &self.my_re_shape,
                replace_sections: false,
            };
            self.merge_nearest(bound, &mut target)?;
        }
        let is_merged = !merge.with_bound.is_empty();
        let Ok(sections) = self.my_bound_sections.find(bound) else {
            return Ok(merge);
        };
        let mut target = MergeTarget {
            merged: &mut merge.with_sections,
            loc_reshape: &merge.sections_reshape,
            replace_sections: true,
        };
        for section in sections.iter() {
            if self.my_merged_edges.contains(section) {
                continue;
            }
            if !self.merge_nearest(section, &mut target)? && is_merged {
                // Reject merging of sections.
                target.merged.clear(true);
                break;
            }
        }
        Ok(merge)
    }

    /// Whether the sections win over the whole boundary: some merged section is not
    /// already part of the whole merge, and the smallest tolerance of those is below
    /// the tolerance of the whole merge (plus the minimal tolerance), or non-manifold.
    fn sections_win(&self, bound: &Shape, merge: &BoundMerge) -> Result<bool, Failure> {
        let mut map_split_edges = IndexedMapOfShape::default();
        for i in 1..=merge.with_sections.extent() {
            map_split_edges.add(merge.with_sections.find_key(i)?.clone());
        }
        for i in 1..=merge.with_bound.extent() {
            let edge = merge.with_bound.find_key(i)?;
            map_split_edges.remove_key(edge);
            if let Ok(sections) = self.my_bound_sections.find(edge) {
                for sec in sections.iter() {
                    map_split_edges.remove_key(sec);
                }
            }
        }
        let mut min_split_tol = real_last();
        for i in 1..=map_split_edges.extent() {
            let edge = topods::edge(
                merge
                    .with_sections
                    .find_from_key(map_split_edges.find_key(i)?)?,
            )?;
            min_split_tol = std_min(min_split_tol, BRepTool::edge_tolerance(&edge)?);
        }
        let bound_edge = topods::edge(merge.with_bound.find_from_key(bound)?)?;
        let bound_edge_tol = BRepTool::edge_tolerance(&bound_edge)?;
        let is_splitted =
            min_split_tol < bound_edge_tol + self.min_tolerance() || self.my_nonmanifold;
        Ok(!map_split_edges.is_empty() && is_splitted)
    }

    /// One boundary of faces: merged whole or through its sections, and the context
    /// updated with the merged edges (or the wire of the sections).
    fn merge_bound(&mut self, bound: &Shape) -> Result<(), Failure> {
        let merge = self.merges_of(bound)?;
        let (is_merged, is_merged_split) = (
            !merge.with_bound.is_empty(),
            !merge.with_sections.is_empty(),
        );
        let reshape_handle = self.my_re_shape.clone();
        let reshape = live_reshape(&reshape_handle)?;
        let replace_by_wire = || -> Result<(), Failure> {
            let old = reshape.apply(bound, ShapeEnum::Shape)?;
            reshape.replace(&old, &reshape.apply(&merge.bound_wire, ShapeEnum::Shape)?)
        };
        if !is_merged && !is_merged_split {
            // Nothing was merged in this iteration: replace a previously split bound.
            if merge.is_prev_split {
                replace_by_wire()?;
            }
            return Ok(());
        }
        let mut is_splitted = (!is_merged && is_merged_split) || merge.is_prev_split;
        if is_merged && is_merged_split && !merge.is_prev_split {
            is_splitted = self.sections_win(bound, &merge)?;
        }
        if is_splitted {
            replace_by_wire()?;
            let local = live_reshape(&merge.sections_reshape)?;
            for i in 1..=merge.with_sections.extent() {
                let newedge =
                    local.apply(merge.with_sections.find_from_index(i)?, ShapeEnum::Shape)?;
                self.record_replacement(merge.with_sections.find_key(i)?, &newedge, reshape)?;
            }
            return Ok(());
        }
        for i in 1..=merge.with_bound.extent() {
            let newedge = merge.with_bound.find_from_index(i)?.clone();
            self.record_replacement(merge.with_bound.find_key(i)?, &newedge, reshape)?;
        }
        self.my_bound_sections.unbind(bound);
        if !self.my_merged_edges.contains(bound) {
            self.my_merged_edges.add(bound.clone());
        }
        Ok(())
    }

    /// The context replaces a merged section by its merged edge; the section is marked
    /// merged and its cutting sections forgotten.
    fn record_replacement(
        &mut self,
        oldedge: &Shape,
        newedge: &Shape,
        reshape: &dyn ReShape,
    ) -> Result<(), Failure> {
        replace_edge(&reshape.apply(oldedge, ShapeEnum::Shape)?, newedge, reshape)?;
        self.my_merged_edges.add(oldedge.clone());
        self.my_bound_sections.unbind(oldedge);
        Ok(())
    }
}
