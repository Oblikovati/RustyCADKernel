// Rust translation of the Init members of BRepBuilderAPI_MakeEdge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Init members of `BRepBuilderAPI_MakeEdge`: each runs the `BRepLib_MakeEdge` Init
//! of the same arguments. A failing Init keeps the shape and the done flag of an earlier
//! successful one, as natively.
use super::BRepBuilderAPIMakeEdge;
use crate::brep_builder_api::wrap::wrapped_updates;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::gp::Point3;

impl BRepBuilderAPIMakeEdge {
    wrapped_updates!(BRepBuilderAPIMakeEdge, my_make_edge;
        init_curve(curve: &Handle<dyn GeomCurve>) = "Init(C)";
        init_curve_params(curve: &Handle<dyn GeomCurve>, p1: f64, p2: f64) = "Init(C, p1, p2)";
        init_curve_points(curve: &Handle<dyn GeomCurve>, p1: Point3, p2: Point3) = "Init(C, P1, P2)";
        init_curve_vertices(curve: &Handle<dyn GeomCurve>, v1: &Vertex, v2: &Vertex) = "Init(C, V1, V2)";
        init_curve_points_params(curve: &Handle<dyn GeomCurve>, point1: Point3, point2: Point3, p1: f64, p2: f64)
            = "Init(C, P1, P2, p1, p2)";
        init_curve_vertices_params(curve: &Handle<dyn GeomCurve>, v1: &Vertex, v2: &Vertex, p1: f64, p2: f64)
            = "Init(C, V1, V2, p1, p2)";
        init_pcurve(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>) = "Init(C, S)";
        init_pcurve_params(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, p1: f64, p2: f64)
            = "Init(C, S, p1, p2)";
        init_pcurve_points(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, p1: Point3, p2: Point3)
            = "Init(C, S, P1, P2)";
        init_pcurve_vertices(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, v1: &Vertex, v2: &Vertex)
            = "Init(C, S, V1, V2)";
        init_pcurve_points_params(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, point1: Point3,
            point2: Point3, p1: f64, p2: f64) = "Init(C, S, P1, P2, p1, p2)";
        init_pcurve_vertices_params(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, v1: &Vertex,
            v2: &Vertex, p1: f64, p2: f64) = "Init(C, S, V1, V2, p1, p2)";
    );
}
