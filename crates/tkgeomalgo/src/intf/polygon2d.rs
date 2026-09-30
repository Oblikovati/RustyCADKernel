// Rust translation of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_Polygon2d.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_Polygon2d: the polygon description the 2d interference reads.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point2;

/// Intf_Polygon2d: an abstract polyline (its derived classes, such as the polygons of
/// Geom2dInt and HLRBRep, implement this trait). The protected `myBox` of the native
/// class is the box that `bounding` returns, kept by the implementor.
///
/// ```
/// use rustycad_tkgeomalgo::intf::Polygon2d;
/// use rustycad_tkernel::standard::Failure;
/// use rustycad_tkmath::bnd::BoundingBox2;
/// use rustycad_tkmath::gp::Point2;
/// struct Segment(BoundingBox2);
/// impl Polygon2d for Segment {
///     fn bounding(&self) -> &BoundingBox2 { &self.0 }
///     fn deflection_over_estimation(&self) -> f64 { 0.0 }
///     fn nb_segments(&self) -> i32 { 1 }
///     fn segment(&self, _index: i32) -> Result<(Point2, Point2), Failure> {
///         Ok((Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)))
///     }
/// }
/// assert!(!Segment(BoundingBox2::default()).closed());
/// ```
pub trait Polygon2d {
    /// Bounding: the bounding box of the polygon.
    fn bounding(&self) -> &BoundingBox2;

    /// Closed: whether the polyline is closed; false unless overridden, as natively.
    fn closed(&self) -> bool {
        false
    }

    /// DeflectionOverEstimation: the tolerance of the polygon.
    fn deflection_over_estimation(&self) -> f64;

    /// NbSegments: the number of segments of the polyline.
    fn nb_segments(&self) -> i32;

    /// Segment: the begin and end points of segment `index` (one-based); an
    /// implementation raises its own exception for a bad index.
    fn segment(&self, index: i32) -> Result<(Point2, Point2), Failure>;
}
