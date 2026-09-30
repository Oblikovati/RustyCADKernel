// Rust translation of the virtual dispatch of BRepTopAdaptor_TopolTool.hxx (the
// overrides of Adaptor3d_TopolTool) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTopAdaptor_TopolTool` as the domain of IntStart and IntPatch: the IntStart trait
//! `TheTopolTool` is the virtual interface of `Adaptor3d_TopolTool`
//! (docs/topalgo-design.md section 8), and each member maps to the override the native
//! virtual call reaches, or to the inherited base member.
use super::topol_tool::BRepTopAdaptorTopolTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::int_start::{TheArc, TheTopolTool, TheVertex};
use rustycad_tkmath::gp::{Point2, Point3};

impl TheTopolTool for BRepTopAdaptorTopolTool {
    fn nb_samples_u(&self) -> Result<i32, Failure> {
        BRepTopAdaptorTopolTool::nb_samples_u(self)
    }
    fn nb_samples_v(&self) -> Result<i32, Failure> {
        BRepTopAdaptorTopolTool::nb_samples_v(self)
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        BRepTopAdaptorTopolTool::nb_samples(self)
    }
    fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure> {
        BRepTopAdaptorTopolTool::sample_point(self, index)
    }
    fn classify(&self, point: Point2, tol: f64, reframe: bool) -> Result<State, Failure> {
        BRepTopAdaptorTopolTool::classify(self, point, tol, reframe)
    }
    fn init(&self) {
        BRepTopAdaptorTopolTool::init(self);
    }
    fn more(&self) -> bool {
        BRepTopAdaptorTopolTool::more(self)
    }
    fn value(&self) -> Result<TheArc, Failure> {
        BRepTopAdaptorTopolTool::value(self)
    }
    fn next(&self) {
        BRepTopAdaptorTopolTool::next(self);
    }
    fn initialize_arc(&self, arc: &TheArc) -> Result<(), Failure> {
        self.initialize_curve(arc)
    }
    fn init_vertex_iterator(&self) -> Result<(), Failure> {
        BRepTopAdaptorTopolTool::init_vertex_iterator(self)
    }
    fn more_vertex(&self) -> bool {
        BRepTopAdaptorTopolTool::more_vertex(self)
    }
    fn vertex(&self) -> Result<TheVertex, Failure> {
        BRepTopAdaptorTopolTool::vertex(self)
    }
    fn next_vertex(&self) -> Result<(), Failure> {
        BRepTopAdaptorTopolTool::next_vertex(self)
    }
    fn identical(&self, first: &TheVertex, second: &TheVertex) -> Result<bool, Failure> {
        BRepTopAdaptorTopolTool::identical(self, first, second)
    }
    // The native `Edge()` points at the edge of the current arc; past the last arc it
    // dereferences a null handle, which is the Standard_NoSuchObject of the struct.
    fn edge(&self) -> Result<Option<Edge>, Failure> {
        BRepTopAdaptorTopolTool::edge(self).map(Some)
    }
    fn is_the_point_on(&self, point: Point2, tol: f64, reframe: bool) -> Result<bool, Failure> {
        BRepTopAdaptorTopolTool::is_the_point_on(self, point, tol, reframe)
    }
    fn domain_is_infinite(&self) -> Result<bool, Failure> {
        BRepTopAdaptorTopolTool::domain_is_infinite(self)
    }
    fn sample_pnts(&self, deflection: f64, nu_min: i32, nv_min: i32) -> Result<(), Failure> {
        BRepTopAdaptorTopolTool::sample_pnts(self, deflection, nu_min, nv_min)
    }
    fn u_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        BRepTopAdaptorTopolTool::u_parameters(self)
    }
    fn v_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        BRepTopAdaptorTopolTool::v_parameters(self)
    }
    fn is_uniform_sampling(&self) -> Result<bool, Failure> {
        BRepTopAdaptorTopolTool::is_uniform_sampling(self)
    }
    fn has_3d(&self) -> bool {
        BRepTopAdaptorTopolTool::has_3d(self)
    }
    fn tol_3d_arc(&self, arc: &TheArc) -> Result<f64, Failure> {
        self.curve_tol_3d(arc)
    }
    fn tol_3d_vertex(&self, vertex: &TheVertex) -> Result<f64, Failure> {
        self.vertex_tol_3d(vertex)
    }
    fn arc_orientation(&self, arc: &TheArc) -> Result<Orientation, Failure> {
        self.curve_orientation(arc)
    }
    fn vertex_orientation(&self, vertex: &TheVertex) -> Result<Orientation, Failure> {
        BRepTopAdaptorTopolTool::vertex_orientation(self, vertex)
    }
}
