// Rust translation of gce_MakeDir.hxx/.cxx and gce_MakeDir2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakeDir and gce_MakeDir2d: construction of gp_Dir and gp_Dir2d.
use super::{ErrorType, gce_result};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    self, Direction2, Direction3, Point2, Point3, Vector2, Vector3, Xy, Xyz,
};

/// gce_MakeDir: builds a direction from a vector, coordinates or two points;
/// gce_NullVector or gce_ConfusedPoints when the length is at most
/// gp::Resolution().
///
/// `let d = MakeDir::from_coordinates(1.0, 0.0, 0.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeDir {
    the_result: Direction3,
    the_error: ErrorType,
}
gce_result!(MakeDir, Direction3, "gce_MakeDir");

impl MakeDir {
    /// The maker for a checked length: `length <= gp::Resolution()` gives
    /// `failure`, otherwise `build` normalizes (and raises as gp_Dir does).
    fn checked(
        length: f64,
        failure: ErrorType,
        build: impl FnOnce() -> Result<Direction3, Failure>,
    ) -> Result<Self, Failure> {
        if length <= gp::resolution() {
            return Ok(Self {
                the_error: failure,
                ..Self::default()
            });
        }
        Ok(Self {
            the_result: build()?,
            the_error: ErrorType::Done,
        })
    }

    /// gce_MakeDir(V): the direction of `vector`. `MakeDir::from_vector(v)?`.
    pub fn from_vector(vector: Vector3) -> Result<Self, Failure> {
        Self::checked(vector.magnitude(), ErrorType::NullVector, || {
            Direction3::from_vector(vector)
        })
    }

    /// gce_MakeDir(Coord): the direction of `coord`. `MakeDir::from_xyz(xyz)?`.
    pub fn from_xyz(coord: Xyz) -> Result<Self, Failure> {
        Self::checked(coord.modulus(), ErrorType::NullVector, || {
            Direction3::from_xyz(coord)
        })
    }

    /// gce_MakeDir(Xv, Yv, Zv): the direction of the coordinates; the null test
    /// compares the square length with gp::Resolution().
    /// `MakeDir::from_coordinates(1.0, 0.0, 0.0)?`.
    pub fn from_coordinates(xv: f64, yv: f64, zv: f64) -> Result<Self, Failure> {
        let square: f64 = xv * xv + yv * yv + zv * zv;
        Self::checked(square, ErrorType::NullVector, || {
            Direction3::new(xv, yv, zv)
        })
    }

    /// gce_MakeDir(P1, P2): the direction from `p1` to `p2`.
    /// `MakeDir::from_points(a, b)?`.
    pub fn from_points(p1: Point3, p2: Point3) -> Result<Self, Failure> {
        let length: f64 = p1.distance(&p2);
        Self::checked(length, ErrorType::ConfusedPoints, || {
            Direction3::from_xyz(p2.xyz() - p1.xyz())
        })
    }
}

/// gce_MakeDir2d: builds a 2D direction from a vector, coordinates or two
/// points; gce_NullVector or gce_ConfusedPoints when the length is at most
/// gp::Resolution().
///
/// `let d = MakeDir2d::from_coordinates(1.0, 0.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeDir2d {
    the_result: Direction2,
    the_error: ErrorType,
}
gce_result!(MakeDir2d, Direction2, "gce_MakeDir2d");

impl MakeDir2d {
    /// The maker for a checked length, as [`MakeDir`]'s.
    fn checked(
        length: f64,
        failure: ErrorType,
        build: impl FnOnce() -> Result<Direction2, Failure>,
    ) -> Result<Self, Failure> {
        if length <= gp::resolution() {
            return Ok(Self {
                the_error: failure,
                ..Self::default()
            });
        }
        Ok(Self {
            the_result: build()?,
            the_error: ErrorType::Done,
        })
    }

    /// gce_MakeDir2d(V): the direction of `vector`. `MakeDir2d::from_vector(v)?`.
    pub fn from_vector(vector: Vector2) -> Result<Self, Failure> {
        Self::checked(vector.magnitude(), ErrorType::NullVector, || {
            Direction2::from_vector(vector)
        })
    }

    /// gce_MakeDir2d(Coord): the direction of `coord`. `MakeDir2d::from_xy(xy)?`.
    pub fn from_xy(coord: Xy) -> Result<Self, Failure> {
        Self::checked(coord.modulus(), ErrorType::NullVector, || {
            Direction2::from_xy(coord)
        })
    }

    /// gce_MakeDir2d(Xv, Yv): the direction of the coordinates; the null test
    /// compares the square length with gp::Resolution().
    /// `MakeDir2d::from_coordinates(1.0, 0.0)?`.
    pub fn from_coordinates(xv: f64, yv: f64) -> Result<Self, Failure> {
        Self::checked(xv * xv + yv * yv, ErrorType::NullVector, || {
            Direction2::new(xv, yv)
        })
    }

    /// gce_MakeDir2d(P1, P2): the direction from `p1` to `p2`.
    /// `MakeDir2d::from_points(a, b)?`.
    pub fn from_points(p1: Point2, p2: Point2) -> Result<Self, Failure> {
        let length: f64 = p1.distance(p2);
        Self::checked(length, ErrorType::ConfusedPoints, || {
            Direction2::from_xy(p2.xy() - p1.xy())
        })
    }
}
