//! Context and neighbor updates for OCCT's long-edge `FixLacking` branch.
use super::super::{FixRoot, ShapeFixWire};
use super::types::LackingPair;
use crate::shape_build::ShapeBuildEdge;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::Failure;

impl ShapeFixWire {
    pub(super) fn replace_long_neighbors(
        &self,
        pair: &LackingPair,
        new_first: &Vertex,
        new_last: &Vertex,
    ) -> Result<(), Failure> {
        let first_replacement = self.replace_first_long_neighbor(pair, new_first, new_last)?;
        let second_replacement = self.replace_second_long_neighbor(pair, new_last)?;
        self.record_long_replacements(
            pair,
            new_first,
            new_last,
            &first_replacement,
            second_replacement.as_ref(),
        )
    }

    fn replace_first_long_neighbor(
        &self,
        pair: &LackingPair,
        new_first: &Vertex,
        new_last: &Vertex,
    ) -> Result<Edge, Failure> {
        let same_edge = pair.previous_index == pair.current_index;
        let last = if same_edge {
            new_last
        } else {
            &Vertex::default()
        };
        let replacement = ShapeBuildEdge.copy_replace_vertices(&pair.previous, last, new_first)?;
        self.set_lacking_wire_edge(pair, &replacement, pair.previous_index)?;
        Ok(replacement)
    }

    fn replace_second_long_neighbor(
        &self,
        pair: &LackingPair,
        new_last: &Vertex,
    ) -> Result<Option<Edge>, Failure> {
        if pair.previous_index == pair.current_index {
            return Ok(None);
        }
        let replacement =
            ShapeBuildEdge.copy_replace_vertices(&pair.current, new_last, &Vertex::default())?;
        self.set_lacking_wire_edge(pair, &replacement, pair.current_index)?;
        Ok(Some(replacement))
    }

    fn set_lacking_wire_edge(
        &self,
        pair: &LackingPair,
        edge: &Edge,
        index: i32,
    ) -> Result<(), Failure> {
        pair.wire
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire lost wire data"))?
            .set(edge, index)
    }

    fn record_long_replacements(
        &self,
        pair: &LackingPair,
        new_first: &Vertex,
        new_last: &Vertex,
        first_replacement: &Edge,
        second_replacement: Option<&Edge>,
    ) -> Result<(), Failure> {
        let context = self.context();
        if context.is_null() {
            return Ok(());
        }
        let reshape = context
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire context is null"))?;
        self.record_long_edge_context(reshape, pair, first_replacement, second_replacement)?;
        self.record_long_vertex_context(reshape, pair, new_first, new_last)?;
        self.update_wire()?;
        Ok(())
    }

    fn record_long_edge_context(
        &self,
        reshape: &crate::shape_build::ShapeBuildReShape,
        pair: &LackingPair,
        first: &Edge,
        second: Option<&Edge>,
    ) -> Result<(), Failure> {
        ReShape::replace(
            reshape,
            &pair.previous.clone().into_shape(),
            &first.clone().into_shape(),
        )?;
        if let Some(second) = second {
            ReShape::replace(
                reshape,
                &pair.current.clone().into_shape(),
                &second.clone().into_shape(),
            )?;
        }
        Ok(())
    }

    fn record_long_vertex_context(
        &self,
        reshape: &crate::shape_build::ShapeBuildReShape,
        pair: &LackingPair,
        new_first: &Vertex,
        new_last: &Vertex,
    ) -> Result<(), Failure> {
        ReShape::replace(
            reshape,
            &pair.first_vertex.clone().into_shape(),
            &new_first.oriented(pair.first_vertex.orientation()),
        )?;
        if pair.first_vertex.is_same(&pair.last_vertex) {
            return Ok(());
        }
        ReShape::replace(
            reshape,
            &pair.last_vertex.clone().into_shape(),
            &new_last.oriented(pair.last_vertex.orientation()),
        )
    }
}
