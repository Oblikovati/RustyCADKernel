// Rust translation of IntAna_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_Curve: the parametric curve of a cylinder-quadric or cone-quadric
//! intersection. The curve at `Theta` is the root in `Z` of
//! `A(Theta) Z² + B(Theta) Z + C(Theta)` on the cylinder or the cone; this module
//! holds its definition, `evaluation` its points and `projection` the
//! parameters of a point.
mod evaluation;
mod projection;

use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis3, Cone3, Cylinder3};

/// The six coefficients of a trigonometric polynomial
/// `Cte + Sin sin + Cos cos + SinSin sin² + CosCos cos² + CosSin sin(2 Theta)`
/// (no factor 2 in front of the CosSin term).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct TrigTerms {
    pub cte: f64,
    pub sin: f64,
    pub cos: f64,
    pub sin_sin: f64,
    pub cos_cos: f64,
    pub cos_sin: f64,
}

/// IntAna_Curve: the intersection curve of a cylinder or a cone with a quadric,
/// parametrized by the angle on the cylinder or the cone.
///
/// The quadric coefficients are `[Qxx, Qyy, Qzz, Qxy, Qxz, Qyz, Qx, Qy, Qz, Q1]`
/// in the frame of the cylinder or the cone.
/// `let mut curve = Curve::new(); curve.set_cylinder_quad_values(cyl, q, 1e-8, (0.0, 1.0), false, true);`
#[derive(Clone, Copy, Debug)]
pub struct Curve {
    z0: TrigTerms,
    z1: TrigTerms,
    z2: TrigTerms,
    two_curves: bool,
    take_z_positive: bool,
    tolerance: f64,
    domain_inf: f64,
    domain_sup: f64,
    restricted_inf: bool,
    restricted_sup: bool,
    firstbounded: bool,
    lastbounded: bool,
    typequadric: SurfaceType,
    rcyl: f64,
    angle: f64,
    ax3: Axis3,
    first_parameter: f64,
    last_parameter: f64,
}

impl Default for Curve {
    /// IntAna_Curve(): every coefficient null, an unrestricted domain and no
    /// quadric (GeomAbs_OtherSurface).
    fn default() -> Self {
        Self {
            z0: TrigTerms::default(),
            z1: TrigTerms::default(),
            z2: TrigTerms::default(),
            two_curves: false,
            take_z_positive: false,
            tolerance: 0.0,
            domain_inf: 0.0,
            domain_sup: 0.0,
            restricted_inf: false,
            restricted_sup: false,
            firstbounded: false,
            lastbounded: false,
            typequadric: SurfaceType::OtherSurface,
            rcyl: 0.0,
            angle: 0.0,
            ax3: Axis3::default(),
            first_parameter: 0.0,
            last_parameter: 0.0,
        }
    }
}

impl Curve {
    /// IntAna_Curve(): the empty curve. `Curve::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// SetCylinderQuadValues: the curve of the cylinder `cylinder` and the quadric
    /// `q` (in the cylinder frame) on the domain `(DomInf, DomSup)`; `two_curves`
    /// gives two Z per Theta (the parameter runs to `2 DomSup - DomInf`),
    /// `z_positive` takes `(-B + sqrt(disc)) / 2A`. The semi-angle is kept.
    /// `curve.set_cylinder_quad_values(cyl, q, 1e-8, (0.0, 1.0), false, true)`.
    pub fn set_cylinder_quad_values(
        &mut self,
        cylinder: Cylinder3,
        q: [f64; 10],
        tolerance: f64,
        domain: (f64, f64),
        two_curves: bool,
        z_positive: bool,
    ) {
        let [qxx, qyy, qzz, qxy, qxz, qyz, qx, qy, qz, q1] = q;
        self.ax3 = cylinder.position();
        self.rcyl = cylinder.radius();
        self.typequadric = SurfaceType::Cylinder;
        let (rcyl, rcylmul2) = (self.rcyl, self.rcyl + self.rcyl);
        self.z0 = TrigTerms {
            cte: q1,
            sin: rcylmul2 * qy,
            cos: rcylmul2 * qx,
            cos_cos: qxx * rcyl * rcyl,
            sin_sin: qyy * rcyl * rcyl,
            cos_sin: rcyl * rcyl * qxy,
        };
        self.z1 = TrigTerms {
            cte: qz + qz,
            sin: rcylmul2 * qyz,
            cos: rcylmul2 * qxz,
            ..TrigTerms::default()
        };
        self.z2 = TrigTerms {
            cte: qzz,
            ..TrigTerms::default()
        };
        self.set_domain_values(tolerance, domain, two_curves, z_positive);
    }

    /// SetConeQuadValues: the curve of the cone `cone` and the quadric `q` (in the
    /// frame of the cone apex), as [`Self::set_cylinder_quad_values`]. The Z terms
    /// carry no factor 2 in front of their CosSin coefficients.
    /// `curve.set_cone_quad_values(cone, q, 1e-8, (0.0, 1.0), false, true)`.
    pub fn set_cone_quad_values(
        &mut self,
        cone: Cone3,
        q: [f64; 10],
        tolerance: f64,
        domain: (f64, f64),
        two_curves: bool,
        z_positive: bool,
    ) {
        let [qxx, qyy, qzz, qxy, qxz, qyz, qx, qy, qz, q1] = q;
        self.ax3 = cone.position();
        self.rcyl = cone.ref_radius();
        self.angle = cone.semi_angle();
        let un_sur_tg: f64 = 1.0 / cone.semi_angle().tan();
        self.typequadric = SurfaceType::Cone;
        self.z0 = TrigTerms {
            cte: q1,
            ..TrigTerms::default()
        };
        self.z1 = TrigTerms {
            cte: 2.0 * un_sur_tg * qz,
            sin: qy + qy,
            cos: qx + qx,
            ..TrigTerms::default()
        };
        self.z2 = TrigTerms {
            cte: qzz * un_sur_tg * un_sur_tg,
            sin: (un_sur_tg + un_sur_tg) * qyz,
            cos: (un_sur_tg + un_sur_tg) * qxz,
            cos_cos: qxx,
            sin_sin: qyy,
            cos_sin: qxy,
        };
        self.set_domain_values(tolerance, domain, two_curves, z_positive);
    }

    /// The domain part shared by both Set*QuadValues: a bounded domain, not open
    /// at either end, trimmed to `[DomInf, DomSup]` (to `2 DomSup - DomInf` for two
    /// curves).
    fn set_domain_values(
        &mut self,
        tolerance: f64,
        domain: (f64, f64),
        two_curves: bool,
        z_positive: bool,
    ) {
        self.two_curves = two_curves;
        self.take_z_positive = z_positive;
        self.tolerance = tolerance;
        (self.domain_inf, self.domain_sup) = domain;
        self.restricted_inf = true;
        self.restricted_sup = true;
        self.firstbounded = false;
        self.lastbounded = false;
        self.first_parameter = self.domain_inf;
        self.last_parameter = if two_curves {
            self.domain_sup + self.domain_sup - self.domain_inf
        } else {
            self.domain_sup
        };
    }

    /// IsOpen(): true when the domain is bounded at both ends (the native
    /// RestrictedInf && RestrictedSup). `curve.is_open()`.
    pub fn is_open(&self) -> bool {
        self.restricted_inf && self.restricted_sup
    }

    /// Domain(): the trimmed parametric domain; Standard_DomainError when the
    /// domain is not bounded (a curve never set). `let (first, last) = curve.domain()?;`
    pub fn domain(&self) -> Result<(f64, f64), Failure> {
        if !(self.restricted_inf && self.restricted_sup) {
            return Err(domain_error(
                "the domain is unbounded; expected a curve set by Set*QuadValues",
            ));
        }
        Ok((self.first_parameter, self.last_parameter))
    }

    /// IsConstant(): always false, as natively. `curve.is_constant()`.
    pub fn is_constant(&self) -> bool {
        false
    }

    /// IsFirstOpen(): the curve is not defined at the first parameter.
    /// `curve.is_first_open()`.
    pub fn is_first_open(&self) -> bool {
        self.firstbounded
    }

    /// IsLastOpen(): the curve is not defined at the last parameter.
    /// `curve.is_last_open()`.
    pub fn is_last_open(&self) -> bool {
        self.lastbounded
    }

    /// SetIsFirstOpen(Flag). `curve.set_is_first_open(true)`.
    pub fn set_is_first_open(&mut self, flag: bool) {
        self.firstbounded = flag;
    }

    /// SetIsLastOpen(Flag). `curve.set_is_last_open(true)`.
    pub fn set_is_last_open(&mut self, flag: bool) {
        self.lastbounded = flag;
    }

    /// SetDomain(First, Last): trims the curve; Standard_DomainError when
    /// `last <= first`. `curve.set_domain(0.0, 1.0)?`.
    pub fn set_domain(&mut self, first: f64, last: f64) -> Result<(), Failure> {
        if last <= first {
            return Err(domain_error(&format!(
                "the trim [{first}, {last}] is empty; expected First < Last"
            )));
        }
        self.first_parameter = first;
        self.last_parameter = last;
        Ok(())
    }
}

/// The Standard_DomainError("IntAna_Curve::Domain") raised by the curve.
fn domain_error(detail: &str) -> Failure {
    Failure::with_type(
        "Standard_DomainError",
        &format!("IntAna_Curve::Domain - {detail}"),
        "",
    )
}
