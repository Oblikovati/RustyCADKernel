use super::{
    EdgeMergeOptions, bspline, first_circle_pair, first_linear_pair, replace_circle_pair,
    replace_line_pair,
};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Applies edge candidates in one shape-wide context, following native unique-edge scope.
pub(super) fn merge_edges(shape: &Shape, options: &EdgeMergeOptions<'_>) -> Result<Shape, Failure> {
    let limit = super::global_context::unique_edges(shape)?.len();
    let mut result = shape.clone();
    for _ in 0..limit {
        let edges = super::global_context::unique_edges(&result)?;
        let protected = remap_protected_vertices(options)?;
        let current_options = EdgeMergeOptions {
            keep_shapes: &protected,
            ..*options
        };
        let Some(merged) = merge_next_candidate(&result, &edges, &current_options)? else {
            break;
        };
        result = merged;
    }
    Ok(result)
}

fn remap_protected_vertices(options: &EdgeMergeOptions<'_>) -> Result<Vec<Shape>, Failure> {
    options
        .keep_shapes
        .iter()
        .map(|shape| options.context.apply(shape, ShapeEnum::Shape))
        .collect()
}

fn merge_next_candidate(
    shape: &Shape,
    edges: &[rustycad_tkbrep::topods::Edge],
    options: &EdgeMergeOptions<'_>,
) -> Result<Option<Shape>, Failure> {
    if let Some((first, second, start, end)) =
        first_linear_pair(edges, options.angular_tolerance, options.keep_shapes)?
    {
        return replace_line(shape, options, &first, &second, start, end).map(Some);
    }
    if let Some((first, second, start, end)) =
        first_circle_pair(edges, options.angular_tolerance, options.keep_shapes)?
    {
        return replace_circle(shape, options, &first, &second, start, end).map(Some);
    }
    if options.concat_bspline_edges {
        return bspline::merge_first_pair(shape, edges, options);
    }
    Ok(None)
}

fn replace_line(
    shape: &Shape,
    options: &EdgeMergeOptions<'_>,
    first: &rustycad_tkbrep::topods::Edge,
    second: &rustycad_tkbrep::topods::Edge,
    start: rustycad_tkbrep::topods::Vertex,
    end: rustycad_tkbrep::topods::Vertex,
) -> Result<Shape, Failure> {
    replace_line_pair(
        shape,
        options.context,
        first,
        second,
        start,
        end,
        options.safe_input_mode,
    )
}

fn replace_circle(
    shape: &Shape,
    options: &EdgeMergeOptions<'_>,
    first: &rustycad_tkbrep::topods::Edge,
    second: &rustycad_tkbrep::topods::Edge,
    start: rustycad_tkbrep::topods::Vertex,
    end: rustycad_tkbrep::topods::Vertex,
) -> Result<Shape, Failure> {
    replace_circle_pair(
        shape,
        options.context,
        first,
        second,
        start,
        end,
        options.safe_input_mode,
    )
}
