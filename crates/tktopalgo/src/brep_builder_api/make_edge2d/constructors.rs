// Rust translation of the constructors of BRepBuilderAPI_MakeEdge2d.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructors of `BRepBuilderAPI_MakeEdge2d`: each builds the
//! `BRepLib_MakeEdge2d` of the same arguments and takes its edge when it is done.
use super::BRepBuilderAPIMakeEdge2d;
use crate::brep_builder_api::wrap::wrapped_constructors;
use crate::brep_lib::BRepLibMakeEdge2d;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2};

impl BRepBuilderAPIMakeEdge2d {
    wrapped_constructors!(BRepBuilderAPIMakeEdge2d, BRepLibMakeEdge2d;
        from_vertices(v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge2d(V1, V2)";
        from_points(p1: Point2, p2: Point2) = "BRepBuilderAPI_MakeEdge2d(P1, P2)";
        from_lin2d(curve: Line2) = "BRepBuilderAPI_MakeEdge2d(const gp_Lin2d& L)";
        from_lin2d_params(curve: Line2, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge2d(L, p1, p2)";
        from_lin2d_points(curve: Line2, p1: Point2, p2: Point2) = "BRepBuilderAPI_MakeEdge2d(L, P1, P2)";
        from_lin2d_vertices(curve: Line2, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge2d(L, V1, V2)";
        from_circ2d(curve: Circle2) = "BRepBuilderAPI_MakeEdge2d(const gp_Circ2d& L)";
        from_circ2d_params(curve: Circle2, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge2d(C, p1, p2)";
        from_circ2d_points(curve: Circle2, p1: Point2, p2: Point2) = "BRepBuilderAPI_MakeEdge2d(C, P1, P2)";
        from_circ2d_vertices(curve: Circle2, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge2d(C, V1, V2)";
        from_elips2d(curve: Ellipse2) = "BRepBuilderAPI_MakeEdge2d(const gp_Elips2d& L)";
        from_elips2d_params(curve: Ellipse2, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge2d(E, p1, p2)";
        from_elips2d_points(curve: Ellipse2, p1: Point2, p2: Point2) = "BRepBuilderAPI_MakeEdge2d(E, P1, P2)";
        from_elips2d_vertices(curve: Ellipse2, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge2d(E, V1, V2)";
        from_hypr2d(curve: Hyperbola2) = "BRepBuilderAPI_MakeEdge2d(const gp_Hypr2d& L)";
        from_hypr2d_params(curve: Hyperbola2, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge2d(H, p1, p2)";
        from_hypr2d_points(curve: Hyperbola2, p1: Point2, p2: Point2) = "BRepBuilderAPI_MakeEdge2d(H, P1, P2)";
        from_hypr2d_vertices(curve: Hyperbola2, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge2d(H, V1, V2)";
        from_parab2d(curve: Parabola2) = "BRepBuilderAPI_MakeEdge2d(const gp_Parab2d& L)";
        from_parab2d_params(curve: Parabola2, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge2d(P, p1, p2)";
        from_parab2d_points(curve: Parabola2, p1: Point2, p2: Point2) = "BRepBuilderAPI_MakeEdge2d(P, P1, P2)";
        from_parab2d_vertices(curve: Parabola2, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge2d(P, V1, V2)";
        from_curve(curve: &Handle<dyn Geom2dCurve>) = "BRepBuilderAPI_MakeEdge2d(const occ::handle<Geom2d_Curve>& L)";
        from_curve_params(curve: &Handle<dyn Geom2dCurve>, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge2d(L, p1, p2)";
        from_curve_points(curve: &Handle<dyn Geom2dCurve>, p1: Point2, p2: Point2) = "BRepBuilderAPI_MakeEdge2d(L, P1, P2)";
        from_curve_vertices(curve: &Handle<dyn Geom2dCurve>, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge2d(L, V1, V2)";
        from_curve_points_params(curve: &Handle<dyn Geom2dCurve>, point1: Point2, point2: Point2, p1: f64, p2: f64)
            = "BRepBuilderAPI_MakeEdge2d(L, P1, P2, p1, p2)";
        from_curve_vertices_params(curve: &Handle<dyn Geom2dCurve>, v1: &Vertex, v2: &Vertex, p1: f64, p2: f64)
            = "BRepBuilderAPI_MakeEdge2d(L, V1, V2, p1, p2)";
    );
}
