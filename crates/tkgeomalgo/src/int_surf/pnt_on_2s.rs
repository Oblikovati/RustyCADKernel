// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntSurf/IntSurf_PntOn2S.hxx/.lxx/.cxx and
// IntSurf_Couple.hxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntSurf_PntOn2S` (a point on two surfaces) and `IntSurf_Couple` (a pair of indices).
use rustycad_tkmath::gp::{Point2, Point3};

/// A 3d point with its parameters on two surfaces (`IntSurf_PntOn2S`).
///
/// `let mut p = IntSurfPntOn2S::new(); p.set_value_all(Point3::new(1.0, 0.0, 0.0), 0.0, 1.0, 2.0, 3.0);`
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IntSurfPntOn2S {
    pt: Point3,
    u1: f64,
    v1: f64,
    u2: f64,
    v2: f64,
}

impl IntSurfPntOn2S {
    /// The origin with all parameters 0 (`IntSurf_PntOn2S()`); `IntSurfPntOn2S::new()`.
    pub const fn new() -> Self {
        Self {
            pt: Point3::new(0.0, 0.0, 0.0),
            u1: 0.0,
            v1: 0.0,
            u2: 0.0,
            v2: 0.0,
        }
    }

    /// Sets the 3d point only (`SetValue(Pt)`); `p.set_point(Point3::new(1.0, 2.0, 3.0))`.
    pub fn set_point(&mut self, pt: Point3) {
        self.pt = pt;
    }

    /// Sets the 3d point and the parameters on one surface (`SetValue(Pt, OnFirst, U, V)`).
    /// `p.set_value_on(Point3::new(0.0, 0.0, 0.0), true, 0.5, 0.25)`.
    pub fn set_value_on(&mut self, pt: Point3, on_first: bool, u: f64, v: f64) {
        self.pt = pt;
        self.set_parameters_on(on_first, u, v);
    }

    /// Sets the 3d point and the parameters on both surfaces (`SetValue(Pt, U1, V1, U2, V2)`).
    /// `p.set_value_all(Point3::new(0.0, 0.0, 0.0), 0.0, 1.0, 2.0, 3.0)`.
    pub fn set_value_all(&mut self, pt: Point3, u1: f64, v1: f64, u2: f64, v2: f64) {
        self.pt = pt;
        self.set_parameters(u1, v1, u2, v2);
    }

    /// Sets the parameters on one surface (`SetValue(OnFirst, U, V)`).
    /// `p.set_parameters_on(false, 0.5, 0.25)`.
    pub fn set_parameters_on(&mut self, on_first: bool, u: f64, v: f64) {
        if on_first {
            self.u1 = u;
            self.v1 = v;
        } else {
            self.u2 = u;
            self.v2 = v;
        }
    }

    /// Sets the parameters on both surfaces (`SetValue(U1, V1, U2, V2)`).
    /// `p.set_parameters(0.0, 1.0, 2.0, 3.0)`.
    pub fn set_parameters(&mut self, u1: f64, v1: f64, u2: f64, v2: f64) {
        self.u1 = u1;
        self.v1 = v1;
        self.u2 = u2;
        self.v2 = v2;
    }

    /// The 3d point (`Value`); `p.value().x()`.
    pub const fn value(&self) -> Point3 {
        self.pt
    }

    /// The parameters on one surface as a 2d point (`ValueOnSurface`); `p.value_on_surface(true)`.
    pub const fn value_on_surface(&self, on_first: bool) -> Point2 {
        let (u, v) = self.parameters_on_surface(on_first);
        Point2::new(u, v)
    }

    /// `(U1, V1)` (`ParametersOnS1`); `p.parameters_on_s1()`.
    pub const fn parameters_on_s1(&self) -> (f64, f64) {
        (self.u1, self.v1)
    }

    /// `(U2, V2)` (`ParametersOnS2`); `p.parameters_on_s2()`.
    pub const fn parameters_on_s2(&self) -> (f64, f64) {
        (self.u2, self.v2)
    }

    /// The parameters on one surface (`ParametersOnSurface`); `p.parameters_on_surface(false)`.
    pub const fn parameters_on_surface(&self, on_first: bool) -> (f64, f64) {
        if on_first {
            (self.u1, self.v1)
        } else {
            (self.u2, self.v2)
        }
    }

    /// `(U1, V1, U2, V2)` (`Parameters`); `p.parameters()`.
    pub const fn parameters(&self) -> (f64, f64, f64, f64) {
        (self.u1, self.v1, self.u2, self.v2)
    }

    /// Whether `other` has the same 3d point within `tol_3d` and, when `tol_2d` is not
    /// negative, the same parameters on both surfaces within `tol_2d` (`IsSame`; the
    /// native defaults are 0 and -1). `p.is_same(&q, 1e-7, -1.0)`.
    pub fn is_same(&self, other: &Self, tol_3d: f64, tol_2d: f64) -> bool {
        if self.pt.square_distance(&other.value()) > tol_3d * tol_3d {
            return false;
        }
        if tol_2d < 0.0 {
            // Only the 3d points are compared.
            return true;
        }
        let (au1, av1, au2, av2) = other.parameters();
        let (mut p1, mut p2) = (Point2::new(self.u1, self.v1), Point2::new(au1, av1));
        if !p1.is_equal(p2, tol_2d) {
            return false;
        }
        p1.set_coordinates(self.u2, self.v2);
        p2.set_coordinates(au2, av2);
        p1.is_equal(p2, tol_2d)
    }
}

/// A couple of indices (`IntSurf_Couple`); `IntSurfCouple::with_indices(1, 2).second() == 2`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IntSurfCouple {
    first_integer: i32,
    second_integer: i32,
}

impl IntSurfCouple {
    /// The couple (0, 0) (`IntSurf_Couple()`); `IntSurfCouple::new()`.
    pub const fn new() -> Self {
        Self::with_indices(0, 0)
    }

    /// The couple `(index1, index2)` (`IntSurf_Couple(Index1, Index2)`).
    /// `IntSurfCouple::with_indices(3, 4)`.
    pub const fn with_indices(index1: i32, index2: i32) -> Self {
        Self {
            first_integer: index1,
            second_integer: index2,
        }
    }

    /// The first index (`First`); `c.first()`.
    pub const fn first(&self) -> i32 {
        self.first_integer
    }

    /// The second index (`Second`); `c.second()`.
    pub const fn second(&self) -> i32 {
        self.second_integer
    }
}
