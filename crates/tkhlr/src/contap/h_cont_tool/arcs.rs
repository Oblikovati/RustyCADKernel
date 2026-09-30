//! Arc and vertex operations shared with native IntPatch_HInterTool.
use super::ContapHContTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::{
    int_patch::IntPatchHInterTool,
    int_start::{TheArc, TheVertex},
};
use rustycad_tkmath::gp::{Point2, Point3};
impl ContapHContTool {
    /// NbSamplesOnArc(A): the sample count of an arc by curve type;
    /// `ContapHContTool::nb_samples_on_arc(&arc)?`.
    pub fn nb_samples_on_arc(a: &TheArc) -> Result<i32, Failure> {
        IntPatchHInterTool::nb_samples_on_arc(a)
    }

    /// Bounds(C, Ufirst, Ulast); `ContapHContTool::bounds(&arc)?`.
    pub fn bounds(c: &TheArc) -> Result<(f64, f64), Failure> {
        IntPatchHInterTool::bounds(c)
    }

    /// Project(C, P, Paramproj, Ptproj): the nearest extremum of `p` on the arc (20
    /// samples), `None` when there is none; `ContapHContTool::project(&arc, p)?`.
    pub fn project(c: &TheArc, p: Point2) -> Result<Option<(f64, Point2)>, Failure> {
        IntPatchHInterTool::project(c, p)
    }

    /// Tolerance(V, C): the parametric resolution of the vertex on the arc;
    /// `ContapHContTool::tolerance(&vertex, &arc)?`.
    pub fn tolerance(v: &TheVertex, c: &TheArc) -> Result<f64, Failure> {
        IntPatchHInterTool::tolerance(v, c)
    }

    /// Parameter(V, C): the parameter of the vertex on the arc;
    /// `ContapHContTool::parameter(&vertex, &arc)?`.
    pub fn parameter(v: &TheVertex, c: &TheArc) -> Result<f64, Failure> {
        IntPatchHInterTool::parameter(v, c)
    }

    /// HasBeenSeen: always false; `ContapHContTool::has_been_seen(&arc)`.
    pub fn has_been_seen(c: &TheArc) -> bool {
        IntPatchHInterTool::has_been_seen(c)
    }

    /// NbPoints: always 0; `ContapHContTool::nb_points(&arc)`.
    pub fn nb_points(c: &TheArc) -> i32 {
        IntPatchHInterTool::nb_points(c)
    }

    /// Value: always Standard_OutOfRange; `ContapHContTool::value(&arc, 1)`.
    pub fn value(c: &TheArc, index: i32) -> Result<(Point3, f64, f64), Failure> {
        IntPatchHInterTool::value(c, index)
    }

    /// IsVertex: always false; `ContapHContTool::is_vertex(&arc, 1)`.
    pub fn is_vertex(c: &TheArc, index: i32) -> bool {
        IntPatchHInterTool::is_vertex(c, index)
    }

    /// Vertex: always Standard_OutOfRange; `ContapHContTool::vertex(&arc, 1)`.
    pub fn vertex(c: &TheArc, index: i32) -> Result<TheVertex, Failure> {
        IntPatchHInterTool::vertex(c, index)
    }

    /// NbSegments: always 0; `ContapHContTool::nb_segments(&arc)`.
    pub fn nb_segments(c: &TheArc) -> i32 {
        IntPatchHInterTool::nb_segments(c)
    }

    /// HasFirstPoint: always Standard_OutOfRange; `ContapHContTool::has_first_point(&arc, 1)`.
    pub fn has_first_point(c: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        IntPatchHInterTool::has_first_point(c, index)
    }

    /// HasLastPoint: always Standard_OutOfRange; `ContapHContTool::has_last_point(&arc, 1)`.
    pub fn has_last_point(c: &TheArc, index: i32) -> Result<Option<i32>, Failure> {
        IntPatchHInterTool::has_last_point(c, index)
    }

    /// IsAllSolution: always false; `ContapHContTool::is_all_solution(&arc)`.
    pub fn is_all_solution(c: &TheArc) -> bool {
        IntPatchHInterTool::is_all_solution(c)
    }
}
