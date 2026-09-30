// Rust translation of gce_MakeHypr.hxx/.cxx and gce_MakeHypr2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakeHypr and gce_MakeHypr2d: construction of gp_Hypr and gp_Hypr2d.
use super::{ErrorType, gce_result};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    self, Axis2, Axis2d, Axis22d, Direction2, Direction3, Hyperbola2, Hyperbola3, Line2, Line3,
    Point2, Point3, Xy, Xyz,
};

/// gce_MakeHypr: builds a hyperbola from a frame and radii, or from the ends of
/// its axes and its center.
///
/// `let hyperbola = MakeHypr::new(Axis2::default(), 2.0, 1.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeHypr {
    the_result: Hyperbola3,
    the_error: ErrorType,
}
gce_result!(MakeHypr, Hyperbola3, "gce_MakeHypr");

impl MakeHypr {
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }

    /// gce_MakeHypr(A2, MajorRadius, MinorRadius): gce_NegativeRadius when a
    /// radius is negative; the major radius may be smaller than the minor one.
    /// `MakeHypr::new(frame, 2.0, 1.0)?`.
    pub fn new(frame: Axis2, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if major_radius < 0.0 || minor_radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let hyperbola = Hyperbola3::new(frame, major_radius, minor_radius)?;
        Ok(Self {
            the_result: hyperbola,
            the_error: ErrorType::Done,
        })
    }

    /// gce_MakeHypr(S1, S2, Center): the hyperbola of center `center` whose major
    /// axis ends at `s1` and whose minor radius is the distance of `s2` from the
    /// major axis; gce_ConfusedPoints when two points coincide, gce_ColinearPoints
    /// when `s2` lies on the major axis. `MakeHypr::from_points(s1, s2, center)?`.
    pub fn from_points(s1: Point3, s2: Point3, center: Point3) -> Result<Self, Failure> {
        let vec_s1: Xyz = s1.xyz() - center.xyz();
        let vec_s2: Xyz = s2.xyz() - center.xyz();
        let dist_s1: f64 = vec_s1.modulus();
        if confused(dist_s1, vec_s2.modulus(), s1.distance(&s2)) {
            return Ok(Self::failed(ErrorType::ConfusedPoints));
        }
        let x_axis: Direction3 = Direction3::from_xyz(vec_s1)?;
        let minor_dist: f64 = Line3::new(center, x_axis).distance(s2);
        if minor_dist <= gp::resolution() {
            return Ok(Self::failed(ErrorType::ColinearPoints));
        }
        let cross: Xyz = x_axis.xyz().crossed(vec_s2);
        if cross.modulus() <= gp::resolution() {
            return Ok(Self::failed(ErrorType::ColinearPoints));
        }
        let frame = Axis2::with_x_direction(center, Direction3::from_xyz(cross)?, x_axis)?;
        let hyperbola = Hyperbola3::new(frame, dist_s1, minor_dist)?;
        Ok(Self {
            the_result: hyperbola,
            the_error: ErrorType::Done,
        })
    }
}

/// gce_MakeHypr2d: builds a 2D hyperbola from a major axis, a frame, or the ends
/// of its axes and its center.
///
/// `let hyperbola = MakeHypr2d::from_frame(Axis22d::default(), 2.0, 1.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeHypr2d {
    the_result: Hyperbola2,
    the_error: ErrorType,
}
gce_result!(MakeHypr2d, Hyperbola2, "gce_MakeHypr2d");

impl MakeHypr2d {
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }
    fn built(hyperbola: Hyperbola2) -> Self {
        Self {
            the_result: hyperbola,
            the_error: ErrorType::Done,
        }
    }

    /// gce_MakeHypr2d(S1, S2, Center): the hyperbola of center `center`, major axis
    /// toward `s1`, Y direction toward `s2` and minor radius the distance of `s2`
    /// from the major axis; gce_ConfusedPoints, then gce_ColinearPoints.
    /// `MakeHypr2d::from_points(s1, s2, center)?`.
    pub fn from_points(s1: Point2, s2: Point2, center: Point2) -> Result<Self, Failure> {
        let vec_s1: Xy = s1.xy() - center.xy();
        let vec_s2: Xy = s2.xy() - center.xy();
        let dist_s1: f64 = vec_s1.modulus();
        if confused(dist_s1, vec_s2.modulus(), s1.distance(s2)) {
            return Ok(Self::failed(ErrorType::ConfusedPoints));
        }
        let x_axis: Direction2 = Direction2::from_xy(vec_s1)?;
        let y_axis: Direction2 = Direction2::from_xy(vec_s2)?;
        let frame = Axis22d::with_directions(center, x_axis, y_axis)?;
        let minor_dist: f64 = Line2::new(center, x_axis).distance(s2);
        if minor_dist <= gp::resolution() {
            return Ok(Self::failed(ErrorType::ColinearPoints));
        }
        Ok(Self::built(Hyperbola2::new(frame, dist_s1, minor_dist)?))
    }

    /// gce_MakeHypr2d(MajorAxis, MajorRadius, MinorRadius, Sense): the hyperbola of
    /// major axis `major_axis`; gce_NegativeRadius when a radius is negative.
    /// `MakeHypr2d::new(axis, 2.0, 1.0, true)?`.
    pub fn new(
        major_axis: Axis2d,
        major_radius: f64,
        minor_radius: f64,
        sense: bool,
    ) -> Result<Self, Failure> {
        if major_radius < 0.0 || minor_radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let hyperbola = Hyperbola2::from_x_axis(major_axis, major_radius, minor_radius, sense)?;
        Ok(Self::built(hyperbola))
    }

    /// gce_MakeHypr2d(A, MajorRadius, MinorRadius): the hyperbola of frame `frame`;
    /// gce_NegativeRadius when a radius is negative.
    /// `MakeHypr2d::from_frame(frame, 2.0, 1.0)?`.
    pub fn from_frame(
        frame: Axis22d,
        major_radius: f64,
        minor_radius: f64,
    ) -> Result<Self, Failure> {
        if major_radius < 0.0 || minor_radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Hyperbola2::new(
            frame,
            major_radius,
            minor_radius,
        )?))
    }
}

/// The confused-points test of both three-point constructors: a point at the
/// center or the two axis ends within gp::Resolution().
fn confused(dist_s1: f64, dist_s2: f64, dist_s1_s2: f64) -> bool {
    let resolution: f64 = gp::resolution();
    dist_s1 <= resolution || dist_s2 <= resolution || dist_s1_s2 <= resolution
}
