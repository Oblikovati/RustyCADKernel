// Rust translation of IntAna_IntConicQuad.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_IntConicQuad: the intersection of a gp conic with a quadric; the
//! general case is in `quadric`, the plane special cases in `plane`.
mod plane;
mod quadric;

use super::Quadric;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Plane3, Point3};

/// IntAna_IntConicQuad: the points (with their parameter on the conic) of the
/// intersection of a line, circle, ellipse, parabola or hyperbola with a
/// [`Quadric`], or, as special cases, with a plane.
///
/// `let inter = IntConicQuad::line_quadric(line, &Quadric::from_sphere(sphere))?;`
#[derive(Clone, Copy, Debug)]
pub struct IntConicQuad {
    done: bool,
    parallel: bool,
    inquadric: bool,
    nbpts: i32,
    pnts: [Point3; 4],
    paramonc: [f64; 4],
}

impl Default for IntConicQuad {
    /// IntAna_IntConicQuad(): not done, no point, null parameters.
    fn default() -> Self {
        Self {
            done: false,
            parallel: false,
            inquadric: false,
            nbpts: 0,
            pnts: [Point3::default(); 4],
            paramonc: [0.0; 4],
        }
    }
}

/// Builds an intersection with one `perform_*` member.
macro_rules! conic_quad_constructor {
    ($(#[$doc:meta])* $name:ident => $perform:ident($($arg:ident: $ty:ty),*)) => {
        $(#[$doc])*
        pub fn $name($($arg: $ty),*) -> Result<Self, Failure> {
            let mut inter: Self = Self::default();
            inter.$perform($($arg),*)?;
            Ok(inter)
        }
    };
}

impl IntConicQuad {
    /// IntAna_IntConicQuad(): the empty constructor. `IntConicQuad::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Lin, IntAna_Quadric). `IntConicQuad::line_quadric(l, &q)?`.
        line_quadric => perform_line_quadric(line: Line3, quadric: &Quadric));
    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Circ, IntAna_Quadric). `IntConicQuad::circle_quadric(c, &q)?`.
        circle_quadric => perform_circle_quadric(circle: Circle3, quadric: &Quadric));
    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Elips, IntAna_Quadric). `IntConicQuad::ellipse_quadric(e, &q)?`.
        ellipse_quadric => perform_ellipse_quadric(ellipse: Ellipse3, quadric: &Quadric));
    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Parab, IntAna_Quadric). `IntConicQuad::parabola_quadric(p, &q)?`.
        parabola_quadric => perform_parabola_quadric(parabola: Parabola3, quadric: &Quadric));
    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Hypr, IntAna_Quadric). `IntConicQuad::hyperbola_quadric(h, &q)?`.
        hyperbola_quadric => perform_hyperbola_quadric(hyperbola: Hyperbola3, quadric: &Quadric));
    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Lin, gp_Pln, Tolang, Tol, Len) (the native
        /// defaults of Tol and Len are 0). `IntConicQuad::line_plane(l, p, 1e-12, 0.0, 0.0)?`.
        line_plane => perform_line_plane(line: Line3, plane: Plane3, tolang: f64, tol: f64, len: f64));
    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Circ, gp_Pln, Tolang, Tol).
        /// `IntConicQuad::circle_plane(c, p, 1e-12, 1e-7)?`.
        circle_plane => perform_circle_plane(circle: Circle3, plane: Plane3, tolang: f64, tol: f64));
    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Elips, gp_Pln, Tolang, Tol).
        /// `IntConicQuad::ellipse_plane(e, p, 1e-12, 1e-7)?`.
        ellipse_plane => perform_ellipse_plane(ellipse: Ellipse3, plane: Plane3, tolang: f64, tol: f64));
    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Parab, gp_Pln, Tolang).
        /// `IntConicQuad::parabola_plane(p, pln, 1e-12)?`.
        parabola_plane => perform_parabola_plane(parabola: Parabola3, plane: Plane3, tolang: f64));
    conic_quad_constructor!(
        /// IntAna_IntConicQuad(gp_Hypr, gp_Pln, Tolang).
        /// `IntConicQuad::hyperbola_plane(h, pln, 1e-12)?`.
        hyperbola_plane => perform_hyperbola_plane(hyperbola: Hyperbola3, plane: Plane3, tolang: f64));

    /// IsDone(): true when the computation completed. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// IsInQuadric(): the conic lies in the quadric; StdFail_NotDone when not
    /// done. `inter.is_in_quadric()?`.
    pub fn is_in_quadric(&self) -> Result<bool, Failure> {
        self.check_done("IsInQuadric")?;
        Ok(self.inquadric)
    }

    /// IsParallel(): the conic lies in a quadric parallel to the quadric (a line
    /// or a circle parallel to the plane); StdFail_NotDone when not done.
    /// `inter.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        self.check_done("IsParallel")?;
        Ok(self.parallel)
    }

    /// NbPoints(): the number of points; StdFail_NotDone when not done,
    /// Standard_DomainError when parallel or in the quadric. `inter.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.check_points("NbPoints")?;
        Ok(self.nbpts)
    }

    /// Point(N): the point of range `n`; the errors of [`Self::nb_points`] and
    /// Standard_OutOfRange outside `1..=NbPoints()`. `inter.point(1)?`.
    pub fn point(&self, n: i32) -> Result<Point3, Failure> {
        Ok(self.pnts[self.check_index("Point", n)?])
    }

    /// ParamOnConic(N): the parameter on the conic of the point of range `n`,
    /// with the errors of [`Self::point`]. `inter.param_on_conic(1)?`.
    pub fn param_on_conic(&self, n: i32) -> Result<f64, Failure> {
        Ok(self.paramonc[self.check_index("ParamOnConic", n)?])
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(Failure::with_type(
            "StdFail_NotDone",
            &format!(
                "IntAna_IntConicQuad::{member}() - the intersection is not done; expected IsDone()"
            ),
            "",
        ))
    }

    fn check_points(&self, member: &str) -> Result<(), Failure> {
        self.check_done(member)?;
        if self.parallel || self.inquadric {
            return Err(Failure::with_type(
                "Standard_DomainError",
                &format!(
                    "IntAna_IntConicQuad::{member}() - parallel {} / in quadric {}; expected isolated points",
                    self.parallel, self.inquadric
                ),
                "",
            ));
        }
        Ok(())
    }

    fn check_index(&self, member: &str, n: i32) -> Result<usize, Failure> {
        self.check_points(member)?;
        if n > self.nbpts || n <= 0 {
            return Err(Failure::out_of_range(&format!(
                "IntAna_IntConicQuad::{member}() - index {n} outside [1, {}]",
                self.nbpts
            )));
        }
        Ok((n - 1) as usize)
    }
}
