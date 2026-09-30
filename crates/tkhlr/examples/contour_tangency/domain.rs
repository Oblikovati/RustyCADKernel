//! Domain subclass varies only virtual orientations; all geometry and searches remain native.
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dTopolTool,
    top_abs::{Orientation, State},
};
use rustycad_tkgeomalgo::int_start::{TheArc, TheTopolTool, TheVertex};
use rustycad_tkmath::gp::{Point2, Point3};
pub(super) struct OrientedDomain {
    pub tool: Adaptor3dTopolTool,
    pub mode: i32,
}
impl TheTopolTool for OrientedDomain {
    fn nb_samples_u(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples_u(&self.tool)
    }
    fn nb_samples_v(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples_v(&self.tool)
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        TheTopolTool::nb_samples(&self.tool)
    }
    fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure> {
        TheTopolTool::sample_point(&self.tool, index)
    }
    fn classify(&self, point: Point2, tol: f64, reframe: bool) -> Result<State, Failure> {
        TheTopolTool::classify(&self.tool, point, tol, reframe)
    }
    fn init(&self) {
        TheTopolTool::init(&self.tool);
    }
    fn more(&self) -> bool {
        TheTopolTool::more(&self.tool)
    }
    fn value(&self) -> Result<TheArc, Failure> {
        TheTopolTool::value(&self.tool)
    }
    fn next(&self) {
        TheTopolTool::next(&self.tool);
    }
    fn initialize_arc(&self, arc: &TheArc) -> Result<(), Failure> {
        self.tool.initialize_arc(arc)
    }
    fn init_vertex_iterator(&self) -> Result<(), Failure> {
        TheTopolTool::init_vertex_iterator(&self.tool)
    }
    fn more_vertex(&self) -> bool {
        TheTopolTool::more_vertex(&self.tool)
    }
    fn vertex(&self) -> Result<TheVertex, Failure> {
        TheTopolTool::vertex(&self.tool)
    }
    fn next_vertex(&self) -> Result<(), Failure> {
        TheTopolTool::next_vertex(&self.tool)
    }
    fn identical(&self, first: &TheVertex, second: &TheVertex) -> Result<bool, Failure> {
        TheTopolTool::identical(&self.tool, first, second)
    }
    fn edge(&self) -> Result<Option<Edge>, Failure> {
        TheTopolTool::edge(&self.tool)
    }
    fn is_the_point_on(&self, point: Point2, tol: f64, reframe: bool) -> Result<bool, Failure> {
        TheTopolTool::is_the_point_on(&self.tool, point, tol, reframe)
    }
    fn domain_is_infinite(&self) -> Result<bool, Failure> {
        TheTopolTool::domain_is_infinite(&self.tool)
    }
    fn sample_pnts(&self, deflection: f64, nu_min: i32, nv_min: i32) -> Result<(), Failure> {
        TheTopolTool::sample_pnts(&self.tool, deflection, nu_min, nv_min)
    }
    fn u_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        TheTopolTool::u_parameters(&self.tool)
    }
    fn v_parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        TheTopolTool::v_parameters(&self.tool)
    }
    fn is_uniform_sampling(&self) -> Result<bool, Failure> {
        TheTopolTool::is_uniform_sampling(&self.tool)
    }
    fn arc_orientation(&self, arc: &TheArc) -> Result<Orientation, Failure> {
        Ok(match self.mode % 5 {
            1 => Orientation::Reversed,
            2 => Orientation::Internal,
            3 => Orientation::External,
            4 => {
                if arc.first_parameter()? < 0. {
                    Orientation::Reversed
                } else {
                    Orientation::Forward
                }
            }
            _ => self.tool.curve_orientation(arc),
        })
    }
    fn vertex_orientation(&self, vertex: &TheVertex) -> Result<Orientation, Failure> {
        let orientation = TheTopolTool::vertex_orientation(&self.tool, vertex)?;
        Ok(if self.mode >= 5 {
            match orientation {
                Orientation::Forward => Orientation::Reversed,
                Orientation::Reversed => Orientation::Forward,
                other => other,
            }
        } else {
            orientation
        })
    }
}
