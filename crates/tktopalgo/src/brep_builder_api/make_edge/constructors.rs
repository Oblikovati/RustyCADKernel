// Rust translation of the constructors of BRepBuilderAPI_MakeEdge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructors of `BRepBuilderAPI_MakeEdge`: each builds the `BRepLib_MakeEdge` of
//! the same arguments and takes its edge when it is done.
use super::BRepBuilderAPIMakeEdge;
use crate::brep_builder_api::wrap::wrapped_constructors;
use crate::brep_lib::BRepLibMakeEdge;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3};

impl BRepBuilderAPIMakeEdge {
    wrapped_constructors!(BRepBuilderAPIMakeEdge, BRepLibMakeEdge;
        from_vertices(v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge(V1, V2)";
        from_points(p1: Point3, p2: Point3) = "BRepBuilderAPI_MakeEdge(P1, P2)";
        from_lin(curve: Line3) = "BRepBuilderAPI_MakeEdge(const gp_Lin& L)";
        from_lin_params(curve: Line3, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge(L, p1, p2)";
        from_lin_points(curve: Line3, p1: Point3, p2: Point3) = "BRepBuilderAPI_MakeEdge(L, P1, P2)";
        from_lin_vertices(curve: Line3, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge(L, V1, V2)";
        from_circ(curve: Circle3) = "BRepBuilderAPI_MakeEdge(const gp_Circ& L)";
        from_circ_params(curve: Circle3, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge(C, p1, p2)";
        from_circ_points(curve: Circle3, p1: Point3, p2: Point3) = "BRepBuilderAPI_MakeEdge(C, P1, P2)";
        from_circ_vertices(curve: Circle3, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge(C, V1, V2)";
        from_elips(curve: Ellipse3) = "BRepBuilderAPI_MakeEdge(const gp_Elips& L)";
        from_elips_params(curve: Ellipse3, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge(E, p1, p2)";
        from_elips_points(curve: Ellipse3, p1: Point3, p2: Point3) = "BRepBuilderAPI_MakeEdge(E, P1, P2)";
        from_elips_vertices(curve: Ellipse3, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge(E, V1, V2)";
        from_hypr(curve: Hyperbola3) = "BRepBuilderAPI_MakeEdge(const gp_Hypr& L)";
        from_hypr_params(curve: Hyperbola3, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge(H, p1, p2)";
        from_hypr_points(curve: Hyperbola3, p1: Point3, p2: Point3) = "BRepBuilderAPI_MakeEdge(H, P1, P2)";
        from_hypr_vertices(curve: Hyperbola3, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge(H, V1, V2)";
        from_parab(curve: Parabola3) = "BRepBuilderAPI_MakeEdge(const gp_Parab& L)";
        from_parab_params(curve: Parabola3, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge(P, p1, p2)";
        from_parab_points(curve: Parabola3, p1: Point3, p2: Point3) = "BRepBuilderAPI_MakeEdge(P, P1, P2)";
        from_parab_vertices(curve: Parabola3, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge(P, V1, V2)";
        from_curve(curve: &Handle<dyn GeomCurve>) = "BRepBuilderAPI_MakeEdge(const occ::handle<Geom_Curve>& L)";
        from_curve_params(curve: &Handle<dyn GeomCurve>, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge(L, p1, p2)";
        from_curve_points(curve: &Handle<dyn GeomCurve>, p1: Point3, p2: Point3) = "BRepBuilderAPI_MakeEdge(L, P1, P2)";
        from_curve_vertices(curve: &Handle<dyn GeomCurve>, v1: &Vertex, v2: &Vertex) = "BRepBuilderAPI_MakeEdge(L, V1, V2)";
        from_curve_points_params(curve: &Handle<dyn GeomCurve>, point1: Point3, point2: Point3, p1: f64, p2: f64)
            = "BRepBuilderAPI_MakeEdge(L, P1, P2, p1, p2)";
        from_curve_vertices_params(curve: &Handle<dyn GeomCurve>, v1: &Vertex, v2: &Vertex, p1: f64, p2: f64)
            = "BRepBuilderAPI_MakeEdge(L, V1, V2, p1, p2)";
        from_pcurve(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>)
            = "BRepBuilderAPI_MakeEdge(const occ::handle<Geom2d_Curve>& L, S)";
        from_pcurve_params(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, p1: f64, p2: f64)
            = "BRepBuilderAPI_MakeEdge(L, S, p1, p2)";
        from_pcurve_points(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, p1: Point3, p2: Point3)
            = "BRepBuilderAPI_MakeEdge(L, S, P1, P2)";
        from_pcurve_vertices(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, v1: &Vertex, v2: &Vertex)
            = "BRepBuilderAPI_MakeEdge(L, S, V1, V2)";
        from_pcurve_points_params(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, point1: Point3,
            point2: Point3, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge(L, S, P1, P2, p1, p2)";
        from_pcurve_vertices_params(curve: &Handle<dyn Geom2dCurve>, surface: &Handle<dyn GeomSurface>, v1: &Vertex,
            v2: &Vertex, p1: f64, p2: f64) = "BRepBuilderAPI_MakeEdge(L, S, V1, V2, p1, p2)";
    );
}
