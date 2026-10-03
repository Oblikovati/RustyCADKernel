//! `ShapeFix_IntersectionTool::CutEdge`, kept distinct from the similarly named SplitTool method.
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg2d::geom2d::{Geom2dLine, Geom2dTrimmedCurve};

pub(super) fn cut_edge(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    face: &Face,
    cut_line: &mut bool,
) -> Result<bool, Failure> {
    let difference = (cut - endpoint).abs();
    if difference < 10.0 * precision::p_confusion() {
        return Ok(false);
    }
    let original_range = spatial_range(edge)?;
    if difference < 10.0 * precision::p_confusion() {
        return Ok(false);
    }
    cut_edge_by_pcurve(edge, endpoint, cut, face, original_range, cut_line)
}

fn spatial_range(edge: &Edge) -> Result<(f64, f64), Failure> {
    let curve = BRepTool::curve_with_location(edge)?;
    Ok((curve.first, curve.last))
}

fn cut_edge_by_pcurve(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    face: &Face,
    original_range: (f64, f64),
    cut_line: &mut bool,
) -> Result<bool, Failure> {
    if BRepTool::same_parameter(edge)? {
        return cut_same_parameter(edge, endpoint, cut, original_range);
    }
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    let trimmed = curve.cast::<Geom2dTrimmedCurve>();
    let Some(trimmed) = trimmed.get() else {
        return Ok(false);
    };
    cut_trimmed_pcurve(
        edge,
        (endpoint, cut),
        (first, last),
        original_range,
        trimmed,
        cut_line,
    )
}

fn cut_trimmed_pcurve(
    edge: &Edge,
    parameters: (f64, f64),
    pcurve_range: (f64, f64),
    original_range: (f64, f64),
    trimmed: &Geom2dTrimmedCurve,
    cut_line: &mut bool,
) -> Result<bool, Failure> {
    let (endpoint, cut) = parameters;
    let builder = BRepBuilder::default();
    builder.range(edge, endpoint.min(cut), endpoint.max(cut), false)?;
    if trimmed.basis_curve().cast::<Geom2dLine>().is_null() {
        return Ok(true);
    }
    update_cut_line_range(
        &builder,
        edge,
        endpoint,
        cut,
        pcurve_range,
        original_range,
        cut_line,
    )?;
    Ok(true)
}

fn update_cut_line_range(
    builder: &BRepBuilder,
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    pcurve_range: (f64, f64),
    original_range: (f64, f64),
    cut_line: &mut bool,
) -> Result<(), Failure> {
    let confusion = precision::p_confusion();
    if (endpoint - pcurve_range.1).abs() < confusion {
        return cut_line_from_start(builder, edge, cut, pcurve_range, original_range, cut_line);
    }
    if (endpoint - pcurve_range.0).abs() < confusion {
        return cut_line_from_end(builder, edge, cut, pcurve_range, original_range, cut_line);
    }
    Ok(())
}

fn cut_line_from_start(
    builder: &BRepBuilder,
    edge: &Edge,
    cut: f64,
    pcurve: (f64, f64),
    spatial: (f64, f64),
    cut_line: &mut bool,
) -> Result<(), Failure> {
    let distance = (cut - pcurve.0) * (spatial.1 - spatial.0) / (pcurve.1 - pcurve.0);
    builder.range(edge, spatial.0 + distance, spatial.1, true)?;
    *cut_line = true;
    Ok(())
}

fn cut_line_from_end(
    builder: &BRepBuilder,
    edge: &Edge,
    cut: f64,
    pcurve: (f64, f64),
    spatial: (f64, f64),
    cut_line: &mut bool,
) -> Result<(), Failure> {
    let distance = (pcurve.1 - cut) * (spatial.1 - spatial.0) / (pcurve.1 - pcurve.0);
    builder.range(edge, spatial.0, spatial.1 - distance, true)?;
    *cut_line = true;
    Ok(())
}

fn cut_same_parameter(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    original_range: (f64, f64),
) -> Result<bool, Failure> {
    let span = (original_range.1 - original_range.0).abs();
    if (span - (cut - endpoint).abs()).abs() < precision::p_confusion() {
        return Ok(false);
    }
    BRepBuilder::default().range(edge, endpoint.min(cut), endpoint.max(cut), false)?;
    Ok(true)
}
