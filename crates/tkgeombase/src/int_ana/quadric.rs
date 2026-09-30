// Rust translation of IntAna_Quadric.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_Quadric: a quadric known by the coefficients of its equation.
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis3, Cone3, Cylinder3, Plane3, Point3, Sphere3, Transform3};
use std::f64::consts::FRAC_PI_2;

/// IntAna_Quadric: the quadric
/// `CXX x² + CYY y² + CZZ z² + 2 (CXY xy + CXZ xz + CYZ yz) + 2 (CX x + CY y + CZ z) + CCte`
/// in the natural frame, and its special points (cone apex, sphere poles).
///
/// The coefficient arrays are `[CXX, CYY, CZZ, CXY, CXZ, CYZ, CX, CY, CZ, CCte]`.
/// `let quadric = Quadric::from_sphere(sphere); let c = quadric.coefficients();`
#[derive(Clone)]
pub struct Quadric {
    coefficients: [f64; 10],
    special_points: List<Point3>,
}

impl Default for Quadric {
    /// IntAna_Quadric(): every coefficient null but CCte = 1 (an empty quadric).
    fn default() -> Self {
        let mut coefficients: [f64; 10] = [0.0; 10];
        coefficients[9] = 1.0;
        Self {
            coefficients,
            special_points: List::default(),
        }
    }
}

impl Quadric {
    /// IntAna_Quadric(): the empty quadric. `Quadric::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// IntAna_Quadric(gp_Pln). `Quadric::from_plane(plane)?`.
    pub fn from_plane(plane: Plane3) -> Result<Self, Failure> {
        let mut quadric: Self = Self::default();
        quadric.set_plane(plane)?;
        Ok(quadric)
    }

    /// IntAna_Quadric(gp_Sphere). `Quadric::from_sphere(sphere)`.
    pub fn from_sphere(sphere: Sphere3) -> Self {
        let mut quadric: Self = Self::default();
        quadric.set_sphere(sphere);
        quadric
    }

    /// IntAna_Quadric(gp_Cylinder). `Quadric::from_cylinder(cylinder)`.
    pub fn from_cylinder(cylinder: Cylinder3) -> Self {
        let mut quadric: Self = Self::default();
        quadric.set_cylinder(cylinder);
        quadric
    }

    /// IntAna_Quadric(gp_Cone). `Quadric::from_cone(cone)`.
    pub fn from_cone(cone: Cone3) -> Self {
        let mut quadric: Self = Self::default();
        quadric.set_cone(cone);
        quadric
    }

    /// SetQuadric(gp_Pln): the plane `A x + B y + C z + D` written as
    /// `2 (A/2 x + B/2 y + C/2 z) + D`. As natively, the special points are kept.
    /// `quadric.set_plane(plane)?`.
    pub fn set_plane(&mut self, plane: Plane3) -> Result<(), Failure> {
        let [a, b, c, d] = plane.coefficients()?;
        self.coefficients = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, a * 0.5, b * 0.5, c * 0.5, d];
        Ok(())
    }

    /// SetQuadric(gp_Sphere): the sphere coefficients; appends the two poles to the
    /// special points (without clearing earlier ones, as natively).
    /// `quadric.set_sphere(sphere)`.
    pub fn set_sphere(&mut self, sphere: Sphere3) {
        self.coefficients = sphere.coefficients();
        self.special_points
            .append(elslib::value(0.0, -FRAC_PI_2, &sphere));
        self.special_points
            .append(elslib::value(0.0, FRAC_PI_2, &sphere));
    }

    /// SetQuadric(gp_Cone): the cone coefficients; appends the apex
    /// (`V = -RefRadius / sin(SemiAngle)`) to the special points.
    /// `quadric.set_cone(cone)`.
    pub fn set_cone(&mut self, cone: Cone3) {
        self.coefficients = cone.coefficients();
        let apex_parameter: f64 = -cone.ref_radius() / cone.semi_angle().sin();
        self.special_points
            .append(elslib::value(0.0, apex_parameter, &cone));
    }

    /// SetQuadric(gp_Cylinder): the cylinder coefficients; the special points are
    /// kept. `quadric.set_cylinder(cylinder)`.
    pub fn set_cylinder(&mut self, cylinder: Cylinder3) {
        self.coefficients = cylinder.coefficients();
    }

    /// Coefficients(): `[CXX, CYY, CZZ, CXY, CXZ, CYZ, CX, CY, CZ, CCte]`.
    /// `quadric.coefficients()[9]`.
    pub fn coefficients(&self) -> [f64; 10] {
        self.coefficients
    }

    /// NewCoefficients(..., Axis): the equation `coefficients` (natural frame)
    /// rewritten in the local frame of `axis`. As natively, the coefficients of
    /// `self` are not read: the input array is transformed.
    /// `quadric.new_coefficients(quadric.coefficients(), frame)?`.
    pub fn new_coefficients(
        &self,
        coefficients: [f64; 10],
        axis: Axis3,
    ) -> Result<[f64; 10], Failure> {
        let inverse: Transform3 = Transform3::transformation_to(axis).inverted()?;
        let mut rows: [[f64; 4]; 3] = [[0.0; 4]; 3];
        for (row, values) in rows.iter_mut().enumerate() {
            for (column, value) in values.iter_mut().enumerate() {
                *value = inverse.value(row as i32 + 1, column as i32 + 1)?;
            }
        }
        let [cxx, cyy, czz, cxy, cxz, cyz] = second_order(&rows, &coefficients);
        let [cx, cy, cz] = first_order(&rows, &coefficients);
        Ok([
            cxx,
            cyy,
            czz,
            cxy,
            cxz,
            cyz,
            cx,
            cy,
            cz,
            constant(&rows, &coefficients),
        ])
    }

    /// SpecialPoints(): the points with singularities (cone apex, sphere poles).
    /// `quadric.special_points().len()`.
    pub fn special_points(&self) -> &List<Point3> {
        &self.special_points
    }
}

/// The frame change `x = t11 X + t12 Y + t13 Z + t14` (rows of the inverse
/// transformation, `t[i][j]` = `T(i+1, j+1)`).
type Rows = [[f64; 4]; 3];

/// The new constant term Ccte.
fn constant(t: &Rows, q: &[f64; 10]) -> f64 {
    let [cxx, cyy, czz, cxy, cxz, cyz, cx, cy, cz, ccte] = *q;
    let (t14, t24, t34) = (t[0][3], t[1][3], t[2][3]);
    ccte + t14 * t14 * cxx
        + t24 * t24 * cyy
        + t34 * t34 * czz
        + 2.0 * (t14 * (cx + t24 * cxy + t34 * cxz) + t24 * (cy + t34 * cyz) + t34 * cz)
}

/// The new second-order terms `[Cxx, Cyy, Czz, Cxy, Cxz, Cyz]`.
fn second_order(t: &Rows, q: &[f64; 10]) -> [f64; 6] {
    let [cxx, cyy, czz, cxy, cxz, cyz, ..] = *q;
    let [[t11, t12, t13, _], [t21, t22, t23, _], [t31, t32, t33, _]] = *t;
    let (t11_p2, t21_p2, t31_p2) = (t11 * t11, t21 * t21, t31 * t31);
    let (t12_p2, t22_p2, t32_p2) = (t12 * t12, t22 * t22, t32 * t32);
    let (t13_p2, t23_p2, t33_p2) = (t13 * t13, t23 * t23, t33 * t33);
    [
        t11_p2 * cxx
            + t21_p2 * cyy
            + t31_p2 * czz
            + 2.0 * (t11 * (t21 * cxy + t31 * cxz) + t21 * t31 * cyz),
        t12_p2 * cxx
            + t22_p2 * cyy
            + t32_p2 * czz
            + 2.0 * (t12 * (t22 * cxy + t32 * cxz) + t22 * t32 * cyz),
        t13_p2 * cxx
            + t33_p2 * czz
            + t23_p2 * cyy
            + 2.0 * (t13 * (t23 * cxy + t33 * cxz) + t23 * t33 * cyz),
        t11 * (t12 * cxx + t22 * cxy + t32 * cxz)
            + t12 * (t21 * cxy + t31 * cxz)
            + t21 * (t22 * cyy + t32 * cyz)
            + t31 * (t22 * cyz + t32 * czz),
        t11 * (t13 * cxx + t23 * cxy + t33 * cxz)
            + t13 * (t21 * cxy + t31 * cxz)
            + t21 * (t23 * cyy + t33 * cyz)
            + t31 * (t23 * cyz + t33 * czz),
        t12 * (t13 * cxx + t23 * cxy + t33 * cxz)
            + t13 * (t22 * cxy + t32 * cxz)
            + t22 * (t23 * cyy + t33 * cyz)
            + t32 * (t23 * cyz + t33 * czz),
    ]
}

/// The new first-order terms `[Cx, Cy, Cz]`.
fn first_order(t: &Rows, q: &[f64; 10]) -> [f64; 3] {
    let [cxx, cyy, czz, cxy, cxz, cyz, cx, cy, cz, _] = *q;
    let [
        [t11, t12, t13, t14],
        [t21, t22, t23, t24],
        [t31, t32, t33, t34],
    ] = *t;
    [
        t11 * (cx + t14 * cxx + t24 * cxy + t34 * cxz)
            + t14 * (t21 * cxy + t31 * cxz)
            + t21 * (cy + t24 * cyy + t34 * cyz)
            + t31 * (t24 * cyz + cz + t34 * czz),
        t12 * (cx + t14 * cxx + t24 * cxy + t34 * cxz)
            + t14 * (t22 * cxy + t32 * cxz)
            + t22 * (cy + t24 * cyy + t34 * cyz)
            + t32 * (cz + t24 * cyz + t34 * czz),
        t13 * cx
            + t13 * (t14 * cxx + t24 * cxy + t34 * cxz)
            + t14 * (t23 * cxy + t33 * cxz)
            + t23 * (cy + t24 * cyy + t34 * cyz)
            + t33 * (t24 * cyz + cz + t34 * czz),
    ]
}
