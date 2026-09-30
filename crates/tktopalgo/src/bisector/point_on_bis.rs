// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_PointOnBis.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_PointOnBis`: a point of a bisector with its parameters on the two
//! curves it separates, on the bisector and its distance to the curves.
use rustycad_tkernel::storage::format_general;
use rustycad_tkmath::gp::Point2;

/// `Bisector_PointOnBis`: one vertex of the polygon of a [`super::BisecCC`].
/// ```
/// use rustycad_tktopalgo::bisector::PointOnBis;
/// use rustycad_tkmath::gp::Point2;
/// let p = PointOnBis::new(0.5, 0.25, 1.0, 2.0, Point2::new(1.0, 2.0));
/// assert_eq!(p.param_on_bis(), 1.0);
/// assert!(!p.is_infinite());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointOnBis {
    param1: f64,
    param2: f64,
    param_bis: f64,
    distance: f64,
    infinite: bool,
    point: Point2,
}

impl PointOnBis {
    /// `Bisector_PointOnBis(Param1, Param2, ParamBis, Distance, P)`; not infinite;
    /// `PointOnBis::new(u1, u2, u, d, p)`.
    pub fn new(param1: f64, param2: f64, param_bis: f64, distance: f64, point: Point2) -> Self {
        Self {
            param1,
            param2,
            param_bis,
            distance,
            infinite: false,
            point,
        }
    }
    /// `ParamOnC1(Param)`; `p.set_param_on_c1(0.5)`.
    pub fn set_param_on_c1(&mut self, param: f64) {
        self.param1 = param;
    }
    /// `ParamOnC2(Param)`; `p.set_param_on_c2(0.5)`.
    pub fn set_param_on_c2(&mut self, param: f64) {
        self.param2 = param;
    }
    /// `ParamOnBis(Param)`; `p.set_param_on_bis(0.5)`.
    pub fn set_param_on_bis(&mut self, param: f64) {
        self.param_bis = param;
    }
    /// `Distance(Distance)`; `p.set_distance(1.0)`.
    pub fn set_distance(&mut self, distance: f64) {
        self.distance = distance;
    }
    /// `IsInfinite(Infinite)`; `p.set_infinite(true)`.
    pub fn set_infinite(&mut self, infinite: bool) {
        self.infinite = infinite;
    }
    /// `Point(P)`; `p.set_point(Point2::new(0.0, 0.0))`.
    pub fn set_point(&mut self, point: Point2) {
        self.point = point;
    }
    /// `ParamOnC1()`; `p.param_on_c1()`.
    pub fn param_on_c1(&self) -> f64 {
        self.param1
    }
    /// `ParamOnC2()`; `p.param_on_c2()`.
    pub fn param_on_c2(&self) -> f64 {
        self.param2
    }
    /// `ParamOnBis()`; `p.param_on_bis()`.
    pub fn param_on_bis(&self) -> f64 {
        self.param_bis
    }
    /// `Distance()`; `p.distance()`.
    pub fn distance(&self) -> f64 {
        self.distance
    }
    /// `Point()`; `p.point()`.
    pub fn point(&self) -> Point2 {
        self.point
    }
    /// `IsInfinite()`; `p.is_infinite()`.
    pub fn is_infinite(&self) -> bool {
        self.infinite
    }
    /// `Dump()`: the parameters and the distance, as `std::cout` prints them;
    /// `p.dump(&mut text)`.
    pub fn dump(&self, out: &mut String) {
        let lines = [
            ("Param1    :", self.param1),
            ("Param2    :", self.param2),
            ("Param Bis :", self.param_bis),
            ("Distance  :", self.distance),
        ];
        for (label, value) in lines {
            out.push_str(&format!("{label}{}\n", format_general(value, 6)));
        }
    }
}
