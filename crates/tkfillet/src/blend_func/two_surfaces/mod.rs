// The common body of BlendFunc_ConstRad.cxx and BlendFunc_EvolRad.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The machinery the constant- and evolutive-radius fillet functions share: the cached
//! positioning on both surfaces, the equations and their derivatives, the solution test,
//! the sections and the queries. Where the two native classes differ, the code branches
//! on [`Radii`], so each keeps its own operation order.
mod compute;
mod equations;
mod queries;
pub(crate) use queries::arc_tolerances;
mod second_order;
mod sections;
mod sections_d2;
mod solution;

pub(crate) use compute::SurfaceScratch;
pub(crate) use sections::{CircleSection, point_parameters};

use super::SectionShape;
use super::guide::PlaneFrame;
use super::linalg::vector;
use super::normal_terms::SurfaceJet;
use super::section_solve::Equations;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
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

/// The radii of a section: the two signed constant radii of `BlendFunc_ConstRad`, or the
/// signs, the law value and its derivatives of `BlendFunc_EvolRad`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Radii {
    Constant {
        ray1: f64,
        ray2: f64,
    },
    Evolutive {
        sg1: f64,
        sg2: f64,
        ray: f64,
        dray: f64,
        d2ray: f64,
    },
}

impl Radii {
    /// `ray1` (`sg1 * ray` for an evolutive radius).
    pub(crate) fn ray1(&self) -> f64 {
        match *self {
            Self::Constant { ray1, .. } => ray1,
            Self::Evolutive { sg1, ray, .. } => sg1 * ray,
        }
    }

    /// `ray2` (`sg2 * ray` for an evolutive radius).
    pub(crate) fn ray2(&self) -> f64 {
        match *self {
            Self::Constant { ray2, .. } => ray2,
            Self::Evolutive { sg2, ray, .. } => sg2 * ray,
        }
    }

    /// The orientation tests of the sections: `ray1 > 0` (`sg1 > 0` for an evolutive
    /// radius, whatever the sign of the law).
    pub(crate) fn positive(&self, second: bool) -> bool {
        match *self {
            Self::Constant { ray1, ray2 } => {
                if second {
                    ray2 > 0.
                } else {
                    ray1 > 0.
                }
            }
            Self::Evolutive { sg1, sg2, .. } => {
                if second {
                    sg2 > 0.
                } else {
                    sg1 > 0.
                }
            }
        }
    }

    /// The radius of the section circle: `|ray1|` (`|ray|`).
    pub(crate) fn circle_radius(&self) -> f64 {
        match *self {
            Self::Constant { ray1, .. } => ray1.abs(),
            Self::Evolutive { ray, .. } => ray.abs(),
        }
    }
}

/// The state both fillet functions keep: surfaces, cached point and parameter, the
/// current solution and its tangents, the stored equations and the approximation data.
pub(crate) struct TwoSurfaceCore {
    pub surf1: Handle<dyn Adaptor3dSurface>,
    pub surf2: Handle<dyn Adaptor3dSurface>,
    pub s1: SurfacePoint,
    pub s2: SurfacePoint,
    pub istangent: bool,
    pub tg1: Vector3,
    pub tg12d: Vector2,
    pub tg2: Vector3,
    pub tg22d: Vector2,
    pub param: f64,
    pub choix: i32,
    pub x_order: i32,
    pub t_order: i32,
    pub xval: Vector<'static>,
    pub tval: f64,
    pub plane: PlaneFrame,
    pub eq: Equations,
    pub maxang: f64,
    pub minang: f64,
    pub distmin: f64,
    pub shape: SectionShape,
    pub tconv: ParameterisationType,
}

impl TwoSurfaceCore {
    /// The state of a new function: the cache control variables at -9.876e100.
    pub(crate) fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<Self, Failure> {
        let mut xval = vector(4)?;
        // Initialisation of cash control variables.
        xval.init(-9.876e100);
        Ok(Self {
            surf1: s1.clone(),
            surf2: s2.clone(),
            s1: SurfacePoint::default(),
            s2: SurfacePoint::default(),
            istangent: true,
            tg1: Vector3::default(),
            tg12d: Vector2::default(),
            tg2: Vector3::default(),
            tg22d: Vector2::default(),
            param: 0.0,
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
        })
    }

    /// The tangent query of a class: Standard_DomainError with `message` on a tangency
    /// point.
    pub(crate) fn when_not_tangent<T>(&self, value: T, message: &str) -> Result<T, Failure> {
        if self.istangent {
            return Err(Failure::with_message("Standard_DomainError", message));
        }
        Ok(value)
    }
}
