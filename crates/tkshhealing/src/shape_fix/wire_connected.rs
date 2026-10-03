use super::{ShapeFixWire, wire_small::select_edge_index};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_build::{ShapeBuildEdge, ShapeBuildVertex};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{self, Builder, Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};

struct ConnectedPair<'a> {
    wire_data: &'a Handle<crate::shape_extend::ShapeExtendWireData>,
    previous_index: i32,
    current_index: i32,
    previous: &'a Edge,
    current: &'a Edge,
}

impl ShapeFixWire {
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

    /// Repairs every adjacent pair and refreshes context-replaced predecessors in native order.
    /// Example: `fixer.fix_connected_all(1.0e-7)?`.
    pub fn fix_connected_all(&self, precision: f64) -> Result<bool, Failure> {
        self.state.write().connected_status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        let Some(analyzer) = self.loaded_wire_analyzer()? else {
            return Ok(false);
        };
        let stop = if self.closed_mode() { 0 } else { 1 };
        for index in ((stop + 1)..=analyzer.nb_edges()).rev() {
            self.fix_connected_at(index, precision, false)?;
            self.accumulate_connected_status();
            self.refresh_next_edge(&analyzer, index, stop)?;
        }
        if !self.context().is_null() {
            self.update_wire()?;
        }
        Ok(self.status_connected(ShapeExtendStatus::Done))
    }

    pub(super) fn fix_connected_after_removal(
        &self,
        index: i32,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let count = self.wire_data()?.nb_edges();
        let next = if index <= count { index } else { 1 };
        let saved = self.state.read().last_fix_status;
        self.fix_connected_at(next, tolerance, true)?;
        let follow_up = if self.last_fix_status(ShapeExtendStatus::Fail) {
            ShapeExtend::encode_status(ShapeExtendStatus::Fail3)
        } else {
            0
        };
        self.state.write().last_fix_status = saved | follow_up;
        Ok(())
    }

    fn copy_connected_failure(&self, analyzer: &ShapeAnalysisWire) {
        if analyzer.last_check_status(ShapeExtendStatus::Fail) {
            self.set_last_fix_status(ShapeExtendStatus::Fail1);
        }
    }

    fn accumulate_connected_status(&self) {
        let last_status = self.state.read().last_fix_status;
        self.state.write().connected_status |= last_status;
    }

    fn refresh_next_edge(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        stop: i32,
    ) -> Result<(), Failure> {
        let context = self.context();
        if context.is_null() || index - 1 <= stop {
            return Ok(());
        }
        let next = if index - 1 > 1 {
            index - 2
        } else {
            analyzer.nb_edges()
        };
        if next == index || next == index - 1 {
            return Ok(());
        }
        let wire_data = analyzer.wire_data();
        let previous = wire_data.edge(next)?;
        let context = context.get().ok_or_else(null_context)?;
        let previous_shape = previous.clone().into_shape();
        let replacement = ReShape::apply(
            context,
            &previous_shape,
            rustycad_tkg3d::top_abs::ShapeEnum::Shape,
        )?;
        if replacement.is_null() || replacement.is_same(&previous.clone().into_shape()) {
            return Ok(());
        }
        if replacement.shape_type()? != rustycad_tkg3d::top_abs::ShapeEnum::Edge {
            return Ok(());
        }
        wire_data.set(&topods::edge(&replacement)?, next)
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

fn null_context() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_Wire::FixConnected: null context",
    )
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

pub(super) fn connected_tolerance(fixer: &ShapeFixWire, precision: f64) -> f64 {
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
