// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Gauss.hxx/.cxx (the
// class, its arithmetic and the elementary parts) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_Gauss`: the Gauss integrator of the surface and volume properties of a
//! face (docs/topalgo-design.md section 9). The non-adaptive overloads of `Compute` are
//! in `gauss_fixed.rs`, the adaptive ones in `gauss_adaptive.rs`.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Matrix3, Point3, Vector3, Xyz};

/// `BRepGProp_Gauss::GaussType`: the region the integrator computes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GaussType {
    /// `Vinert`: a region of space bounded by the surface and a point or a plane.
    Vinert = 0,
    /// `Sinert`: the face itself.
    Sinert,
}

/// The `add` and `mult` members (`std::function` natively): plain arithmetic, or the
/// arithmetic of infinite values that `checkBounds` selects for unbounded faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GaussArithmetic {
    /// `::Add` and `::Mult`.
    Finite,
    /// `::AddInf` and `::MultInf`.
    Infinite,
}

/// `BRepGProp_Gauss::Inertia`: the mass and the static and quadratic moments.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Inertia {
    pub(super) mass: f64,
    pub(super) ix: f64,
    pub(super) iy: f64,
    pub(super) iz: f64,
    pub(super) ixx: f64,
    pub(super) iyy: f64,
    pub(super) izz: f64,
    pub(super) ixy: f64,
    pub(super) ixz: f64,
    pub(super) iyz: f64,
}

/// The output references of `Compute` (`theOutMass`, `theOutGravityCenter`,
/// `theOutInertia`): a `Compute` that returns early leaves them unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GaussProps {
    /// The mass (area or volume).
    pub mass: f64,
    /// The gravity centre, relative to the location.
    pub gravity_center: Point3,
    /// The matrix of inertia at the location.
    pub inertia: Matrix3,
}

/// `BRepGProp_Gauss`: adaptive and non-adaptive 2D Gauss integration of the global
/// properties of a face (`Sinert`) or of the region between a face and a point or a
/// plane (`Vinert`).
///
/// ```
/// use rustycad_tktopalgo::brep_gprop::{BRepGPropGauss, GaussType};
/// let gauss = BRepGPropGauss::new(GaussType::Sinert);
/// assert_eq!(gauss.gauss_type(), GaussType::Sinert);
/// ```
#[derive(Debug)]
pub struct BRepGPropGauss {
    pub(super) kind: GaussType,
    pub(super) arithmetic: GaussArithmetic,
}

impl BRepGPropGauss {
    /// `BRepGProp_Gauss(theType)`: an integrator with plain arithmetic;
    /// `BRepGPropGauss::new(GaussType::Vinert)`.
    pub fn new(kind: GaussType) -> Self {
        Self {
            kind,
            arithmetic: GaussArithmetic::Finite,
        }
    }

    /// The type given to the constructor (`myType`); `gauss.gauss_type()`.
    pub fn gauss_type(&self) -> GaussType {
        self.kind
    }

    /// `add(a, b)`.
    pub(super) fn add(&self, a: f64, b: f64) -> f64 {
        match self.arithmetic {
            GaussArithmetic::Finite => a + b,
            GaussArithmetic::Infinite => add_inf(a, b),
        }
    }

    /// `mult(a, b)`.
    pub(super) fn mult(&self, a: f64, b: f64) -> f64 {
        match self.arithmetic {
            GaussArithmetic::Finite => a * b,
            GaussArithmetic::Infinite => mult_inf(a, b),
        }
    }

    /// `checkBounds`: the infinite arithmetic once a bound is infinite (never reset).
    pub(super) fn check_bounds(&mut self, bounds: (f64, f64, f64, f64)) {
        let (u1, u2, v1, v2) = bounds;
        if [u1, u2, v1, v2].into_iter().any(precision::is_infinite) {
            self.arithmetic = GaussArithmetic::Infinite;
        }
    }

    /// `Standard_ASSERT_RAISE(myType == theType, ...)`: Standard_ProgramError for a
    /// `Compute` overload of the other type.
    pub(super) fn assert_type(&self, expected: GaussType) -> Result<(), Failure> {
        if self.kind == expected {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_ProgramError",
            &format!(
                "BRepGProp_Gauss: Incorrect type {:?}; expected {expected:?} for this Compute",
                self.kind
            ),
            "",
        ))
    }

    /// `addAndRestoreInertia(theInInertia, theOutInertia)`.
    pub(super) fn add_inertia(&self, input: &Inertia, output: &mut Inertia) {
        output.mass = self.add(output.mass, input.mass);
        output.ix = self.add(output.ix, input.ix);
        output.iy = self.add(output.iy, input.iy);
        output.iz = self.add(output.iz, input.iz);
        output.ixx = self.add(output.ixx, input.ixx);
        output.iyy = self.add(output.iyy, input.iyy);
        output.izz = self.add(output.izz, input.izz);
        output.ixy = self.add(output.ixy, input.ixy);
        output.ixz = self.add(output.ixz, input.ixz);
        output.iyz = self.add(output.iyz, input.iyz);
    }

    /// `multAndRestoreInertia(theValue, theInertia)`.
    pub(super) fn mult_inertia(&self, value: f64, inertia: &mut Inertia) {
        for field in inertia.fields_mut() {
            *field = self.mult(*field, value);
        }
    }

    /// `computeSInertiaOfElementaryPart`: one Gauss point of the surface integral, with
    /// the member arithmetic.
    pub(super) fn add_surface_part(
        &self,
        point: Point3,
        normal: Vector3,
        location: Point3,
        weight: f64,
        out: &mut Inertia,
    ) {
        let ds = self.mult(normal.magnitude(), weight);
        let x = self.add(point.x(), -location.x());
        let y = self.add(point.y(), -location.y());
        let z = self.add(point.z(), -location.z());
        out.mass = self.add(out.mass, ds);
        let (xds, yds, zds) = (self.mult(x, ds), self.mult(y, ds), self.mult(z, ds));
        out.ix = self.add(out.ix, xds);
        out.iy = self.add(out.iy, yds);
        out.iz = self.add(out.iz, zds);
        out.ixy = self.add(out.ixy, self.mult(x, yds));
        out.iyz = self.add(out.iyz, self.mult(y, zds));
        out.ixz = self.add(out.ixz, self.mult(x, zds));
        let (xxds, yyds, zzds) = (self.mult(x, xds), self.mult(y, yds), self.mult(z, zds));
        out.ixx = self.add(out.ixx, self.add(yyds, zzds));
        out.iyy = self.add(out.iyy, self.add(xxds, zzds));
        out.izz = self.add(out.izz, self.add(xxds, yyds));
    }

    /// `convert(theInertia, G, I, Mass)`: the surface result, the centre `I / Mass` and the
    /// matrix with negated products; mass and centre 0 below 1e-30.
    pub(super) fn convert_surface(&self, inertia: &Inertia, out: &mut GaussProps) {
        if inertia.mass.abs() >= EPS_DIM {
            let inv_mass = 1.0 / inertia.mass;
            out.gravity_center = Point3::new(
                inertia.ix * inv_mass,
                inertia.iy * inv_mass,
                inertia.iz * inv_mass,
            );
            out.mass = inertia.mass;
        } else {
            out.mass = 0.0;
            out.gravity_center = Point3::new(0.0, 0.0, 0.0);
        }
        out.inertia = Matrix3::from_columns(
            Xyz::new(inertia.ixx, -inertia.ixy, -inertia.ixz),
            Xyz::new(-inertia.ixy, inertia.iyy, -inertia.iyz),
            Xyz::new(-inertia.ixz, -inertia.iyz, inertia.izz),
        );
    }

    /// `convert(theInertia, theCoeff, theIsByPoint, G, I, Mass)`: the volume result; by
    /// point the centre is shifted by the coefficients; by plane the mass and centre are
    /// 0 (as natively: the by-plane branch of the native test is dead); the matrix keeps
    /// the signs of the sums.
    pub(super) fn convert_volume(
        &self,
        inertia: &Inertia,
        coeff: &[f64],
        by_point: bool,
        out: &mut GaussProps,
    ) {
        self.convert_surface(inertia, out);
        if inertia.mass.abs() >= EPS_DIM && by_point {
            let inv_mass = 1.0 / inertia.mass;
            out.gravity_center = Point3::new(
                coeff[0] + inertia.ix * inv_mass,
                coeff[1] + inertia.iy * inv_mass,
                coeff[2] + inertia.iz * inv_mass,
            );
            out.mass = inertia.mass;
        } else {
            out.mass = 0.0;
            out.gravity_center = Point3::new(0.0, 0.0, 0.0);
        }
        out.inertia = Matrix3::from_columns(
            Xyz::new(inertia.ixx, inertia.ixy, inertia.ixz),
            Xyz::new(inertia.ixy, inertia.iyy, inertia.iyz),
            Xyz::new(inertia.ixz, inertia.iyz, inertia.izz),
        );
    }
}

/// The minimal `|mass|` of a result (`EPS_DIM`).
pub(super) const EPS_DIM: f64 = 1.0e-30;

impl Inertia {
    fn fields_mut(&mut self) -> [&mut f64; 10] {
        [
            &mut self.mass,
            &mut self.ix,
            &mut self.iy,
            &mut self.iz,
            &mut self.ixx,
            &mut self.iyy,
            &mut self.izz,
            &mut self.ixy,
            &mut self.ixz,
            &mut self.iyz,
        ]
    }
}

/// `computeVInertiaOfElementaryPart`: one Gauss point of the volume integral, with
/// plain arithmetic: the cone to the location (`by_point`, coefficients the shift of
/// the inertia point) or the prism to the plane `coeff` (a, b, c, d).
pub(super) fn add_volume_part(
    point: Point3,
    normal: Vector3,
    location: Point3,
    weight: f64,
    plane: (&[f64], bool),
    out: &mut Inertia,
) {
    let x = point.x() - location.x();
    let y = point.y() - location.y();
    let z = point.z() - location.z();
    let n = [
        normal.x() * weight,
        normal.y() * weight,
        normal.z() * weight,
    ];
    let (coeff, by_point) = plane;
    if by_point {
        add_cone_part([x, y, z], n, coeff, out);
    } else {
        add_prism_part([x, y, z], n, coeff, out);
    }
}

/// The by-point branch of `computeVInertiaOfElementaryPart` (OFV code, modified by APO).
fn add_cone_part(p: [f64; 3], n: [f64; 3], coeff: &[f64], out: &mut Inertia) {
    let [mut x, mut y, mut z] = p;
    let mut dv = x * n[0] + y * n[1] + z * n[2];
    out.mass += dv / 3.0;
    out.ix += 0.25 * x * dv;
    out.iy += 0.25 * y * dv;
    out.iz += 0.25 * z * dv;
    x -= coeff[0];
    y -= coeff[1];
    z -= coeff[2];
    dv *= 0.2;
    out.ixy -= x * y * dv;
    out.iyz -= y * z * dv;
    out.ixz -= x * z * dv;
    let (x2, y2, z2) = (x * x, y * y, z * z);
    out.ixx += (y2 + z2) * dv;
    out.iyy += (x2 + z2) * dv;
    out.izz += (x2 + y2) * dv;
}

/// The by-plane branch of `computeVInertiaOfElementaryPart`.
fn add_prism_part(p: [f64; 3], n: [f64; 3], c: &[f64], out: &mut Inertia) {
    let [x, y, z] = p;
    let s = n[0] * c[0] + n[1] * c[1] + n[2] * c[2];
    let d1 = c[0] * x + c[1] * y + c[2] * z - c[3];
    let mut d2 = d1 * d1;
    let d3 = d1 * d2 / 3.0;
    let dv = s * d1;
    out.mass += dv;
    out.ix += (x - (c[0] * d1 * 0.5)) * dv;
    out.iy += (y - (c[1] * d1 * 0.5)) * dv;
    out.iz += (z - (c[2] * d1 * 0.5)) * dv;
    let (px, py, pz) = (x - c[0] * d1, y - c[1] * d1, z - c[2] * d1);
    let qx = px * px * d1 + px * c[0] * d2 + c[0] * c[0] * d3;
    let qy = py * py * d1 + py * c[1] * d2 + c[1] * c[1] * d3;
    let qz = pz * pz * d1 + pz * c[2] * d2 + c[2] * c[2] * d3;
    out.ixx += (qy + qz) * s;
    out.iyy += (qx + qz) * s;
    out.izz += (qx + qy) * s;
    d2 *= 0.5;
    let rx = (py * pz * d1) + (py * c[2] * d2) + (pz * c[1] * d2) + (c[1] * c[2] * d3);
    let ry = (px * pz * d1) + (pz * c[0] * d2) + (px * c[2] * d2) + (c[0] * c[2] * d3);
    let rz = (px * py * d1) + (px * c[1] * d2) + (py * c[0] * d2) + (c[0] * c[1] * d3);
    out.ixy -= rz * s;
    out.iyz -= rx * s;
    out.ixz -= ry * s;
}

/// `AddInf`: the sum of values that may be infinite (`Precision::Infinite()`), 0 for
/// opposite infinities.
fn add_inf(a: f64, b: f64) -> f64 {
    use precision::{infinite, is_negative_infinite as neg, is_positive_infinite as pos};
    if pos(a) {
        return if neg(b) { 0.0 } else { infinite() };
    }
    if pos(b) {
        return if neg(a) { 0.0 } else { infinite() };
    }
    if neg(a) {
        return if pos(b) { 0.0 } else { -infinite() };
    }
    if neg(b) {
        return if pos(a) { 0.0 } else { -infinite() };
    }
    a + b
}

/// `MultInf`: the product of values that may be infinite, 0 when a factor is exactly 0.
fn mult_inf(a: f64, b: f64) -> f64 {
    use precision::{infinite, is_negative_infinite as neg, is_positive_infinite as pos};
    if a == 0.0 || b == 0.0 {
        return 0.0;
    }
    let signed = |negative: bool| if negative { -infinite() } else { infinite() };
    let is_negative = |x: f64| x < 0.0;
    if pos(a) {
        return signed(is_negative(b));
    }
    if pos(b) {
        return signed(is_negative(a));
    }
    // A negative infinity keeps its sign against a factor that is not negative (NaN
    // included, as the native `theB < 0.0` test is false for it).
    if neg(a) {
        return signed(!is_negative(b));
    }
    if neg(b) {
        return signed(!is_negative(a));
    }
    a * b
}
