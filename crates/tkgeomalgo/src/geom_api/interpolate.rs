// Rust translation of GeomAPI_Interpolate.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_Interpolate`: the 3D instantiation of the interpolation body shared with
//! Geom2dAPI_Interpolate (docs/geomalgo-design.md section 15).
use crate::geom2d_api::InterpolateGen;
use crate::geom2d_api::interpolate_space::InterpolateSpace;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `GeomAPI_Interpolate`: a B-spline curve through 3D points, with optional tangents.
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::Interpolate;
/// use rustycad_tkernel::ncollection::{Array1, HArray1};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::Point3;
/// let mut points = Array1::with_bounds(1, 3).unwrap();
/// points[1] = Point3::new(0.0, 0.0, 0.0);
/// points[2] = Point3::new(1.0, 1.0, 0.5);
/// points[3] = Point3::new(2.0, 0.0, 1.0);
/// let mut interp = Interpolate::new(&Handle::new(HArray1::from_owned(points)), false, 1e-7).unwrap();
/// interp.perform().unwrap();
/// assert!(interp.is_done());
/// ```
pub type Interpolate = InterpolateGen<Interpolate3dSpace>;

/// The 3D space of GeomAPI_Interpolate: `gp_Pnt`, `gp_Vec` and `Geom_BSplineCurve`.
pub struct Interpolate3dSpace;

impl InterpolateSpace for Interpolate3dSpace {
    type Point = Point3;
    type Vector = Vector3;
    type Curve = GeomBSplineCurve;
    const NAME: &'static str = "GeomAPI_Interpolate";
    fn square_distance(a: &Point3, b: &Point3) -> f64 {
        a.square_distance(b)
    }
    fn distance(a: &Point3, b: &Point3) -> f64 {
        a.distance(b)
    }
    fn square_magnitude(v: &Vector3) -> f64 {
        v.square_magnitude()
    }
    fn new_curve(
        poles: &Array1<'_, Point3>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<GeomBSplineCurve, Failure> {
        GeomBSplineCurve::new(poles, knots, mults, degree, periodic)
    }
}
