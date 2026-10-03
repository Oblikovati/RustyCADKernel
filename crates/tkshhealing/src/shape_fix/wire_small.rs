use super::ShapeFixWire;
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_build::{ShapeBuildEdge, ShapeBuildVertex};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Builder, Edge, Vertex};
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

struct ConnectedPair<'a> {
    wire_data: &'a Handle<crate::shape_extend::ShapeExtendWireData>,
    previous_index: i32,
    current_index: i32,
    previous: &'a Edge,
    current: &'a Edge,
}

impl ShapeFixWire {
    /// Removes one small edge and repairs its adjacent connection when needed.
    /// Example: `fixer.fix_small_at(1, false, 1.0e-7)?`.
    pub fn fix_small_at(
        &self,
        index: i32,
        lock_vertex: bool,
        small_tolerance: f64,
    ) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let Some(analyzer) = self.ready_small_analyzer()? else {
            return Ok(false);
        };
        let selected = select_edge_index(index, analyzer.nb_edges());
        analyzer.check_small_at(selected, small_tolerance)?;
        self.copy_small_failure(&analyzer);
        if !analyzer.last_check_status(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        self.remove_small_edge(&analyzer, selected, lock_vertex, small_tolerance)
    }

    /// Removes small edges in reverse order and accumulates native status flags.
    /// Example: `fixer.fix_small_all(false, 1.0e-7)?`.
    pub fn fix_small_all(&self, lock_vertex: bool, small_tolerance: f64) -> Result<bool, Failure> {
        self.state.write().small_status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        let Some(analyzer) = self.loaded_wire_analyzer()? else {
            return Ok(false);
        };
        for index in (1..=analyzer.nb_edges()).rev() {
            self.fix_small_at(index, lock_vertex, small_tolerance)?;
            self.accumulate_small_status();
        }
        Ok(self.status_small(ShapeExtendStatus::Done))
    }

    /// Repairs one adjacent vertex pair using the analyzer's last CheckConnected result.
    /// Example: `fixer.fix_connected_at(1, 1.0e-7, true)?`.
    pub fn fix_connected_at(
        &self,
        index: i32,
        precision: f64,
        update_wire: bool,
    ) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let Some(analyzer) = self.loaded_wire_analyzer()? else {
            return Ok(false);
        };
        let tolerance = connected_tolerance(self, precision);
        analyzer.check_connected_at(index, tolerance)?;
        self.copy_connected_failure(&analyzer);
        if !analyzer.last_check_status(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        self.replace_connected_vertices(&analyzer, index, update_wire)
    }

    fn ready_small_analyzer(&self) -> Result<Option<Handle<ShapeAnalysisWire>>, Failure> {
        let Some(analyzer) = self.loaded_wire_analyzer()? else {
            return Ok(None);
        };
        if analyzer.nb_edges() <= 1 {
            return Ok(None);
        }
        Ok(Some(analyzer))
    }

    fn loaded_wire_analyzer(&self) -> Result<Option<Handle<ShapeAnalysisWire>>, Failure> {
        let analyzer = self.state.read().analyzer.clone();
        if analyzer.is_null() || !analyzer.is_loaded() || analyzer.nb_edges() <= 0 {
            return Ok(None);
        }
        Ok(Some(analyzer))
    }

    fn remove_small_edge(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        lock_vertex: bool,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let done2 = analyzer.last_check_status(ShapeExtendStatus::Done2);
        if done2 && (lock_vertex || !self.topology_mode()) {
            self.or_last_fix_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        let wire_data = analyzer.wire_data();
        let edge = wire_data.edge(index)?;
        self.remove_edge_from_context(&edge)?;
        self.warn_small_edge(&edge);
        wire_data.remove(index)?;
        self.or_last_fix_status(if done2 {
            ShapeExtendStatus::Done2
        } else {
            ShapeExtendStatus::Done1
        });
        if done2 {
            self.fix_connected_after_removal(index, tolerance)?;
        }
        Ok(true)
    }

    fn remove_edge_from_context(&self, edge: &Edge) -> Result<(), Failure> {
        if let Some(context) = self.context().get() {
            ReShape::remove(context, &edge.clone().into_shape())?;
        }
        Ok(())
    }

    fn warn_small_edge(&self, edge: &Edge) {
        let key = AsciiString::from("FixAdvWire.FixSmall.MSG0");
        self.send_warning(&edge.clone().into_shape(), &Msg::from_key(&key));
    }

    fn fix_connected_after_removal(&self, index: i32, tolerance: f64) -> Result<(), Failure> {
        let count = self.wire_data()?.nb_edges();
        let next = if index <= count { index } else { 1 };
        let saved = self.state.read().last_fix_status;
        self.fix_connected_at(next, tolerance, true)?;
        let failure = self.last_fix_status(ShapeExtendStatus::Fail);
        let follow_up = if failure {
            ShapeExtend::encode_status(ShapeExtendStatus::Fail3)
        } else {
            0
        };
        self.state.write().last_fix_status = saved | follow_up;
        Ok(())
    }

    fn copy_small_failure(&self, analyzer: &ShapeAnalysisWire) {
        if analyzer.last_check_status(ShapeExtendStatus::Fail) {
            self.set_last_fix_status(ShapeExtendStatus::Fail1);
        }
    }

    fn copy_connected_failure(&self, analyzer: &ShapeAnalysisWire) {
        if analyzer.last_check_status(ShapeExtendStatus::Fail) {
            self.set_last_fix_status(ShapeExtendStatus::Fail1);
        }
    }

    fn accumulate_small_status(&self) {
        let last_status = self.state.read().last_fix_status;
        self.state.write().small_status |= last_status;
    }

    fn replace_connected_vertices(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        update_wire: bool,
    ) -> Result<bool, Failure> {
        let (wire_data, previous_index, current_index) = adjacent_edge_indices(analyzer, index)?;
        let previous = wire_data.edge(previous_index)?;
        let current = wire_data.edge(current_index)?;
        let vertex = self.connected_vertex(analyzer, &previous, &current)?;
        let pair = ConnectedPair {
            wire_data: &wire_data,
            previous_index,
            current_index,
            previous: &previous,
            current: &current,
        };
        self.rebuild_connected_edges(analyzer, pair, &vertex)?;
        if update_wire && !self.context().is_null() {
            self.update_wire()?;
        }
        Ok(true)
    }

    fn connected_vertex(
        &self,
        analyzer: &ShapeAnalysisWire,
        previous: &Edge,
        current: &Edge,
    ) -> Result<Vertex, Failure> {
        let edge_analysis = ShapeAnalysisEdge::new();
        let old_previous = edge_analysis.last_vertex(previous);
        let old_current = edge_analysis.first_vertex(current);
        if analyzer.last_check_status(ShapeExtendStatus::Done1) {
            return self.choose_confused_vertex(
                &old_previous,
                &old_current,
                current,
                &edge_analysis,
            );
        }
        let status = if analyzer.last_check_status(ShapeExtendStatus::Done2) {
            ShapeExtendStatus::Done2
        } else {
            ShapeExtendStatus::Done3
        };
        self.or_last_fix_status(status);
        let combined = ShapeBuildVertex.combine_vertices(&old_previous, &old_current, 1.0001)?;
        self.replace_context_vertex(&old_previous, &combined)?;
        self.replace_context_vertex(&old_current, &combined)?;
        Ok(combined)
    }

    fn choose_confused_vertex(
        &self,
        previous: &Vertex,
        current: &Vertex,
        edge: &Edge,
        analysis: &ShapeAnalysisEdge,
    ) -> Result<Vertex, Failure> {
        self.or_last_fix_status(ShapeExtendStatus::Done1);
        let current_last = analysis.last_vertex(edge);
        if current.is_same(&current_last) {
            self.replace_context_vertex(previous, current)?;
            return Ok(current.clone());
        }
        self.replace_context_vertex(current, previous)?;
        Ok(previous.clone())
    }

    fn replace_context_vertex(&self, old: &Vertex, new: &Vertex) -> Result<(), Failure> {
        if let Some(context) = self.context().get() {
            let replacement = new.clone().into_shape().oriented(old.orientation());
            ReShape::replace(context, &old.clone().into_shape(), &replacement)?;
        }
        Ok(())
    }

    fn rebuild_connected_edges(
        &self,
        analyzer: &ShapeAnalysisWire,
        pair: ConnectedPair<'_>,
        vertex: &Vertex,
    ) -> Result<(), Failure> {
        if pair.wire_data.nb_edges() < 2 {
            return self.rebuild_single_edge(pair, vertex);
        }
        if self.topology_mode() && is_free(pair.previous) && is_free(pair.current) {
            return self.edit_connected_edges(analyzer, pair, vertex);
        }
        self.copy_connected_edges(analyzer, pair, vertex)
    }

    fn rebuild_single_edge(&self, pair: ConnectedPair<'_>, vertex: &Vertex) -> Result<(), Failure> {
        if self.topology_mode() && is_free(pair.current) {
            remove_edge_vertices(pair.current)?;
            add_edge_vertex(
                pair.current,
                vertex,
                rustycad_tkg3d::top_abs::Orientation::Forward,
            )?;
            return add_edge_vertex(
                pair.current,
                vertex,
                rustycad_tkg3d::top_abs::Orientation::Reversed,
            );
        }
        let replacement = self.copy_with_replacement(pair.current, vertex, vertex)?;
        self.store_connected_edge(
            pair.wire_data,
            pair.current,
            &replacement,
            pair.current_index,
        )
    }

    fn edit_connected_edges(
        &self,
        analyzer: &ShapeAnalysisWire,
        pair: ConnectedPair<'_>,
        vertex: &Vertex,
    ) -> Result<(), Failure> {
        let edge_analysis = ShapeAnalysisEdge::new();
        edit_edge_vertex(
            pair.current,
            &edge_analysis.first_vertex(pair.current),
            vertex,
            rustycad_tkg3d::top_abs::Orientation::Forward,
        )?;
        let same_endpoints = same_edge_vertices(pair.current);
        if should_replace_previous(analyzer, same_endpoints) {
            edit_edge_vertex(
                pair.previous,
                &edge_analysis.last_vertex(pair.previous),
                vertex,
                rustycad_tkg3d::top_abs::Orientation::Reversed,
            )?;
        }
        Ok(())
    }

    fn copy_connected_edges(
        &self,
        analyzer: &ShapeAnalysisWire,
        pair: ConnectedPair<'_>,
        vertex: &Vertex,
    ) -> Result<(), Failure> {
        let current_copy = self.copy_with_replacement(pair.current, vertex, &Vertex::default())?;
        self.store_connected_edge(
            pair.wire_data,
            pair.current,
            &current_copy,
            pair.current_index,
        )?;
        if should_replace_previous(analyzer, same_edge_vertices(pair.current)) {
            let previous_copy =
                self.copy_with_replacement(pair.previous, &Vertex::default(), vertex)?;
            self.store_connected_edge(
                pair.wire_data,
                pair.previous,
                &previous_copy,
                pair.previous_index,
            )?;
        }
        Ok(())
    }

    fn copy_with_replacement(
        &self,
        edge: &Edge,
        first: &Vertex,
        last: &Vertex,
    ) -> Result<Edge, Failure> {
        ShapeBuildEdge.copy_replace_vertices(edge, first, last)
    }

    fn store_connected_edge(
        &self,
        wire_data: &Handle<crate::shape_extend::ShapeExtendWireData>,
        old: &Edge,
        new: &Edge,
        index: i32,
    ) -> Result<(), Failure> {
        wire_data.set(new, index)?;
        if let Some(context) = self.context().get() {
            ReShape::replace(
                context,
                &old.clone().into_shape(),
                &new.clone().into_shape(),
            )?;
        }
        Ok(())
    }
}

fn should_replace_previous(analyzer: &ShapeAnalysisWire, same_endpoints: bool) -> bool {
    !analyzer.last_check_status(ShapeExtendStatus::Done1) || same_endpoints
}

fn remove_edge_vertices(edge: &Edge) -> Result<(), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let mut shape = edge.clone().into_shape();
    Builder.remove(&mut shape, &analysis.first_vertex(edge).into_shape())?;
    Builder.remove(&mut shape, &analysis.last_vertex(edge).into_shape())
}

fn add_edge_vertex(
    edge: &Edge,
    vertex: &Vertex,
    orientation: rustycad_tkg3d::top_abs::Orientation,
) -> Result<(), Failure> {
    let mut shape = edge.clone().into_shape();
    Builder.add(
        &mut shape,
        &vertex.clone().into_shape().oriented(orientation),
    )
}

fn same_edge_vertices(edge: &Edge) -> bool {
    let analysis = ShapeAnalysisEdge::new();
    analysis
        .first_vertex(edge)
        .is_same(&analysis.last_vertex(edge))
}

fn is_free(edge: &Edge) -> bool {
    edge.clone()
        .into_shape()
        .tshape()
        .get()
        .is_some_and(|shape| shape.free())
}

fn edit_edge_vertex(
    edge: &Edge,
    old_vertex: &Vertex,
    new_vertex: &Vertex,
    orientation: rustycad_tkg3d::top_abs::Orientation,
) -> Result<(), Failure> {
    let mut shape = edge.clone().into_shape();
    Builder.remove(&mut shape, &old_vertex.clone().into_shape())?;
    Builder.add(
        &mut shape,
        &new_vertex.clone().into_shape().oriented(orientation),
    )
}

fn select_edge_index(index: i32, count: i32) -> i32 {
    if index > 0 { index } else { count }
}

fn connected_tolerance(fixer: &ShapeFixWire, precision: f64) -> f64 {
    if precision < 0.0 {
        fixer.max_tolerance()
    } else {
        precision
    }
}

fn adjacent_edge_indices(
    analyzer: &ShapeAnalysisWire,
    index: i32,
) -> Result<(Handle<crate::shape_extend::ShapeExtendWireData>, i32, i32), Failure> {
    let wire_data = analyzer.wire_data();
    let current = select_edge_index(index, wire_data.nb_edges());
    let previous = if current > 1 {
        current - 1
    } else {
        wire_data.nb_edges()
    };
    Ok((wire_data, previous, current))
}

#[cfg(test)]
mod tests {
    use super::{connected_tolerance, select_edge_index};
    use crate::shape_fix::FixRoot;

    #[test]
    fn nonpositive_native_indices_select_the_last_edge() {
        assert_eq!(select_edge_index(0, 4), 4);
        assert_eq!(select_edge_index(-1, 4), 4);
        assert_eq!(select_edge_index(2, 4), 2);
    }

    #[test]
    fn negative_connected_precision_uses_root_maximum() {
        let fixer = super::ShapeFixWire::default();
        fixer.set_max_tolerance(0.75);
        assert_eq!(connected_tolerance(&fixer, -1.0), 0.75);
        assert_eq!(connected_tolerance(&fixer, 0.25), 0.25);
    }
}
