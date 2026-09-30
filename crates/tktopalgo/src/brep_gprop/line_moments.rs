// Rust translation of the Gauss sums shared by
// ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Cinert.cxx and BRepGProp_MeshCinert.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The length, static and quadratic moments of a line integral, summed as both linear
//! integrators of BRepGProp sum them.
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{self, Matrix3, Point3, Xyz};

/// The ten sums of a line integral: the length `dim`, the static moments and the
/// quadratic moments (the `dimLocal`, `IxLocal`... locals of the native loops).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct LineMoments {
    dim: f64,
    ix: f64,
    iy: f64,
    iz: f64,
    ixx: f64,
    iyy: f64,
    izz: f64,
    ixy: f64,
    ixz: f64,
    iyz: f64,
}

impl LineMoments {
    /// One Gauss point at `(x, y, z)` (relative to the location) of weight `ds`, in the
    /// native order: the products before the squares.
    pub(super) fn add_point(&mut self, point: Xyz, ds: f64) {
        let (x, y, z) = (point.x(), point.y(), point.z());
        self.dim += ds;
        self.ix += x * ds;
        self.iy += y * ds;
        self.iz += z * ds;
        self.ixy += x * y * ds;
        self.iyz += y * z * ds;
        self.ixz += x * z * ds;
        let (x2, y2, z2) = (x * x, y * y, z * z);
        self.ixx += (y2 + z2) * ds;
        self.iyy += (x2 + z2) * ds;
        self.izz += (x2 + y2) * ds;
    }

    /// Every sum times the half range `ur` of an interval.
    pub(super) fn scale(&mut self, ur: f64) {
        for sum in self.sums_mut() {
            *sum *= ur;
        }
    }

    /// The sums of the next interval added to the totals.
    pub(super) fn accumulate(&mut self, local: &LineMoments) {
        let sums = [
            local.dim, local.ix, local.iy, local.iz, local.ixx, local.iyy, local.izz, local.ixy,
            local.ixz, local.iyz,
        ];
        for (total, sum) in self.sums_mut().into_iter().zip(sums) {
            *total += sum;
        }
    }

    fn sums_mut(&mut self) -> [&mut f64; 10] {
        [
            &mut self.dim,
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

    /// The end of both Perform members: the mass, the inertia matrix (products negated)
    /// and the centre, `last_point` when the length is below gp::Resolution().
    pub(super) fn store(&self, base: &mut GProps, last_point: Point3) {
        base.dim = self.dim;
        base.inertia = Matrix3::from_columns(
            Xyz::new(self.ixx, -self.ixy, -self.ixz),
            Xyz::new(-self.ixy, self.iyy, -self.iyz),
            Xyz::new(-self.ixz, -self.iyz, self.izz),
        );
        if self.dim.abs() < gp::resolution() {
            base.g = last_point;
        } else {
            base.g = Point3::new(self.ix / self.dim, self.iy / self.dim, self.iz / self.dim);
        }
    }
}
