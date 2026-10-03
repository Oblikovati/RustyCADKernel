// Rust translation of ShapeFix_ComposeShell::SplitWire at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{ComposeShellFields, ShapeFixComposeShell, grid};
use crate::shape_analysis::ShapeAnalysisSurface;
mod context;
use crate::shape_fix::{FixRoot, ShapeFixWireSegment, WirePatchIndex};
pub(super) use context::apply_context;
use context::distribute_split_points;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

pub(super) struct SplitWireInput<'a> {
    pub(super) wire: &'a mut ShapeFixWireSegment,
    pub(super) indices: &'a mut Sequence<i32>,
    pub(super) values: &'a Sequence<f64>,
    pub(super) vertices: &'a mut Sequence<Shape>,
    pub(super) segment_codes: &'a Sequence<i32>,
    pub(super) cut_by_u: bool,
    pub(super) cut_index: i32,
}

pub(super) fn split_wire(
    composer: &ShapeFixComposeShell,
    input: SplitWireInput<'_>,
) -> Result<ShapeFixWireSegment, Failure> {
    let mut state = SplitWireState::new(composer, input)?;
    state.run()?;
    Ok(state.result)
}

struct SplitWireState<'a> {
    composer: &'a ShapeFixComposeShell,
    fields: ComposeShellFields,
    surface_tool: ShapeAnalysisSurface,
    wire: &'a mut ShapeFixWireSegment,
    indices: &'a mut Sequence<i32>,
    values: &'a Sequence<f64>,
    vertices: &'a mut Sequence<Shape>,
    codes: &'a Sequence<i32>,
    cut_by_u: bool,
    cut_index: i32,
    start: i32,
    result: ShapeFixWireSegment,
}

pub(super) struct SplitEdgeWork {
    edge: Edge,
    patch: WirePatchIndex,
    state: EdgeSplitState,
    stop: i32,
    initial_edge_count: i32,
    split: bool,
    code: i32,
    cut_by_u: bool,
    wire_orientation: Orientation,
}

impl<'a> SplitWireState<'a> {
    fn new(composer: &'a ShapeFixComposeShell, input: SplitWireInput<'a>) -> Result<Self, Failure> {
        let fields = composer.fields.read().clone();
        composer.context().get().ok_or_else(null_context)?;
        let surface_tool =
            ShapeAnalysisSurface::new(&rustycad_tkbrep::brep::BRepTool::surface(&fields.face)?)?;
        Ok(Self {
            composer,
            fields,
            surface_tool,
            wire: input.wire,
            indices: input.indices,
            values: input.values,
            vertices: input.vertices,
            codes: input.segment_codes,
            cut_by_u: input.cut_by_u,
            cut_index: input.cut_index,
            start: 1,
            result: ShapeFixWireSegment::new(),
        })
    }

    fn run(&mut self) -> Result<(), Failure> {
        self.result.set_orientation(self.wire.orientation());
        let mut edge_index = 1;
        while edge_index <= self.wire.nb_edges() {
            self.process_edge(edge_index)?;
            edge_index += 1;
        }
        self.result.set_orientation(self.wire.orientation());
        Ok(())
    }

    fn process_edge(&mut self, edge_index: i32) -> Result<(), Failure> {
        let replaced_count = apply_context(self.wire, edge_index, &self.composer.context())?;
        self.distribute_if_replaced(edge_index, replaced_count)?;
        let edge = self.wire.edge(edge_index)?;
        self.process_edge_geometry(edge_index, edge)
    }

    fn distribute_if_replaced(&mut self, index: i32, count: i32) -> Result<(), Failure> {
        if count == 1 {
            return Ok(());
        }
        distribute_split_points(
            self.wire,
            self.fields.face.clone(),
            index,
            count,
            self.indices,
            self.values,
        )?;
        if count <= 0 {
            // Native SplitWire retries this dismissed edge and the pinned probe crashes.
            return Err(not_implemented(
                "SplitWire cannot process a removed source edge",
            ));
        }
        Ok(())
    }

    fn process_edge_geometry(&mut self, index: i32, edge: Edge) -> Result<(), Failure> {
        let patch = self.wire.patch_index(index)?;
        let code = self.segment_code(self.start.saturating_sub(1))?;
        let stop = self.stop_for_edge(index)?;
        if stop == self.start {
            self.append_unsplit_edge(&edge, patch, code)?;
            return Ok(());
        }
        let edge_state =
            EdgeSplitState::new(self.composer, &self.fields, &self.surface_tool, &edge)?;
        self.split_edge_at_parameters(edge, patch, edge_state, stop)
    }

    fn stop_for_edge(&self, index: i32) -> Result<i32, Failure> {
        let mut stop = self.start;
        while stop <= self.indices.length() && self.index_at(stop)? == index {
            stop += 1;
        }
        Ok(stop)
    }

    fn index_at(&self, index: i32) -> Result<i32, Failure> {
        self.indices.value(index as usize).copied()
    }

    fn segment_code(&self, index: i32) -> Result<i32, Failure> {
        let position = if index == 0 {
            self.codes.length()
        } else {
            index
        };
        self.codes.value(position as usize).copied()
    }

    fn append_unsplit_edge(
        &self,
        edge: &Edge,
        patch: WirePatchIndex,
        code: i32,
    ) -> Result<(), Failure> {
        self.result.add_edge_with_patch(0, edge, patch)?;
        if code != 0 || self.wire.orientation() != Orientation::External {
            return grid::define_patch(&self.result, code, self.cut_by_u, self.cut_index, None);
        }
        Ok(())
    }

    fn split_edge_at_parameters(
        &mut self,
        edge: Edge,
        patch: WirePatchIndex,
        state: EdgeSplitState,
        stop: i32,
    ) -> Result<(), Failure> {
        let mut work = SplitEdgeWork {
            edge,
            patch,
            state,
            stop,
            initial_edge_count: self.result.nb_edges(),
            split: false,
            code: self.segment_code(self.start.saturating_sub(1))?,
            cut_by_u: self.cut_by_u,
            wire_orientation: self.wire.orientation(),
        };
        self.run_split_points(&mut work)?;
        self.start = stop;
        self.finish_edge(&work)
    }

    fn run_split_points(&mut self, work: &mut SplitEdgeWork) -> Result<(), Failure> {
        let mut point = self.start;
        while point <= work.stop && self.split_point(work, point)? {
            point += 1;
        }
        Ok(())
    }

    fn split_point(&mut self, work: &mut SplitEdgeWork, point_index: i32) -> Result<bool, Failure> {
        if !work.split && point_index >= work.stop {
            return Ok(false);
        }
        let parameter = self.parameter_for_point(&work.state, point_index, work.stop)?;
        work.state.current_parameter = parameter;
        let cut = self.choose_cut(&work.state, parameter)?;
        self.update_split_code(point_index, &mut work.code)?;
        self.apply_cut_action(work, point_index, cut)
    }

    fn choose_cut(&self, state: &EdgeSplitState, parameter: f64) -> Result<CutVertex, Failure> {
        choose_cut_vertex(
            state,
            parameter,
            self.fields.grid.clone(),
            self.fields.location.clone(),
            self.composer,
            self.cut_by_u,
            self.cut_index,
        )
    }

    fn update_split_code(&self, point: i32, code: &mut i32) -> Result<(), Failure> {
        if point > self.start {
            *code = self.segment_code(point - 1)?;
        }
        Ok(())
    }

    fn apply_cut_action(
        &mut self,
        work: &mut SplitEdgeWork,
        point: i32,
        cut: CutVertex,
    ) -> Result<bool, Failure> {
        match cut.action {
            CutAction::Skip => self.skip_cut(work, point),
            CutAction::Finish => self.finish_cut(work, point, cut),
            CutAction::Split => {
                self.append_cut_vertex(work, &cut.vertex, &cut.point, cut.parameter, point)?;
                Ok(true)
            }
        }
    }

    fn skip_cut(&mut self, work: &mut SplitEdgeWork, point: i32) -> Result<bool, Failure> {
        self.vertices
            .append(work.state.first_vertex.clone().into_shape());
        work.code = self.segment_code(point)?;
        Ok(true)
    }

    fn finish_cut(
        &mut self,
        work: &mut SplitEdgeWork,
        point: i32,
        cut: CutVertex,
    ) -> Result<bool, Failure> {
        self.append_remaining_endpoint_vertices(&work.state, point, work.stop);
        if !work.split {
            return Ok(false);
        }
        self.append_split_edge(work, &cut.vertex, work.state.last_parameter, point)?;
        Ok(false)
    }

    fn append_remaining_endpoint_vertices(
        &mut self,
        state: &EdgeSplitState,
        point: i32,
        stop: i32,
    ) {
        for _ in point..stop {
            self.vertices.append(state.last_vertex.clone().into_shape());
        }
    }

    fn parameter_for_point(
        &self,
        state: &EdgeSplitState,
        index: i32,
        stop: i32,
    ) -> Result<f64, Failure> {
        if index < stop {
            let parameter = self.values.value(index as usize).copied()?;
            return Ok(adjust_periodic_parameter(parameter, state));
        }
        Ok(state.last_parameter)
    }

    fn append_cut_vertex(
        &mut self,
        work: &mut SplitEdgeWork,
        vertex: &Vertex,
        point: &Point3,
        parameter: f64,
        point_index: i32,
    ) -> Result<(), Failure> {
        let vertex = self.copied_or_new_vertex(vertex, point, &work.state)?;
        self.vertices.append(vertex.clone().into_shape());
        self.append_split_edge(work, &vertex, parameter, point_index)
    }

    fn copied_or_new_vertex(
        &self,
        vertex: &Vertex,
        point: &Point3,
        state: &EdgeSplitState,
    ) -> Result<Vertex, Failure> {
        if !vertex.is_null() {
            return Ok(vertex.clone());
        }
        make_cut_vertex(
            point.transformed(self.fields.location.transformation()),
            state.edge_tolerance,
        )
    }

    fn append_split_edge(
        &mut self,
        work: &mut SplitEdgeWork,
        vertex: &Vertex,
        parameter: f64,
        point_index: i32,
    ) -> Result<(), Failure> {
        let edge = build_split_edge(self, work, vertex, parameter, point_index)?;
        self.result.add_edge_with_patch(0, &edge, work.patch)?;
        grid::define_patch(&self.result, work.code, self.cut_by_u, self.cut_index, None)
    }

    fn finish_edge(&mut self, work: &SplitEdgeWork) -> Result<(), Failure> {
        if work.split {
            let wire = result_edges_as_wire(
                &self.result,
                work.initial_edge_count,
                work.edge.orientation(),
            )?;
            self.composer
                .context()
                .get()
                .ok_or_else(null_context)?
                .replace(&work.edge.clone().into_shape(), &wire.into_shape())?;
            return Ok(());
        }
        self.append_unsplit_after_snap(&work.edge, work.patch, &work.state, work.code)
    }

    fn append_unsplit_after_snap(
        &mut self,
        edge: &Edge,
        patch: WirePatchIndex,
        edge_state: &EdgeSplitState,
        mut code: i32,
    ) -> Result<(), Failure> {
        let edge = self.internal_external_replacement(edge, code, edge_state)?;
        if code == 0 && self.wire.orientation() == Orientation::External {
            code = infer_external_code_by_parameter(
                self.cut_by_u,
                edge_state.first_parameter,
                edge_state.last_parameter,
                edge_state.current_parameter,
            );
        }
        self.result.add_edge_with_patch(0, &edge, patch)?;
        grid::define_patch(&self.result, code, self.cut_by_u, self.cut_index, None)
    }

    fn internal_external_replacement(
        &self,
        edge: &Edge,
        code: i32,
        edge_state: &EdgeSplitState,
    ) -> Result<Edge, Failure> {
        if self.wire.orientation() != Orientation::Internal || code != 0 {
            return Ok(edge.clone());
        }
        copy_as_external_edge(
            edge,
            edge_state,
            &self.fields.face,
            &self.composer.context(),
        )
    }
}

fn null_context() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitWire: null context",
    )
}

fn null_pcurve() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::SplitWire: null p-curve",
    )
}

fn not_implemented(detail: &str) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!("ShapeFix_ComposeShell::SplitWire: {detail}"),
    )
}

mod edge_build;
mod edge_projection;
mod edge_snap;
mod edge_state;
use edge_build::{
    build_split_edge, copy_as_external_edge, infer_external_code_by_parameter, result_edges_as_wire,
};
use edge_projection::make_cut_vertex;
use edge_snap::{CutAction, CutVertex, adjust_periodic_parameter, choose_cut_vertex};
use edge_state::EdgeSplitState;
