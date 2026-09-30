//! Public construction and repeated execution preserve status, edge identity, and accumulation.
use super::{attempt, fixtures, integer, real};
use rustycad_tkbo::int_tools::{EdgeEdge, Range};
use rustycad_tkbrep::{brep::BRepBuilder, topods::Edge};
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn snapshot(intersection: &EdgeEdge, original: &[Edge; 2]) {
    integer(i32::from(intersection.is_done()));
    real(intersection.fuzzy_value());
    integer(i32::from(intersection.is_coincidence_checked_quickly()));
    integer(intersection.common_parts().len() as i32);
    for part in intersection.common_parts() {
        integer(part.shape_type() as i32);
        integer(i32::from(part.all_null_flag()));
        for edge in [part.edge1(), part.edge2()] {
            for source in original {
                integer(i32::from(edge.is_same(source)));
            }
        }
        for value in [
            part.range1().first(),
            part.range1().last(),
            part.vertex_parameter1(),
            part.vertex_parameter2(),
        ] {
            real(value);
        }
        integer(part.ranges2().len() as i32);
        for range in part.ranges2() {
            real(range.first());
            real(range.last());
        }
    }
}
fn configured(edges: &[Edge; 2], values: &[f64]) -> EdgeEdge {
    match values[7] as i32 {
        0 => {
            let mut result = EdgeEdge::new();
            result.set_edge1_range(&edges[0], values[2], values[3]);
            result.set_edge2_range(&edges[1], values[2], values[3]);
            result
        }
        1 => {
            let mut result = EdgeEdge::with_edges(&edges[0], &edges[1]);
            result.set_range1(Range::new(values[2], values[3]));
            result.set_range2(Range::new(values[2], values[3]));
            result.set_range2_bounds(values[2], values[3]);
            result
        }
        _ => EdgeEdge::with_ranges(
            &edges[0], values[2], values[3], &edges[1], values[2], values[3],
        ),
    }
}
pub(super) fn case(values: &[f64]) -> Result<(), Failure> {
    let first = fixtures::geometry(values[0] as i32, 0.5)?;
    let second = fixtures::geometry(values[1] as i32, 0.5)?;
    let mut edges = [
        BRepBuilderAPIMakeEdge::from_curve_params(&first, 0., 1.)?.edge()?,
        BRepBuilderAPIMakeEdge::from_curve_params(&second, 0., 1.)?.edge()?,
    ];
    if values[8] != 0. {
        let mut placement = rustycad_tkmath::gp::Transform3::default();
        placement.set_translation(rustycad_tkmath::gp::Vector3::new(
            values[8],
            0.,
            values[8] / 2.,
        ));
        edges[1].move_by(&rustycad_tkmath::toploc::Location::new(placement), true)?;
    }
    let mut intersection = configured(&edges, values);
    intersection.set_fuzzy_value(values[4]);
    intersection.use_quick_coincidence_check(values[5] != 0.);
    snapshot(&intersection, &edges);
    for _ in 0..2 {
        attempt(intersection.perform());
        snapshot(&intersection, &edges);
    }
    if values[6] != 0. {
        failure_history(&mut intersection, &edges, values[6] as i32)?;
    }
    Ok(())
}
fn failure_history(
    intersection: &mut EdgeEdge,
    edges: &[Edge; 2],
    kind: i32,
) -> Result<(), Failure> {
    let mut invalid = Edge::default();
    if kind >= 2 {
        BRepBuilder::default().make_edge(&mut invalid)?;
    }
    if kind == 2 {
        BRepBuilder::default().degenerated(&invalid, true)?;
    }
    intersection.set_edge1(&invalid);
    attempt(intersection.perform());
    snapshot(intersection, edges);
    intersection.set_edge1(&edges[0]);
    intersection.set_edge2(&edges[1]);
    attempt(intersection.perform());
    snapshot(intersection, edges);
    Ok(())
}
