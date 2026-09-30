//! A virtual adaptor exposing precisely the state Contap samples, including failures.
use super::common::Reader;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dSurface, h_surface_tool::Adaptor3dSurfaceHandle};
use rustycad_tkmath::geom_abs::SurfaceType;
struct ContourSamplingSurface {
    parent: StandardTransient,
    kind: SurfaceType,
    bounds: [f64; 4],
    counts: [i32; 6],
    failure_field: i32,
}
impl_transient!(ContourSamplingSurface, "ContourSamplingSurface", dyn Adaptor3dSurface, parent, interfaces: [dyn Adaptor3dSurface]);
impl ContourSamplingSurface {
    fn check_field(&self, field: i32) -> Result<(), Failure> {
        if self.failure_field == field {
            return Err(Failure::with_message(
                "Standard_DomainError",
                "contour probe rejected field; expected readable adaptor field",
            ));
        }
        Ok(())
    }
    fn bound(&self, index: usize, field: i32) -> Result<f64, Failure> {
        self.check_field(field)?;
        Ok(self.bounds[index])
    }
    fn count(&self, index: usize, field: i32) -> Result<i32, Failure> {
        self.check_field(field)?;
        Ok(self.counts[index])
    }
}
impl Adaptor3dSurface for ContourSamplingSurface {
    fn first_u_parameter(&self) -> Result<f64, Failure> {
        self.bound(0, 1)
    }
    fn last_u_parameter(&self) -> Result<f64, Failure> {
        self.bound(2, 2)
    }
    fn first_v_parameter(&self) -> Result<f64, Failure> {
        self.bound(1, 3)
    }
    fn last_v_parameter(&self) -> Result<f64, Failure> {
        self.bound(3, 4)
    }
    fn get_type(&self) -> Result<SurfaceType, Failure> {
        self.check_field(5)?;
        Ok(self.kind)
    }
    fn nb_u_knots(&self) -> Result<i32, Failure> {
        self.count(0, 6)
    }
    fn u_degree(&self) -> Result<i32, Failure> {
        self.count(2, 7)
    }
    fn nb_v_knots(&self) -> Result<i32, Failure> {
        self.count(1, 8)
    }
    fn v_degree(&self) -> Result<i32, Failure> {
        self.count(3, 9)
    }
    fn nb_u_poles(&self) -> Result<i32, Failure> {
        self.count(4, 10)
    }
    fn nb_v_poles(&self) -> Result<i32, Failure> {
        self.count(5, 11)
    }
}
/// Read a virtual surface with original field values; `sampling_surface(reader)`.
pub fn sampling_surface(reader: &mut Reader) -> Adaptor3dSurfaceHandle {
    let kinds = [
        SurfaceType::Plane,
        SurfaceType::Cylinder,
        SurfaceType::Cone,
        SurfaceType::Sphere,
        SurfaceType::Torus,
        SurfaceType::BezierSurface,
        SurfaceType::BSplineSurface,
        SurfaceType::SurfaceOfRevolution,
        SurfaceType::SurfaceOfExtrusion,
        SurfaceType::OffsetSurface,
        SurfaceType::OtherSurface,
    ];
    let kind = kinds[reader.integer() as usize];
    let bounds = std::array::from_fn(|_| reader.real());
    let counts = std::array::from_fn(|_| reader.integer());
    let failure_field = reader.integer();
    Handle::new(ContourSamplingSurface {
        parent: StandardTransient::default(),
        kind,
        bounds,
        counts,
        failure_field,
    })
    .cast()
}
