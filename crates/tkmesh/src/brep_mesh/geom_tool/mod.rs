//! Native BRepMesh geometry utilities, preserving output mutation and sampling order.
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dIsoCurve, Adaptor3dSurface};
use rustycad_tkgeombase::gcpnts::{TangentialCriteria, TangentialDeflection};
use rustycad_tkmath::{
    geom_abs::IsoType,
    gp::{Point2, Point3},
};
mod cells;
mod intersection;
mod normal;
pub use intersection::IntersectionFlag;
/// Sampled edge/iso geometry; `GeomTool::new(&curve,(0.0,1.0),0.1,0.5,2,1e-7)?`.
pub struct GeomTool<'a> {
    // Native retains the adaptor's edge member by address; later initialization is visible.
    curve: Option<&'a BRepAdaptorCurve>,
    distribution: TangentialDeflection,
    iso: IsoType,
}
impl<'a> GeomTool<'a> {
    /// Discretize a live edge adaptor; `GeomTool::new(&curve,(0.0,1.0),0.1,0.5,2,1e-7)?`.
    pub fn new(
        curve: &'a BRepAdaptorCurve,
        range: (f64, f64),
        linear: f64,
        angular: f64,
        minimum: i32,
        min_size: f64,
    ) -> Result<Self, Failure> {
        let distribution = discretize(curve, range, linear, angular, minimum, min_size)?;
        Ok(Self {
            curve: Some(curve),
            distribution,
            iso: IsoType::NoneIso,
        })
    }
    /// Discretize a surface isoparameter; `GeomTool::on_surface(&surface,IsoType::IsoU,0.0,(0.0,1.0),(0.1,0.5),2,1e-7)?`.
    pub fn on_surface(
        surface: &Handle<BRepAdaptorSurface>,
        iso: IsoType,
        parameter: f64,
        range: (f64, f64),
        deflections: (f64, f64),
        minimum: i32,
        min_size: f64,
    ) -> Result<Self, Failure> {
        let curve = Adaptor3dIsoCurve::with_iso_range(&surface.cast(), iso, parameter, range)?;
        let distribution = discretize(
            &curve,
            range,
            deflections.0,
            deflections.1,
            minimum,
            min_size,
        )?;
        Ok(Self {
            curve: None,
            distribution,
            iso,
        })
    }
    /// Insert or replace by parametric tolerance; `tool.add_point(point,0.5,true)`.
    pub fn add_point(
        &mut self,
        point: Point3,
        parameter: f64,
        replace: bool,
    ) -> Result<i32, Failure> {
        self.distribution.add_point(point, parameter, replace)
    }
    /// Count one-based samples; `tool.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.distribution.nb_points()
    }
    /// Read a sample and construct its UV coordinates; `tool.value_iso(1,0.0,&mut parameter,&mut point,&mut uv)?`.
    pub fn value_iso(
        &self,
        index: i32,
        iso_parameter: f64,
        parameter: &mut f64,
        point: &mut Point3,
        uv: &mut Point2,
    ) -> Result<bool, Failure> {
        if index < 1 || index > self.nb_points() {
            return Ok(false);
        }
        *point = self.distribution.value(index)?;
        *parameter = self.distribution.parameter(index)?;
        *uv = if self.iso == IsoType::IsoU {
            Point2::new(iso_parameter, *parameter)
        } else {
            Point2::new(*parameter, iso_parameter)
        };
        Ok(true)
    }
    /// Read the edge's live pcurve on a face; `tool.value_surface(1,&surface,&mut parameter,&mut point,&mut uv)?`.
    pub fn value_surface(
        &self,
        index: i32,
        surface: &Handle<BRepAdaptorSurface>,
        parameter: &mut f64,
        point: &mut Point3,
        uv: &mut Point2,
    ) -> Result<bool, Failure> {
        if index < 1 || index > self.nb_points() {
            return Ok(false);
        }
        let Some(curve) = self.curve else {
            return Ok(false);
        };
        *point = self.distribution.value(index)?;
        *parameter = self.distribution.parameter(index)?;
        let pcurve = BRepTool::curve_on_face(&curve.edge(), &surface.face())?.pcurve;
        *uv = pcurve.value(*parameter)?;
        Ok(true)
    }
}
fn discretize(
    curve: &dyn Adaptor3dCurve,
    range: (f64, f64),
    linear: f64,
    angular: f64,
    minimum: i32,
    min_size: f64,
) -> Result<TangentialDeflection, Failure> {
    let criteria = TangentialCriteria {
        angular_deflection: angular,
        curvature_deflection: linear,
        minimum_of_points: minimum,
        u_tol: precision::p_confusion(),
        min_len: min_size,
    };
    TangentialDeflection::with_range(curve, range.0, range.1, criteria)
}
