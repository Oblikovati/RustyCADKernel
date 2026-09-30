// Rust translation of ModelingData/TKG3d/Geom/Geom_CylindricalSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_CylindricalSurface`: an infinite circular cylinder, U the angle around
//! the main axis and V the height along it.
use super::circle::GeomCircle;
use super::curve::GeomCurve;
use super::elementary_results::{surface_d2, surface_d3};
use super::elementary_surface::{
    GeomElementarySurface, dump_elementary_surface, elementary_u_reverse, elementary_v_reverse,
    impl_elementary_position,
};
use super::errors;
use super::geometry::GeomGeometry;
use super::line::GeomLine;
use super::parametric::{scaled_parameter, v_affinity};
use super::quadric::cylinder_coefficients;
use super::surface::{
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds, check_dn_orders,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis3, Cylinder3, GeneralTransform2, Point3, Transform3, Vector3};
use std::f64::consts::PI;
use std::io;

#[derive(Clone, Copy)]
struct CylinderState {
    pos: Axis3,
    radius: f64,
}

/// A cylinder; `Handle::new(GeomCylindricalSurface::new(frame, 2.0)?)`.
pub struct GeomCylindricalSurface {
    parent: StandardTransient,
    state: SharedState<CylinderState>,
}
impl_transient!(
    GeomCylindricalSurface,
    "Geom_CylindricalSurface",
    dyn GeomElementarySurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomElementarySurface]
);
impl Clone for GeomCylindricalSurface {
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

fn checked_radius(radius: f64, member: &str) -> Result<f64, Failure> {
    if radius < 0.0 {
        return Err(errors::construction(&format!(
            "Geom_CylindricalSurface::{member}: Radius = {radius}; expected Radius >= 0"
        )));
    }
    Ok(radius)
}

impl GeomCylindricalSurface {
    fn stored(state: CylinderState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Cylinder of `radius` around the main axis of `frame`; Standard_ConstructionError
    /// when `radius < 0` (`Geom_CylindricalSurface(gp_Ax3, Radius)`);
    /// `GeomCylindricalSurface::new(frame, 2.0)?`.
    pub fn new(frame: Axis3, radius: f64) -> Result<Self, Failure> {
        let radius: f64 = checked_radius(radius, "Geom_CylindricalSurface")?;
        Ok(Self::stored(CylinderState { pos: frame, radius }))
    }
    /// Cylinder from a gp cylinder (`Geom_CylindricalSurface(gp_Cylinder)`);
    /// `GeomCylindricalSurface::from_cylinder(c)`.
    pub fn from_cylinder(cylinder: Cylinder3) -> Self {
        Self::stored(CylinderState {
            pos: cylinder.position(),
            radius: cylinder.radius(),
        })
    }
    /// Replace by a gp cylinder (`SetCylinder`); `surface.set_cylinder(c)`.
    pub fn set_cylinder(&self, cylinder: Cylinder3) {
        let mut state = self.state.write();
        state.radius = cylinder.radius();
        state.pos = cylinder.position();
    }
    /// Set the radius; Standard_ConstructionError when negative (`SetRadius`);
    /// `surface.set_radius(3.0)?`.
    pub fn set_radius(&self, radius: f64) -> Result<(), Failure> {
        self.state.write().radius = checked_radius(radius, "SetRadius")?;
        Ok(())
    }
    /// The gp cylinder (`Cylinder`); `surface.cylinder()?`.
    pub fn cylinder(&self) -> Result<Cylinder3, Failure> {
        let state: CylinderState = *self.state.read();
        Cylinder3::new(state.pos, state.radius)
    }
    /// The implicit equation coefficients `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]`
    /// (`Coefficients`); `surface.coefficients()`.
    pub fn coefficients(&self) -> [f64; 10] {
        let state: CylinderState = *self.state.read();
        cylinder_coefficients(state.pos, state.radius)
    }
    /// The radius (`Radius`); `surface.radius()`.
    pub fn radius(&self) -> f64 {
        self.state.read().radius
    }
    fn snapshot(&self) -> CylinderState {
        *self.state.read()
    }
}
impl GeomGeometry for GeomCylindricalSurface {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.radius *= transform.scale_factor().abs();
        state.pos.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state: CylinderState = self.snapshot();
        dump_class_name(stream, "Geom_CylindricalSurface")?;
        dump_base(stream, depth, |s, d| {
            dump_elementary_surface(s, d, state.pos)
        })?;
        dump::dump_real(stream, "radius", state.radius)
    }
}
impl GeomSurface for GeomCylindricalSurface {
    fn u_reverse(&self) -> Result<(), Failure> {
        elementary_u_reverse(&mut self.state.write().pos);
        Ok(())
    }
    fn u_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(2. * PI - u)
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        elementary_v_reverse(&mut self.state.write().pos);
        Ok(())
    }
    fn v_reversed_parameter(&self, v: f64) -> Result<f64, Failure> {
        Ok(-v)
    }
    fn transform_parameters(&self, u: f64, v: f64, transform: Transform3) -> (f64, f64) {
        (u, scaled_parameter(v, transform))
    }
    fn parametric_transformation(
        &self,
        transform: Transform3,
    ) -> Result<GeneralTransform2, Failure> {
        Ok(v_affinity(transform.scale_factor().abs()))
    }
    fn bounds(&self) -> SurfaceBounds {
        (0.0, 2.0 * PI, -precision::infinite(), precision::infinite())
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_u_periodic(&self) -> bool {
        true
    }
    fn is_v_periodic(&self) -> bool {
        false
    }
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let state: CylinderState = self.snapshot();
        let line = elslib::cylinder_u_iso(state.pos, state.radius, u);
        Ok(Handle::new(GeomLine::from_lin(line)).cast())
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let state: CylinderState = self.snapshot();
        let circle = elslib::cylinder_v_iso(state.pos, state.radius, v);
        Ok(Handle::new(GeomCircle::from_circ(circle)).cast())
    }
    fn continuity(&self) -> Shape {
        Shape::CN
    }
    fn is_cn_u(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_cn_v(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let state: CylinderState = self.snapshot();
        Ok(elslib::cylinder_d0(u, v, state.pos, state.radius))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let state: CylinderState = self.snapshot();
        let (point, d1u, d1v) = elslib::cylinder_d1(u, v, state.pos, state.radius);
        Ok(GeomSurfaceD1 { point, d1u, d1v })
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let state: CylinderState = self.snapshot();
        Ok(surface_d2(elslib::cylinder_d2(
            u,
            v,
            state.pos,
            state.radius,
        )))
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        let state: CylinderState = self.snapshot();
        Ok(surface_d3(elslib::cylinder_d3(
            u,
            v,
            state.pos,
            state.radius,
        )))
    }
    /// Orders above 1 in V are zero; the others come from ElSLib::CylinderDN.
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_CylindricalSurface", nu, nv)?;
        if nv > 1 {
            return Ok(Vector3::new(0.0, 0.0, 0.0));
        }
        let state: CylinderState = self.snapshot();
        Ok(elslib::cylinder_dn(u, v, state.pos, state.radius, nu, nv))
    }
}
impl GeomElementarySurface for GeomCylindricalSurface {
    impl_elementary_position!();
}
