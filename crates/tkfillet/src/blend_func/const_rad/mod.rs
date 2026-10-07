// Rust translation of BlendFunc_ConstRad.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constant-radius fillet function between two surfaces along a guide.
mod compute;
mod equations;
mod queries;
mod second_order;
mod sections;
mod solution;
mod traits;

use super::SectionShape;
use super::guide::PlaneFrame;
use super::linalg::vector;
use super::normal_terms::SurfaceJet;
use super::section_solve::Equations;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};
use rustycad_tkmath::math::Vector;

/// The derivatives of the projected normal of one surface that `ComputeValues` keeps
/// (`dndu1`, `dndv1`, `d2ndu1`, ... `dn1w`, `d2n1w` for the first surface).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NormalDerivatives {
    pub dndu: Vector3,
    pub dndv: Vector3,
    pub d2ndu: Vector3,
    pub d2ndv: Vector3,
    pub d2nduv: Vector3,
    pub d2ndtu: Vector3,
    pub d2ndtv: Vector3,
    pub dnw: Vector3,
    pub d2nw: Vector3,
}

/// The values of one surface at the current point: the point, its derivatives up to
/// order 2 (order 3 sits in the thread scratch), the normal `D1U ^ D1V` and its first
/// derivatives (`dns1u1`, `dns1v1`; named `dns1u2`, `dns1v2` natively for the second).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SurfacePoint {
    pub pts: Point3,
    pub jet: SurfaceJet,
    pub nsurf: Vector3,
    pub dnsu: Vector3,
    pub dnsv: Vector3,
    pub normal: NormalDerivatives,
}

/// `BlendFunc_ConstRad`: the fillet of constant radius between two surfaces. The
/// variables are `(U1, V1, U2, V2)`; the four equations put the middle of the two contact
/// points in the plane normal to the guide and make the two points the ends of a circular
/// section of the radius, centred on both surface offsets.
///
/// `ComputeValues` caches its last point and parameter; as natively, the guide values and
/// the third surface derivatives are thread-wide scratch shared by every `ConstRad`.
///
/// ```ignore
/// let mut function = ConstRad::new(&surface1, &surface2, &guide)?;
/// function.set_radius(2.0, 1);
/// ```
pub struct ConstRad {
    surf1: Handle<dyn Adaptor3dSurface>,
    surf2: Handle<dyn Adaptor3dSurface>,
    curv: Handle<dyn Adaptor3dCurve>,
    tcurv: Handle<dyn Adaptor3dCurve>,
    s1: SurfacePoint,
    s2: SurfacePoint,
    istangent: bool,
    tg1: Vector3,
    tg12d: Vector2,
    tg2: Vector3,
    tg22d: Vector2,
    param: f64,
    ray1: f64,
    ray2: f64,
    choix: i32,
    x_order: i32,
    t_order: i32,
    xval: Vector<'static>,
    tval: f64,
    plane: PlaneFrame,
    eq: Equations,
    maxang: f64,
    minang: f64,
    distmin: f64,
    shape: SectionShape,
    tconv: ParameterisationType,
    capture: FailureCapture,
}

impl ConstRad {
    /// `BlendFunc_ConstRad(S1, S2, C)`: the function on the surfaces `s1`, `s2` along the
    /// guide `c`, radius 0, rational sections; `ConstRad::new(&s1, &s2, &c)?`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Self, Failure> {
        let mut xval = vector(4)?;
        // Initialisation of cash control variables.
        xval.init(-9.876e100);
        Ok(Self {
            surf1: s1.clone(),
            surf2: s2.clone(),
            curv: c.clone(),
            tcurv: c.clone(),
            s1: SurfacePoint::default(),
            s2: SurfacePoint::default(),
            istangent: true,
            tg1: Vector3::default(),
            tg12d: Vector2::default(),
            tg2: Vector3::default(),
            tg22d: Vector2::default(),
            param: 0.0,
            ray1: 0.0,
            ray2: 0.0,
            choix: 0,
            x_order: -1,
            t_order: -1,
            xval,
            tval: -9.876e100,
            plane: PlaneFrame::default(),
            eq: Equations::new()?,
            maxang: f64::MIN,
            minang: f64::MAX,
            distmin: f64::MAX,
            shape: SectionShape::Rational,
            tconv: ParameterisationType::TgtThetaOver2,
            capture: FailureCapture::default(),
        })
    }

    /// `Set(Radius, Choix)`: the radius and the choice of the solution, which signs the
    /// offsets of both surfaces; `function.set_radius(2.0, 1)`.
    pub fn set_radius(&mut self, radius: f64, choix: i32) {
        self.choix = choix;
        (self.ray1, self.ray2) = signed_radii(radius, choix);
    }

    /// `Set(TypeSection)`: the shape of the sections; `function.set_shape(SectionShape::Linear)`.
    pub fn set_shape(&mut self, shape: SectionShape) {
        self.shape = shape;
    }
}

/// The signed radii `(ray1, ray2)` of a choice, as `Set(Radius, Choix)` assigns them.
pub(crate) fn signed_radii(radius: f64, choix: i32) -> (f64, f64) {
    match choix {
        1 | 2 => (-radius, -radius),
        3 | 4 => (radius, -radius),
        5 | 6 => (radius, radius),
        7 | 8 => (-radius, radius),
        _ => (-radius, -radius),
    }
}

#[cfg(test)]
#[path = "const_rad_tests.rs"]
mod tests;
