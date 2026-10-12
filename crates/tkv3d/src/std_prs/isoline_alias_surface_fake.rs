//! Named direct GeomSurface application subclass mutates actual live input cells during its first iso call.
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg3d::geom::*;
use rustycad_tkmath::{
    geom_abs::Shape,
    gp::{Axis3, Point3, Transform3, Vector3},
    toploc::Location,
};
use std::sync::Arc;
pub(super) struct IsolineLiveInputs {
    pub source: SharedState<Handle<dyn GeomSurface>>,
    pub location: SharedState<Location>,
    pub parameters: [SharedState<Sequence<f64>>; 2],
}
pub(super) struct IsolineAliasSurfaceFake {
    parent: StandardTransient,
    plane: GeomPlane,
    pub calls: SharedState<i32>,
    pub fail_closed: SharedState<bool>,
    inputs: Arc<IsolineLiveInputs>,
}
rustycad_tkernel::impl_transient!(IsolineAliasSurfaceFake,"Martensite_IsolineAliasSurfaceFake",StandardTransient,parent,interfaces:[dyn GeomSurface,dyn GeomGeometry]);
impl IsolineAliasSurfaceFake {
    pub(super) fn new(inputs: &Arc<IsolineLiveInputs>) -> Self {
        Self {
            parent: StandardTransient::default(),
            plane: GeomPlane::new(Axis3::default()),
            calls: SharedState::new(0),
            fail_closed: SharedState::new(false),
            inputs: Arc::clone(inputs),
        }
    }
    fn changed_curve(&self, parameter: f64, is_u: bool) -> Result<Handle<dyn GeomCurve>, Failure> {
        *self.calls.write() += 1;
        *self.inputs.source.write() = Handle::new(GeomPlane::new(Axis3::default())).cast();
        let mut transform = Transform3::default();
        transform.set_translation(Vector3::new(7., -3., 2.));
        *self.inputs.location.write() = Location::new(transform);
        self.inputs.parameters[0].write().append(0.75);
        self.inputs.parameters[1].write().append(0.8);
        if is_u {
            self.plane.u_iso(parameter)
        } else {
            self.plane.v_iso(parameter)
        }
    }
}
impl GeomSurface for IsolineAliasSurfaceFake {
    fn u_reverse(&self) -> Result<(), Failure> {
        let plane = &self.plane;
        plane.u_reverse()
    }
    fn u_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        let plane = &self.plane;
        plane.u_reversed_parameter(u)
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        let plane = &self.plane;
        plane.v_reverse()
    }
    fn v_reversed_parameter(&self, v: f64) -> Result<f64, Failure> {
        let plane = &self.plane;
        plane.v_reversed_parameter(v)
    }
    fn bounds(&self) -> SurfaceBounds {
        let plane = &self.plane;
        plane.bounds()
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        if *self.fail_closed.read() {
            return Err(Failure::new(
                "isoline closed callback=true; expected successful closure query",
            ));
        }
        self.plane.is_u_closed()
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        let plane = &self.plane;
        plane.is_v_closed()
    }
    fn is_u_periodic(&self) -> bool {
        let plane = &self.plane;
        plane.is_u_periodic()
    }
    fn is_v_periodic(&self) -> bool {
        let plane = &self.plane;
        plane.is_v_periodic()
    }
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        // Direct GeomSurface hooks are overridable, unlike GeomPlane's final implementations.
        self.changed_curve(u, true)
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        // Direct GeomSurface hooks are overridable, unlike GeomPlane's final implementations.
        self.changed_curve(v, false)
    }
    fn continuity(&self) -> Shape {
        let plane = &self.plane;
        plane.continuity()
    }
    fn is_cn_u(&self, n: i32) -> Result<bool, Failure> {
        let plane = &self.plane;
        plane.is_cn_u(n)
    }
    fn is_cn_v(&self, n: i32) -> Result<bool, Failure> {
        let plane = &self.plane;
        plane.is_cn_v(n)
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let plane = &self.plane;
        plane.eval_d0(u, v)
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let plane = &self.plane;
        plane.eval_d1(u, v)
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let plane = &self.plane;
        plane.eval_d2(u, v)
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        let plane = &self.plane;
        plane.eval_d3(u, v)
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        let plane = &self.plane;
        plane.eval_dn(u, v, nu, nv)
    }
}
impl GeomGeometry for IsolineAliasSurfaceFake {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let plane = &self.plane;
        plane.transform(transform)
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let plane = &self.plane;
        plane.copy()
    }
}
