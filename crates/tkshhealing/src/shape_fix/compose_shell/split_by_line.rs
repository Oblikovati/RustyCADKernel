// Rust translation of ShapeFix_ComposeShell::SplitByLine at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::split_wire::{self, SplitWireInput};
use super::{ShapeFixComposeShell, geometry};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisSurface};
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::{FixRoot, ShapeFixWireSegment};
mod closed_mode;
mod helpers;
mod pcurve_scan;
use helpers::{
    ITP_TANG, JunctionState, SplitArguments, TOLINT, edge_end_parameter, edge_start_parameter,
    null_context, null_wire,
};
use pcurve_scan::EdgePcurveScan;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::{geom2d::Geom2dLine, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::{geom2d_int::GInter, int_res2d::Position};
use rustycad_tkmath::gp::Line2;

pub(super) struct SplitByLineInput<'a> {
    pub(super) wire: &'a mut ShapeFixWireSegment,
    pub(super) line: Line2,
    pub(super) cut_by_u: bool,
    pub(super) cut_index: i32,
    pub(super) line_parameters: &'a mut Sequence<f64>,
    pub(super) point_codes: &'a mut Sequence<i32>,
    pub(super) vertices: &'a mut Sequence<Shape>,
}

struct SplitByLineState<'a> {
    composer: &'a ShapeFixComposeShell,
    input: SplitByLineInput<'a>,
    wire_data: Handle<ShapeExtendWireData>,
    intersections: Vec<SplitIntersection>,
    line_adaptor: Geom2dAdaptorCurve,
}

#[derive(Clone, Copy)]
pub(super) struct SplitIntersection {
    pub(super) edge_index: i32,
    pub(super) edge_parameter: f64,
    pub(super) line_parameter: f64,
    order: u8,
}

pub(super) fn split_by_line(
    composer: &ShapeFixComposeShell,
    input: SplitByLineInput<'_>,
) -> Result<bool, Failure> {
    let mut state = SplitByLineState::new(composer, input)?;
    state.run()
}

impl<'a> SplitByLineState<'a> {
    fn new(
        composer: &'a ShapeFixComposeShell,
        input: SplitByLineInput<'a>,
    ) -> Result<Self, Failure> {
        let wire_data = input.wire.wire_data().clone();
        if !input.wire.is_vertex() {
            wire_data.get().ok_or_else(null_wire)?;
        }
        let line_curve = Handle::new(Geom2dLine::from_lin2d(input.line)).cast();
        let line_adaptor = Geom2dAdaptorCurve::with_curve(&line_curve)?;
        Ok(Self {
            composer,
            input,
            wire_data,
            intersections: Vec::new(),
            line_adaptor,
        })
    }

    fn run(&mut self) -> Result<bool, Failure> {
        if self.input.wire.is_vertex() {
            return self.split_vertex();
        }
        let periodic_axis = closed_mode::periodic_cut_direction(self.composer, self.input.line);
        self.collect_intersections(periodic_axis)?;
        if self.intersections.is_empty() {
            return Ok(false);
        }
        self.sort_intersections();
        closed_mode::remove_duplicates(self.composer, self.input.wire, &mut self.intersections)?;
        if self.intersections.is_empty() {
            return Ok(false);
        }
        let segment_codes = self.merge_tangential_segments()?;
        if self.intersections.is_empty() {
            return Ok(false);
        }
        let (point_codes, split_codes) = self.classify_output(&segment_codes);
        self.split_and_append(&split_codes, &point_codes)?;
        Ok(true)
    }

    fn split_vertex(&mut self) -> Result<bool, Failure> {
        let source = self.input.wire.vertex();
        let point = BRepTool::pnt(&source)?;
        let face = self.composer.fields.read().face.clone();
        let surface = BRepTool::surface(&face)?;
        let uv = ShapeAnalysisSurface::new(&surface)?.value_of_uv(point, precision::confusion())?;
        if geometry::point_line_position(uv, self.input.line).0 != 0 {
            return Ok(false);
        }
        self.replace_vertex(
            &source,
            point,
            geometry::parameter_on_line(uv, self.input.line),
        )?;
        Ok(true)
    }

    fn replace_vertex(
        &mut self,
        source: &rustycad_tkbrep::topods::Vertex,
        point: rustycad_tkmath::gp::Point3,
        parameter: f64,
    ) -> Result<(), Failure> {
        let mut replacement = rustycad_tkbrep::topods::Vertex::default();
        BRepBuilder::default().make_vertex_at(
            &mut replacement,
            point,
            BRepTool::vertex_tolerance(source)?,
        )?;
        replacement.set_orientation(Orientation::Forward);
        self.composer
            .context()
            .get()
            .ok_or_else(null_context)?
            .replace(
                &source.clone().into_shape(),
                &replacement.clone().into_shape(),
            )?;
        self.input.wire.set_vertex(&replacement);
        self.input.line_parameters.append(parameter);
        self.input.point_codes.append(ITP_TANG);
        self.input.vertices.append(replacement.into_shape());
        Ok(())
    }

    fn collect_intersections(
        &mut self,
        periodic_axis: Option<closed_mode::PeriodicAxis>,
    ) -> Result<(), Failure> {
        let mut junction = JunctionState::default();
        for edge_index in 1..=self.input.wire.nb_edges() {
            let edge = self.input.wire.edge(edge_index)?;
            let face = self.composer.fields.read().face.clone();
            let (curve, first, last) =
                ShapeAnalysisEdge::new().pcurve_on_face(&edge, &face, false)?;
            let mut scan = EdgePcurveScan::new(&curve, first, last, periodic_axis)?;
            let transition =
                self.collect_edge_junction(&edge, edge_index, &scan, periodic_axis, &mut junction)?;
            self.collect_curve_intersections(edge_index, transition, &mut scan)?;
        }
        self.collect_closing_junction(junction, periodic_axis)
    }

    fn collect_edge_junction(
        &mut self,
        edge: &Edge,
        index: i32,
        scan: &EdgePcurveScan,
        periodic_axis: Option<closed_mode::PeriodicAxis>,
        junction: &mut JunctionState,
    ) -> Result<(i32, i32), Failure> {
        let start_parameter = edge_start_parameter(edge, scan.first, scan.last);
        let end_parameter = edge_end_parameter(edge, scan.first, scan.last);
        let (start, end) = oriented_scan_endpoints(edge, scan);
        let (code, deviation) = geometry::point_line_position(start, self.input.line);
        let (end_code, end_deviation) = geometry::point_line_position(end, self.input.line);
        let previous_code = junction.previous_code;
        if index == 1 {
            junction.start_code = code;
            junction.start_point = Some(start);
            junction.start_deviation = deviation;
        } else if (code == 0 || code != junction.previous_code)
            && junction_within_half_period(deviation, junction.previous_deviation, periodic_axis)
        {
            self.intersections.push(SplitIntersection {
                edge_index: index,
                edge_parameter: start_parameter,
                line_parameter: geometry::parameter_between_points(
                    start,
                    junction.previous_point,
                    self.input.line,
                ),
                order: 0,
            });
        }
        junction.previous_point = end;
        junction.previous_code = end_code;
        junction.previous_deviation = end_deviation;
        junction.last_parameter = end_parameter;
        Ok((code, previous_code))
    }

    fn collect_curve_intersections(
        &mut self,
        index: i32,
        transition: (i32, i32),
        scan: &mut EdgePcurveScan,
    ) -> Result<(), Failure> {
        for iteration in 0..scan.iterations {
            self.collect_curve_intersection(index, transition, scan)?;
            if iteration + 1 < scan.iterations {
                scan.advance_period()?;
            }
        }
        Ok(())
    }

    fn collect_curve_intersection(
        &mut self,
        edge_index: i32,
        transition: (i32, i32),
        scan: &EdgePcurveScan,
    ) -> Result<(), Failure> {
        let adaptor = Geom2dAdaptorCurve::with_range(&scan.curve, scan.first, scan.last)?;
        let intersection = GInter::between(&self.line_adaptor, &adaptor, TOLINT, TOLINT)?;
        if intersection.is_done() {
            self.append_curve_intersections(
                &intersection,
                edge_index,
                transition.0 != 0 && transition.1 != 0,
                (scan.first, scan.last),
            )?;
        }
        Ok(())
    }

    fn append_curve_intersections(
        &mut self,
        intersection: &GInter,
        edge_index: i32,
        retain_endpoint: bool,
        range: (f64, f64),
    ) -> Result<(), Failure> {
        for rank in 1..=intersection.nb_points()? {
            let point = intersection.point(rank)?;
            if point.transition_of_second().position_on_curve() != Position::Middle
                && !retain_endpoint
            {
                continue;
            }
            self.append_clamped_intersection(edge_index, point, range);
        }
        for rank in 1..=intersection.nb_segments()? {
            let segment = intersection.segment(rank)?;
            self.append_segment_endpoints(edge_index, segment, range)?;
        }
        Ok(())
    }

    fn append_segment_endpoints(
        &mut self,
        edge_index: i32,
        segment: rustycad_tkgeomalgo::int_res2d::IntersectionSegment,
        range: (f64, f64),
    ) -> Result<(), Failure> {
        if segment.has_first_point() {
            self.append_clamped_intersection(edge_index, segment.first_point()?, range);
        }
        if segment.has_last_point() {
            self.append_clamped_intersection(edge_index, segment.last_point()?, range);
        }
        Ok(())
    }

    fn append_clamped_intersection(
        &mut self,
        edge_index: i32,
        point: rustycad_tkgeomalgo::int_res2d::IntersectionPoint,
        range: (f64, f64),
    ) {
        let parameter = point.param_on_second().clamp(range.0, range.1);
        self.intersections.push(SplitIntersection {
            edge_index,
            edge_parameter: parameter,
            line_parameter: point.param_on_first(),
            order: 1,
        });
    }

    fn sort_intersections(&mut self) {
        let directions: Vec<f64> = (1..=self.input.wire.nb_edges())
            .map(|index| {
                if self.edge_is_reversed(index) {
                    -1.0
                } else {
                    1.0
                }
            })
            .collect();
        self.intersections.sort_by(|left, right| {
            left.edge_index.cmp(&right.edge_index).then_with(|| {
                left.order.cmp(&right.order).then_with(|| {
                    let direction = directions[(left.edge_index - 1) as usize];
                    (direction * left.edge_parameter).total_cmp(&(direction * right.edge_parameter))
                })
            })
        });
    }

    fn edge_is_reversed(&self, index: i32) -> bool {
        self.input
            .wire
            .edge(index)
            .is_ok_and(|edge| edge.orientation() == rustycad_tkg3d::top_abs::Orientation::Reversed)
    }

    fn classify_segments(&self) -> Result<Vec<i32>, Failure> {
        let mut codes = Vec::with_capacity(self.intersections.len());
        for index in 0..self.intersections.len() {
            codes.push(self.segment_code(index)?);
        }
        Ok(codes)
    }

    fn merge_tangential_segments(&mut self) -> Result<Vec<i32>, Failure> {
        let mut codes = self.classify_segments()?;
        if matches!(
            self.input.wire.orientation(),
            rustycad_tkg3d::top_abs::Orientation::External
                | rustycad_tkg3d::top_abs::Orientation::Internal
        ) {
            return Ok(codes);
        }
        while let Some(index) = closed_mode::mergeable_tangential_point(
            self.composer.closed_mode(),
            &self.intersections,
            &codes,
        ) {
            self.intersections.remove(index);
            if self.intersections.is_empty() {
                return Ok(Vec::new());
            }
            codes = self.classify_segments()?;
        }
        Ok(codes)
    }

    fn segment_code(&self, index: usize) -> Result<i32, Failure> {
        let next = (index + 1) % self.intersections.len();
        let from = self.intersections[index];
        let to = self.intersections[next];
        self.composer.compute_code(
            self.wire_data.get().ok_or_else(null_wire)?,
            self.input.line,
            from.edge_index,
            to.edge_index,
            from.edge_parameter,
            to.edge_parameter,
            self.input.wire.orientation() == rustycad_tkg3d::top_abs::Orientation::Internal,
        )
    }

    fn classify_output(&self, segment_codes: &[i32]) -> (Vec<i32>, Vec<i32>) {
        closed_mode::classify_output(
            self.composer,
            self.input.wire.orientation(),
            segment_codes,
            &self.intersections,
        )
    }

    fn split_and_append(&mut self, codes: &[i32], point_codes: &[i32]) -> Result<(), Failure> {
        let mut arguments = self.split_arguments(codes)?;
        let mut split_vertices = Sequence::default();
        let result = split_wire::split_wire(
            self.composer,
            SplitWireInput {
                wire: self.input.wire,
                indices: &mut arguments.indices,
                values: &arguments.values,
                vertices: &mut split_vertices,
                segment_codes: &arguments.segment_codes,
                cut_by_u: self.input.cut_by_u,
                cut_index: self.input.cut_index,
            },
        )?;
        *self.input.wire = result;
        self.append_outputs(point_codes, &split_vertices)
    }

    fn split_arguments(&self, codes: &[i32]) -> Result<SplitArguments, Failure> {
        let mut indices = Sequence::default();
        let mut values = Sequence::default();
        let mut segment_codes = Sequence::default();
        for intersection in &self.intersections {
            indices.append(intersection.edge_index);
            values.append(intersection.edge_parameter);
        }
        for code in codes {
            segment_codes.append(*code);
        }
        Ok(SplitArguments {
            indices,
            values,
            segment_codes,
        })
    }

    fn append_outputs(
        &mut self,
        point_codes: &[i32],
        vertices: &Sequence<Shape>,
    ) -> Result<(), Failure> {
        for (index, code) in point_codes.iter().enumerate() {
            self.input
                .line_parameters
                .append(self.intersections[index].line_parameter);
            self.input.point_codes.append(*code);
            self.input
                .vertices
                .append(vertices.value(index + 1)?.clone());
        }
        Ok(())
    }

    fn collect_closing_junction(
        &mut self,
        junction: JunctionState,
        periodic_axis: Option<closed_mode::PeriodicAxis>,
    ) -> Result<(), Failure> {
        let orientation = self.input.wire.orientation();
        let first = junction.start_point.ok_or_else(null_wire)?;
        if matches!(
            orientation,
            rustycad_tkg3d::top_abs::Orientation::External
                | rustycad_tkg3d::top_abs::Orientation::Internal
        ) || (junction.previous_code != 0 && junction.previous_code == junction.start_code)
            || !junction_within_half_period(
                junction.start_deviation,
                junction.previous_deviation,
                periodic_axis,
            )
        {
            return Ok(());
        }
        self.intersections.push(SplitIntersection {
            edge_index: self.input.wire.nb_edges(),
            edge_parameter: junction.last_parameter,
            line_parameter: geometry::parameter_between_points(
                junction.previous_point,
                first,
                self.input.line,
            ),
            order: 2,
        });
        Ok(())
    }
}

fn oriented_scan_endpoints(
    edge: &Edge,
    scan: &EdgePcurveScan,
) -> (rustycad_tkmath::gp::Point2, rustycad_tkmath::gp::Point2) {
    if edge.orientation() == Orientation::Reversed {
        (scan.end, scan.start)
    } else {
        (scan.start, scan.end)
    }
}

fn junction_within_half_period(
    deviation: f64,
    previous_deviation: f64,
    axis: Option<closed_mode::PeriodicAxis>,
) -> bool {
    let Some(axis) = axis else {
        return true;
    };
    let period = match axis {
        closed_mode::PeriodicAxis::U { period, .. }
        | closed_mode::PeriodicAxis::V { period, .. } => period,
    };
    (deviation - previous_deviation).abs() < period * 0.5
}
