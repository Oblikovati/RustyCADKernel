//! Adapter to the original IntStart restriction-search contract.
use super::ContapHContTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_start::{TheArc, TheSOBTool, TheVertex};
use rustycad_tkmath::gp::Point3;

impl TheSOBTool for ContapHContTool {
    fn parameter(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        ContapHContTool::parameter(vertex, arc)
    }
    fn tolerance(vertex: &TheVertex, arc: &TheArc) -> Result<f64, Failure> {
        ContapHContTool::tolerance(vertex, arc)
    }
    fn has_been_seen(arc: &TheArc) -> Result<bool, Failure> {
        Ok(ContapHContTool::has_been_seen(arc))
    }
    fn bounds(arc: &TheArc) -> Result<(f64, f64), Failure> {
        ContapHContTool::bounds(arc)
    }
    fn nb_points(arc: &TheArc) -> Result<i32, Failure> {
        Ok(ContapHContTool::nb_points(arc))
    }
    fn value(arc: &TheArc, index: i32) -> Result<(Point3, f64, f64), Failure> {
        ContapHContTool::value(arc, index)
    }
    fn is_vertex(arc: &TheArc, index: i32) -> Result<bool, Failure> {
        Ok(ContapHContTool::is_vertex(arc, index))
    }
    fn vertex(arc: &TheArc, index: i32) -> Result<TheVertex, Failure> {
        ContapHContTool::vertex(arc, index)
    }
    fn nb_segments(arc: &TheArc) -> Result<i32, Failure> {
        Ok(ContapHContTool::nb_segments(arc))
    }
    fn has_first_point(arc: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        ContapHContTool::has_first_point(arc, index)
    }
    fn has_last_point(arc: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        ContapHContTool::has_last_point(arc, index)
    }
    fn is_all_solution(arc: &TheArc) -> Result<bool, Failure> {
        Ok(ContapHContTool::is_all_solution(arc))
    }
}
