use super::algo_tools2d;
use crate::int_tools::{Context, Curve};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomTrimmedCurve},
    top_abs::Orientation,
};
use rustycad_tktopalgo::brep_lib;
struct EdgeProjection<'a> {
    edge: &'a Edge,
    spatial: Handle<dyn GeomCurve>,
    trimmed: Handle<dyn GeomCurve>,
    range: (f64, f64),
    tolerance: f64,
    context: Option<&'a Context>,
}
/// Install selected intersection pcurves and reconcile edge parameters; `make_pcurve(&edge,&face1,&face2,&curve,true,true,None)?`.
pub fn make_pcurve(
    edge: &Edge,
    first: &Face,
    second: &Face,
    curve: &Curve,
    use_first: bool,
    use_second: bool,
    context: Option<&Context>,
) -> Result<(), Failure> {
    let projection = EdgeProjection::new(edge, context)?;
    if use_first {
        projection.install(first, curve.first_curve2d())?;
    }
    if use_second {
        projection.install(second, curve.second_curve2d())?;
    }
    brep_lib::same_parameter_edge(edge, 1e-5)
}
impl<'a> EdgeProjection<'a> {
    fn new(edge: &'a Edge, context: Option<&'a Context>) -> Result<Self, Failure> {
        let tolerance = BRepTool::edge_tolerance(edge)?;
        let (spatial, first, last) = BRepTool::curve(edge)?;
        let trimmed = Handle::new(GeomTrimmedCurve::new(&spatial, first, last, true, true)?).cast();
        Ok(Self {
            edge,
            spatial,
            trimmed,
            range: (first, last),
            tolerance,
            context,
        })
    }
    fn install(&self, face: &Face, input: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        let mut face = face.clone();
        face.set_orientation(Orientation::Forward);
        let curve = self.input_curve(&face, input)?;
        let adjusted = self.adjusted_curve(&face, &curve)?;
        BRepBuilder::default().update_edge_pcurve_on_face(
            self.edge,
            &adjusted,
            &face,
            self.tolerance,
        )
    }
    fn adjusted_curve(
        &self,
        face: &Face,
        curve: &Handle<dyn Geom2dCurve>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        if self.spatial.is_periodic() {
            return algo_tools2d::adjust_pcurve_on_face(
                face,
                self.range.0,
                self.range.1,
                curve,
                self.context,
            );
        }
        algo_tools2d::adjust_pcurve_for_curve(face, &self.trimmed, curve, self.context)
    }
    fn input_curve(
        &self,
        face: &Face,
        input: &Handle<dyn Geom2dCurve>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        if !input.is_null() {
            return Ok(input.clone());
        }
        algo_tools2d::build_pcurve_for_edge_on_face(self.edge, face, self.context)?;
        let mut curve = Handle::default();
        algo_tools2d::curve_on_surface_simple(self.edge, face, &mut curve, &mut 0., self.context)?;
        Ok(curve)
    }
}
