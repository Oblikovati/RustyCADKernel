use super::*;
use crate::shape_analysis::{ShapeAnalysisTransferParametersProj, TransferParameters};
use crate::shape_build::ShapeBuildEdge;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

pub(super) fn split_tail_edges(
    edges: [&Edge; 2],
    curves: &[TailCurve; 2],
    parameters: [f64; 2],
    tolerance: f64,
    parts: &mut [Edge; 4],
) -> Result<bool, Failure> {
    let mut results = [1; 2];
    for index in 0..2 {
        let curve = &curves[index];
        if (parameters[index] - curve.range[1 - curve.junction]).abs() <= precision::p_confusion() {
            results[index] = 2;
            parts[2 * index] = edges[index].clone();
        } else if (parameters[index] - curve.range[curve.junction]).abs()
            <= precision::p_confusion()
        {
            results[index] = 0;
        }
    }
    for index in 0..2 {
        if results[index] != 1 {
            continue;
        }
        results[index] = cut_tail_edge(
            edges[index],
            &curves[index],
            parameters[index],
            tolerance,
            &mut parts[2 * index..2 * index + 2],
        )?;
    }
    Ok(results[0] + results[1] != 0)
}

fn cut_tail_edge(
    edge: &Edge,
    curve: &TailCurve,
    parameter: f64,
    tolerance: f64,
    parts: &mut [Edge],
) -> Result<i32, Failure> {
    let pieces = build_tail_parts(edge, curve, parameter, tolerance)?;
    if tail_part_length(&pieces[1 - curve.junction])? <= precision::confusion() {
        parts[0] = edge.clone();
        return Ok(2);
    }
    if tail_part_length(&pieces[curve.junction])? <= precision::confusion() {
        return Ok(0);
    }
    parts[0] = pieces[0].clone();
    parts[1] = pieces[1].clone();
    Ok(1)
}

fn build_tail_parts(
    edge: &Edge,
    curve: &TailCurve,
    parameter: f64,
    tolerance: f64,
) -> Result<[Edge; 2], Failure> {
    let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
    let transfer = ShapeAnalysisTransferParametersProj::from_edge(&forward, &Face::default())?;
    transfer.set_max_tolerance(tolerance);
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(
        &mut vertex,
        curve.geometry.value(parameter)?,
        precision::confusion(),
    )?;
    let reversed = Vertex::unchecked(vertex.oriented(Orientation::Reversed));
    let parts = [
        ShapeBuildEdge.copy_replace_vertices(&forward, &Vertex::default(), &reversed)?,
        ShapeBuildEdge.copy_replace_vertices(&forward, &vertex, &Vertex::default())?,
    ];
    prepare_tail_representations(&parts, &forward)?;
    transfer.transfer_range(&parts[0], curve.range[0], parameter, false)?;
    transfer.transfer_range(&parts[1], parameter, curve.range[1], false)?;
    Ok(parts)
}

fn prepare_tail_representations(parts: &[Edge; 2], original: &Edge) -> Result<(), Failure> {
    for part in parts {
        ShapeBuildEdge.copy_pcurves(part, original)?;
    }
    let builder = BRepBuilder::default();
    for part in parts {
        builder.same_range(part, false)?;
    }
    for part in parts {
        builder.same_parameter(part, false)?;
    }
    Ok(())
}

fn tail_part_length(edge: &Edge) -> Result<f64, Failure> {
    let mut properties = GProps::default();
    BRepGProp::linear_properties(&edge.clone().into_shape(), &mut properties, false, false)?;
    Ok(properties.mass())
}
