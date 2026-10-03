//! Mode-one self-intersection repair, which replaces one edge by two trimmed edges.
use super::{FixRoot, ShapeFixWire};
use crate::shape_analysis::{
    ShapeAnalysisEdge, ShapeAnalysisTransferParametersProj, TransferParameters,
};
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use crate::shape_fix::ShapeFixEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::{ncollection::HSequence, precision};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

struct SplitCurves {
    first_2d: Option<Handle<dyn Geom2dCurve>>,
    second_2d: Handle<dyn Geom2dCurve>,
    first_3d: Option<Handle<dyn GeomCurve>>,
    second_3d: Handle<dyn GeomCurve>,
    first_parameter: f64,
    last_parameter: f64,
    middle_vertex_point: Point3,
    first_2d_parameter: f64,
    last_2d_parameter: f64,
}

struct SplitRanges {
    three_d: (f64, f64),
    two_d: (f64, f64),
}

impl ShapeFixWire {
    pub(super) fn split_self_intersection(
        &self,
        analyzer: &crate::shape_analysis::ShapeAnalysisWire,
        index: i32,
        points: &rustycad_tkernel::ncollection::Sequence<
            rustycad_tkgeomalgo::int_res2d::IntersectionPoint,
        >,
    ) -> Result<bool, Failure> {
        let edge_index = if index > 0 {
            index
        } else {
            analyzer.wire_data().nb_edges()
        };
        let edge = analyzer.wire_data().edge(edge_index)?;
        let face = analyzer.face();
        let intersection = *points.value(1)?;
        let Some((first, second, tolerance)) = split_edge(&edge, &face, &intersection)? else {
            self.or_last_fix_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        };
        self.or_last_fix_status(ShapeExtendStatus::Done4);
        let wire = split_wire(first.as_ref(), &second)?;
        self.replace_split_edge(&edge, edge_index, &wire)?;
        if tolerance > self.max_tolerance() {
            self.or_last_fix_status(ShapeExtendStatus::Fail2);
        }
        self.or_last_fix_status(ShapeExtendStatus::Done8);
        Ok(true)
    }

    fn replace_split_edge(
        &self,
        edge: &Edge,
        index: i32,
        replacement: &Wire,
    ) -> Result<(), Failure> {
        let context = self.context();
        if context.is_null() {
            let data = self.wire_data()?;
            data.remove(index)?;
            data.add_wire(replacement, index)?;
            return Ok(());
        }
        context.replace(
            &edge.clone().into_shape(),
            &replacement.clone().into_shape(),
        )?;
        self.update_wire()
    }
}

fn split_edge(
    edge: &Edge,
    face: &Face,
    intersection: &rustycad_tkgeomalgo::int_res2d::IntersectionPoint,
) -> Result<Option<(Option<Edge>, Edge, f64)>, Failure> {
    if BRepTool::is_closed_on_face(edge, face)? {
        return Ok(None);
    }
    let (curve_2d, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    let (curve_3d, first_3d, last_3d) = BRepTool::curve(edge)?;
    if curve_2d.is_null() || curve_3d.is_null() {
        return Ok(None);
    }
    let curves = trimmed_split_curves(
        edge,
        face,
        &curve_2d,
        &curve_3d,
        (first, last),
        (first_3d, last_3d),
        intersection,
    )?;
    let (first_vertex, last_vertex) = split_vertices(edge);
    let middle_vertex = create_middle_vertex(&curves)?;
    let (first_start, first_end, second_start, second_end) =
        split_vertex_pairs(edge, &first_vertex, &last_vertex, &middle_vertex);
    let builder = BRepBuilder::default();
    let mut first_edge = curves
        .first_2d
        .as_ref()
        .map(|_| ShapeBuildEdge.copy_replace_vertices(edge, first_start, first_end))
        .transpose()?;
    let mut second_edge = ShapeBuildEdge.copy_replace_vertices(edge, second_start, second_end)?;
    if let Some(first_edge) = &mut first_edge {
        update_split_edge(
            &builder,
            first_edge,
            face,
            curves.first_2d.as_ref(),
            curves.first_3d.as_ref(),
            SplitRanges {
                three_d: (first_3d, curves.first_parameter),
                two_d: (first, curves.first_2d_parameter),
            },
        )?;
    }
    update_split_edge(
        &builder,
        &mut second_edge,
        face,
        Some(&curves.second_2d),
        Some(&curves.second_3d),
        SplitRanges {
            three_d: (curves.last_parameter, last_3d),
            two_d: (curves.last_2d_parameter, last),
        },
    )?;
    let fixer = ShapeFixEdge::new();
    if let Some(first_edge) = &first_edge {
        fixer.fix_same_parameter_default(first_edge)?;
        fixer.fix_vertex_tolerance(first_edge)?;
    }
    fixer.fix_same_parameter_default(&second_edge)?;
    fixer.fix_vertex_tolerance(&second_edge)?;
    let tolerance = first_edge.as_ref().map_or_else(
        || BRepTool::edge_tolerance(&second_edge),
        |first_edge| {
            Ok(BRepTool::edge_tolerance(first_edge)?.max(BRepTool::edge_tolerance(&second_edge)?))
        },
    )?;
    Ok(Some((first_edge, second_edge, tolerance)))
}

fn split_wire(first: Option<&Edge>, second: &Edge) -> Result<Wire, Failure> {
    let data = Handle::new(ShapeExtendWireData::new());
    if let Some(first) = first {
        data.add_edge(first, 0)?;
    }
    data.add_edge(second, 0)?;
    data.wire()
}

fn update_split_edge(
    builder: &BRepBuilder,
    edge: &mut Edge,
    face: &Face,
    curve_2d: Option<&Handle<dyn Geom2dCurve>>,
    curve_3d: Option<&Handle<dyn GeomCurve>>,
    ranges: SplitRanges,
) -> Result<(), Failure> {
    if let Some(curve_2d) = curve_2d {
        builder.update_edge_pcurve_on_face(edge, curve_2d, face, 0.0)?;
    }
    if let Some(curve_3d) = curve_3d {
        builder.update_edge_curve(edge, curve_3d, 0.0)?;
    }
    builder.range_on_face(edge, face, ranges.two_d.0, ranges.two_d.1)?;
    builder.range(edge, ranges.three_d.0, ranges.three_d.1, true)?;
    builder.same_range(edge, false)
}

fn trimmed_split_curves(
    edge: &Edge,
    face: &Face,
    curve_2d: &Handle<dyn Geom2dCurve>,
    curve_3d: &Handle<dyn GeomCurve>,
    range_2d: (f64, f64),
    range_3d: (f64, f64),
    intersection: &rustycad_tkgeomalgo::int_res2d::IntersectionPoint,
) -> Result<SplitCurves, Failure> {
    let (first_parameter, last_parameter) = sorted_parameters(intersection);
    let first_2d = trim_2d(curve_2d, range_2d.0, first_parameter)?;
    let second_2d = trim_2d(curve_2d, last_parameter, range_2d.1)?
        .ok_or_else(|| Failure::new("ShapeFix_Wire split failed to create its second p-curve"))?;
    let (_crossing, middle_vertex_point, first_3d_parameter, last_3d_parameter) =
        transferred_split_parameters(
            edge,
            face,
            curve_2d,
            curve_3d,
            first_parameter,
            last_parameter,
        )?;
    let first_3d = trim_3d(curve_3d, range_3d.0, first_3d_parameter)?;
    let second_3d = trim_3d(curve_3d, last_3d_parameter, range_3d.1)?
        .ok_or_else(|| Failure::new("ShapeFix_Wire split failed to create its second 3D curve"))?;
    Ok(SplitCurves {
        first_2d,
        second_2d,
        first_3d,
        second_3d,
        first_parameter: first_3d_parameter,
        last_parameter: last_3d_parameter,
        middle_vertex_point,
        first_2d_parameter: first_parameter,
        last_2d_parameter: last_parameter,
    })
}

fn sorted_parameters(
    intersection: &rustycad_tkgeomalgo::int_res2d::IntersectionPoint,
) -> (f64, f64) {
    let first = intersection.param_on_first();
    let second = intersection.param_on_second();
    (first.min(second), first.max(second))
}

fn trim_2d(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    if last - first <= precision::p_confusion() {
        return Ok(None);
    }
    Ok(Some(
        Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast(),
    ))
}

fn trim_3d(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
    if last - first <= precision::p_confusion() {
        return Ok(None);
    }
    Ok(Some(
        Handle::new(GeomTrimmedCurve::new(curve, first, last, true, true)?).cast(),
    ))
}

fn transferred_split_parameters(
    edge: &Edge,
    face: &Face,
    curve_2d: &Handle<dyn Geom2dCurve>,
    curve_3d: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<(Point3, Point3, f64, f64), Failure> {
    let source = ShapeAnalysisTransferParametersProj::from_edge(edge, face)?;
    let values = sequence(&[first, last, (first + last) * 0.5]);
    let mapped = source.perform(&values, false)?;
    let first_3d = sequence_value(&mapped, 1)?;
    let last_3d = sequence_value(&mapped, 2)?;
    let middle_3d = sequence_value(&mapped, 3)?;
    let surface = BRepTool::surface(face)?;
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    let p1 = adaptor.value(curve_2d.value(first)?.x(), curve_2d.value(first)?.y())?;
    let p2 = adaptor.value(curve_2d.value(last)?.x(), curve_2d.value(last)?.y())?;
    let crossing = average(p1, p2);
    let deviation_first = crossing.distance(&curve_3d.value(first_3d)?);
    let deviation_last = crossing.distance(&curve_3d.value(last_3d)?);
    let deviation_middle = crossing.distance(&curve_3d.value(middle_3d)?);
    let (cut_first, cut_last, middle_point) =
        if deviation_middle > deviation_first.max(deviation_last) {
            (
                first_3d,
                last_3d,
                average(curve_3d.value(first_3d)?, curve_3d.value(last_3d)?),
            )
        } else {
            (middle_3d, middle_3d, curve_3d.value(middle_3d)?)
        };
    Ok((crossing, middle_point, cut_first, cut_last))
}

fn sequence(values: &[f64]) -> Handle<HSequence<f64>> {
    let sequence = Handle::new(HSequence::<f64>::default());
    sequence.change_sequence(|items| {
        values.iter().for_each(|value| {
            items.append(*value);
        });
    });
    sequence
}

fn sequence_value(sequence: &Handle<HSequence<f64>>, index: i32) -> Result<f64, Failure> {
    sequence
        .get()
        .ok_or_else(|| Failure::new("ShapeFix_Wire lost projected split parameters"))?
        .sequence(|items| items.value(index as usize).copied())
}

fn average(first: Point3, last: Point3) -> Point3 {
    Point3::new(
        (first.x() + last.x()) * 0.5,
        (first.y() + last.y()) * 0.5,
        (first.z() + last.z()) * 0.5,
    )
}

fn split_vertices(edge: &Edge) -> (Vertex, Vertex) {
    let analysis = ShapeAnalysisEdge::new();
    (analysis.first_vertex(edge), analysis.last_vertex(edge))
}

fn create_middle_vertex(curves: &SplitCurves) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, curves.middle_vertex_point, 0.0)?;
    Ok(vertex)
}

fn split_vertex_pairs<'a>(
    edge: &Edge,
    first: &'a Vertex,
    last: &'a Vertex,
    middle: &'a Vertex,
) -> (&'a Vertex, &'a Vertex, &'a Vertex, &'a Vertex) {
    if edge.orientation() == Orientation::Forward {
        (first, middle, middle, last)
    } else {
        (middle, last, first, middle)
    }
}
