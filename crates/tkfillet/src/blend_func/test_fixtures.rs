//! The inputs the BlendFunc unit tests share.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

/// The surfaces and the guide of a blend between two surfaces.
pub(crate) type BlendInputs = (
    Handle<dyn Adaptor3dSurface>,
    Handle<dyn Adaptor3dSurface>,
    Handle<dyn Adaptor3dCurve>,
);

fn plane(normal: Direction3, x: Direction3) -> Handle<dyn Adaptor3dSurface> {
    let frame = Axis3::new(Point3::default(), normal, x).unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    Handle::new(GeomAdaptorSurface::with_surface(&surface).unwrap()).cast()
}

/// Two planes meeting at a right angle along the X axis: z = 0 (normal +Z, point
/// `(u, v, 0)`) and y = 0 (normal +Y, point `(u, 0, -v)`), with the X axis as guide.
pub(crate) fn right_angle() -> BlendInputs {
    let x = Direction3::new(1., 0., 0.).unwrap();
    let s1 = plane(Direction3::new(0., 0., 1.).unwrap(), x);
    let s2 = plane(Direction3::new(0., 1., 0.).unwrap(), x);
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(Point3::default(), x)).cast();
    let guide: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_curve(&line).unwrap()).cast();
    (s1, s2, guide)
}
