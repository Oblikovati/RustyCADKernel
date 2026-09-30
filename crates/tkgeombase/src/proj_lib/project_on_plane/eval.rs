// Rust translation of the file-local helpers of ProjLib_ProjectOnPlane.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The oblique projection formulas of ProjLib_ProjectOnPlane:
//! `Proj(u) = P(u) + ((PO . Z) / (D . Z)) D` with `O` and `Z` the location and the
//! normal of the plane, and its derivatives, evaluated on the adapted curve.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Vector3, Xyz};

/// The plane and the projection direction shared by every formula.
#[derive(Clone, Copy, Debug)]
pub(super) struct Oblique {
    pub(super) plane: Axis3,
    pub(super) direction: Direction3,
}

impl Oblique {
    // `Alpha = (PO . Z) / (D . Z)` of a point, in the native operation order.
    fn point_alpha(&self, point: Point3) -> f64 {
        let po: Vector3 = Vector3::between(point, self.plane.location());
        let mut alpha: f64 = po.dot(Vector3::from(self.plane.direction()));
        alpha /= self.direction.dot(self.plane.direction());
        alpha
    }
    // `Alpha = (V . Z) / (D . Z)` of a derivative vector.
    fn vector_alpha(&self, vector: Vector3) -> f64 {
        let z: Direction3 = self.plane.direction();
        let mut alpha: f64 = vector.dot(Vector3::from(z));
        alpha /= self.direction.dot(z);
        alpha
    }

    /// `ProjectPnt(ThePlane, TheDir, Point)` and the point part of every `OnPlane_*`.
    pub(super) fn project_point(&self, point: Point3) -> Point3 {
        let alpha: f64 = self.point_alpha(point);
        let mut projected: Point3 = point;
        projected.set_xyz(point.xyz() + scaled(self.direction.xyz(), alpha));
        projected
    }

    /// The derivative part of `OnPlane_D1`..`OnPlane_DN`: `V - ((V . Z) / (D . Z)) D`.
    pub(super) fn project_derivative(&self, vector: Vector3) -> Vector3 {
        let alpha: f64 = self.vector_alpha(vector);
        Vector3::from(*vector.xyz() - scaled(self.direction.xyz(), alpha))
    }

    /// `ProjectVec(ThePlane, TheDir, Vec)`: `D -= ((Vec . Z) / (TheDir . Z)) TheDir`,
    /// with `Z` taken as a gp_Dir for the second dot product (the native implicit
    /// conversion); `oblique.project_vector(v)?`.
    pub(super) fn project_vector(&self, vector: Vector3) -> Result<Vector3, Failure> {
        let z: Vector3 = Vector3::from(self.plane.direction());
        let ratio: f64 = vector.dot(z) / self.direction.dot(Direction3::from_vector(z)?);
        Ok(vector - Vector3::from(self.direction) * ratio)
    }

    /// `OnPlane_Value(U, C, Pl, D)`; `oblique.value(curve, 0.5)?`.
    pub(super) fn value(&self, curve: &dyn Adaptor3dCurve, u: f64) -> Result<Point3, Failure> {
        Ok(self.project_point(curve.value(u)?))
    }

    /// `OnPlane_D1(U, P, V, C, Pl, D)`; `oblique.d1(curve, 0.5)?`.
    pub(super) fn d1(
        &self,
        curve: &dyn Adaptor3dCurve,
        u: f64,
    ) -> Result<(Point3, Vector3), Failure> {
        let (point, vector) = curve.d1(u)?;
        Ok((self.project_point(point), self.project_derivative(vector)))
    }

    /// `OnPlane_D2(U, P, V1, V2, C, Pl, D)`; `oblique.d2(curve, 0.5)?`.
    pub(super) fn d2(
        &self,
        curve: &dyn Adaptor3dCurve,
        u: f64,
    ) -> Result<(Point3, Vector3, Vector3), Failure> {
        let (point, v1, v2) = curve.d2(u)?;
        let point: Point3 = self.project_point(point);
        Ok((
            point,
            self.project_derivative(v1),
            self.project_derivative(v2),
        ))
    }

    /// `OnPlane_D3(U, P, V1, V2, V3, C, Pl, D)`; `oblique.d3(curve, 0.5)?`.
    pub(super) fn d3(
        &self,
        curve: &dyn Adaptor3dCurve,
        u: f64,
    ) -> Result<(Point3, Vector3, Vector3, Vector3), Failure> {
        let (point, v1, v2, v3) = curve.d3(u)?;
        let point: Point3 = self.project_point(point);
        let (v1, v2) = (self.project_derivative(v1), self.project_derivative(v2));
        Ok((point, v1, v2, self.project_derivative(v3)))
    }

    /// `OnPlane_DN(U, N, C, Pl, D)`; `oblique.dn(curve, 0.5, 2)?`.
    pub(super) fn dn(
        &self,
        curve: &dyn Adaptor3dCurve,
        u: f64,
        n: i32,
    ) -> Result<Vector3, Failure> {
        Ok(self.project_derivative(curve.dn(u, n)?))
    }
}

// `Alpha * D.XYZ()`: gp_XYZ scaled by a real.
fn scaled(xyz: Xyz, alpha: f64) -> Xyz {
    xyz * alpha
}
