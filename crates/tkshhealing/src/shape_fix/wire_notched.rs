//! Aggregate `ShapeFix_Wire::FixNotchedEdges` splitting and seam removal.
use super::ShapeFixWire;
use crate::shape_analysis::{
    ShapeAnalysisEdge, ShapeAnalysisTransferParametersProj, TransferParameters,
};
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{self, Builder, Edge, Vertex, Wire};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::top_abs::Orientation;

impl ShapeFixWire {
    /// Splits overlapped adjacent face p-curves; `let changed = fixer.fix_notched_edges()?`.
    pub fn fix_notched_edges(&self) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        self.update_wire_if_context()?;
        let mut index = 1;
        while index <= self.nb_edges() && self.nb_edges() > 2 {
            match self.fix_notch_at(index)? {
                NotchAction::Previous => index -= 1,
                NotchAction::Current => {}
                NotchAction::Next => index += 1,
            }
        }
        self.set_notches_status_from_last_fix();
        Ok(self.last_fix_status(ShapeExtendStatus::Done))
    }

    fn fix_notch_at(&self, index: i32) -> Result<NotchAction, Failure> {
        let (mut to_remove, mut parameter) = (0, 0.0);
        if !self.working_analyzer().check_notched_edges(
            index,
            &mut to_remove,
            &mut parameter,
            self.min_tolerance(),
        )? {
            return Ok(NotchAction::Next);
        }
        self.apply_notch(index, to_remove, parameter)
    }

    fn apply_notch(
        &self,
        index: i32,
        to_remove: i32,
        parameter: f64,
    ) -> Result<NotchAction, Failure> {
        let count = self.nb_edges();
        let next = index.max(1);
        let previous = if next > 1 { next - 1 } else { count };
        let remove_first = previous == to_remove;
        let split_index = if next == to_remove { previous } else { next };
        let edge = self.wire_data()?.edge(split_index)?;
        let (curve, first, last) =
            ShapeAnalysisEdge::new().pcurve_on_face(&edge, &self.face(), true)?;
        if self.notch_is_whole_edge(&edge, remove_first, first, last, parameter)? {
            self.fix_dummy_seam(previous)?;
            self.or_last_fix_status(ShapeExtendStatus::Done1);
            return Ok(NotchAction::Previous);
        }
        if (if remove_first { first } else { last } - parameter).abs() < precision::p_confusion() {
            return Ok(NotchAction::Next);
        }
        let curve = curve
            .get()
            .ok_or_else(|| Failure::new("null p-curve for notched edge"))?;
        self.split_notched_edge(NotchSplit {
            edge: &edge,
            curve,
            range: (first, last),
            parameter,
            split_index,
            to_remove,
            previous,
            next,
            count,
        })?;
        self.or_last_fix_status(ShapeExtendStatus::Done1);
        Ok(NotchAction::Current)
    }

    fn notch_is_whole_edge(
        &self,
        edge: &Edge,
        remove_first: bool,
        first: f64,
        last: f64,
        parameter: f64,
    ) -> Result<bool, Failure> {
        let endpoint = if remove_first { last } else { first };
        if (parameter - endpoint).abs() <= precision::p_confusion() {
            return Ok(true);
        }
        if !ShapeAnalysisEdge::new().is_closed_3d(edge)? {
            return Ok(false);
        }
        let other = if remove_first { first } else { last };
        Ok((parameter - other).abs() <= precision::p_confusion())
    }

    fn split_notched_edge(&self, plan: NotchSplit<'_>) -> Result<(), Failure> {
        let NotchSplit {
            edge,
            curve,
            range,
            parameter,
            split_index,
            to_remove,
            previous,
            next,
            count,
        } = plan;
        let transfer = ShapeAnalysisTransferParametersProj::new();
        transfer.set_max_tolerance(self.max_tolerance());
        transfer.init(edge, &self.face())?;
        let point = self
            .working_analyzer()
            .surface()
            .value_uv(curve.value(parameter)?)?;
        let mut vertex = Vertex::default();
        BRepBuilder::default().make_vertex_at(&mut vertex, point, precision::confusion())?;
        let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
        let pieces = split_edge_parts(&forward, &vertex, range, parameter, &transfer)?;
        self.replace_notched_context(&forward, &pieces)?;
        let pieces = orient_parts(pieces, edge.orientation());
        self.insert_notched_parts(split_index, count, &pieces)?;
        let remove_at = if previous == count && next == 1 {
            count
        } else {
            to_remove
        };
        self.fix_dummy_seam(remove_at)?;
        self.update_wire_if_context()?;
        self.or_last_fix_status(ShapeExtendStatus::Done2);
        Ok(())
    }

    fn replace_notched_context(&self, edge: &Edge, pieces: &[Edge; 2]) -> Result<(), Failure> {
        let context = self.context();
        if context.is_null() {
            return Ok(());
        }
        let mut wire = Wire::default();
        Builder.make_wire(&mut wire);
        let mut shape = wire.into_shape();
        for piece in pieces {
            Builder.add(&mut shape, &piece.clone().into_shape())?;
        }
        let wire = topods::wire(&shape)?;
        context.replace(&edge.clone().into_shape(), &wire.into_shape())
    }

    fn insert_notched_parts(
        &self,
        split_index: i32,
        count: i32,
        pieces: &[Edge; 2],
    ) -> Result<(), Failure> {
        let data = self.wire_data()?;
        let append = split_index == count;
        data.set(&pieces[0], split_index)?;
        data.add_edge(&pieces[1], if append { 0 } else { split_index + 1 })?;
        Ok(())
    }
}

enum NotchAction {
    Previous,
    Current,
    Next,
}

struct NotchSplit<'a> {
    edge: &'a Edge,
    curve: &'a dyn rustycad_tkg2d::geom2d::Geom2dCurve,
    range: (f64, f64),
    parameter: f64,
    split_index: i32,
    to_remove: i32,
    previous: i32,
    next: i32,
    count: i32,
}

fn split_edge_parts(
    edge: &Edge,
    vertex: &Vertex,
    (first, last): (f64, f64),
    parameter: f64,
    transfer: &ShapeAnalysisTransferParametersProj,
) -> Result<[Edge; 2], Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let mut cut_start = Vertex::unchecked(vertex.oriented(Orientation::Reversed));
    let first_piece =
        ShapeBuildEdge.copy_replace_vertices(edge, &analysis.first_vertex(edge), &cut_start)?;
    ShapeBuildEdge.copy_pcurves(&first_piece, edge)?;
    transfer.transfer_range(&first_piece, first.min(last), parameter, true)?;
    let cut_end = Vertex::unchecked(vertex.oriented(Orientation::Forward));
    let second_piece =
        ShapeBuildEdge.copy_replace_vertices(edge, &cut_end, &analysis.last_vertex(edge))?;
    ShapeBuildEdge.copy_pcurves(&second_piece, edge)?;
    transfer.transfer_range(&second_piece, parameter, first.max(last), true)?;
    let builder = BRepBuilder::default();
    builder.same_range(&first_piece, false)?;
    builder.same_parameter(&first_piece, false)?;
    builder.same_range(&second_piece, false)?;
    builder.same_parameter(&second_piece, false)?;
    cut_start.set_orientation(Orientation::Reversed);
    Ok([first_piece, second_piece])
}

fn orient_parts(mut pieces: [Edge; 2], orientation: Orientation) -> [Edge; 2] {
    pieces[0].set_orientation(orientation);
    pieces[1].set_orientation(orientation);
    if orientation == Orientation::Reversed {
        pieces.swap(0, 1);
    }
    pieces
}
