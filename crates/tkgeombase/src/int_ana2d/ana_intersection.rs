// Rust translation of IntAna2d_AnaIntersection.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_AnaIntersection: the state, the constructors and the result accessors;
//! each Perform overload lives in the module of its native source file.
use super::{Conic, IntPoint};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};

/// IntAna2d_AnaIntersection: the analytic intersection of two lines, two circles,
/// a line and a circle, or a gp conic and an implicit [`Conic`], with the machine
/// precision as the only tolerance.
///
/// `let inter = AnaIntersection::lines(l1, l2); let n = inter.nb_points()?;`
#[derive(Clone, Copy, Debug)]
pub struct AnaIntersection {
    pub(super) done: bool,
    pub(super) para: bool,
    pub(super) iden: bool,
    pub(super) empt: bool,
    pub(super) nbp: i32,
    pub(super) lpnt: [IntPoint; 4],
}

impl Default for AnaIntersection {
    /// IntAna2d_AnaIntersection(): not done, empty. `AnaIntersection::default().is_done()`.
    fn default() -> Self {
        Self {
            done: false,
            para: false,
            iden: false,
            empt: true,
            nbp: 0,
            lpnt: [IntPoint::default(); 4],
        }
    }
}

/// The StdFail_NotDone raised by every accessor of an intersection not done.
fn not_done(member: &str) -> Failure {
    Failure::with_type(
        "StdFail_NotDone",
        &format!(
            "IntAna2d_AnaIntersection::{member}() - the intersection is not done; expected IsDone()"
        ),
        "",
    )
}

impl AnaIntersection {
    /// IntAna2d_AnaIntersection(): the empty constructor. `AnaIntersection::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// IntAna2d_AnaIntersection(gp_Lin2d, gp_Lin2d). `AnaIntersection::lines(l1, l2)`.
    pub fn lines(first: Line2, second: Line2) -> Self {
        let mut inter: Self = Self::default();
        inter.perform_lines(first, second);
        inter
    }

    /// IntAna2d_AnaIntersection(gp_Circ2d, gp_Circ2d). `AnaIntersection::circles(c1, c2)?`.
    pub fn circles(first: Circle2, second: Circle2) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform_circles(first, second)?;
        Ok(inter)
    }

    /// IntAna2d_AnaIntersection(gp_Lin2d, gp_Circ2d). `AnaIntersection::line_circle(l, c)?`.
    pub fn line_circle(line: Line2, circle: Circle2) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform_line_circle(line, circle)?;
        Ok(inter)
    }

    /// IntAna2d_AnaIntersection(gp_Lin2d, IntAna2d_Conic).
    /// `AnaIntersection::line_conic(l, &conic)?`.
    pub fn line_conic(line: Line2, conic: &Conic) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform_line_conic(line, conic)?;
        Ok(inter)
    }

    /// IntAna2d_AnaIntersection(gp_Circ2d, IntAna2d_Conic).
    /// `AnaIntersection::circle_conic(c, &conic)?`.
    pub fn circle_conic(circle: Circle2, conic: &Conic) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform_circle_conic(circle, conic)?;
        Ok(inter)
    }

    /// IntAna2d_AnaIntersection(gp_Elips2d, IntAna2d_Conic).
    /// `AnaIntersection::ellipse_conic(e, &conic)?`.
    pub fn ellipse_conic(ellipse: Ellipse2, conic: &Conic) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform_ellipse_conic(ellipse, conic)?;
        Ok(inter)
    }

    /// IntAna2d_AnaIntersection(gp_Parab2d, IntAna2d_Conic).
    /// `AnaIntersection::parabola_conic(p, &conic)?`.
    pub fn parabola_conic(parabola: Parabola2, conic: &Conic) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform_parabola_conic(parabola, conic)?;
        Ok(inter)
    }

    /// IntAna2d_AnaIntersection(gp_Hypr2d, IntAna2d_Conic).
    /// `AnaIntersection::hyperbola_conic(h, &conic)?`.
    pub fn hyperbola_conic(hyperbola: Hyperbola2, conic: &Conic) -> Result<Self, Failure> {
        let mut inter: Self = Self::default();
        inter.perform_hyperbola_conic(hyperbola, conic)?;
        Ok(inter)
    }

    /// IsDone(): true when the computation succeeded. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// IsEmpty(): no point and the elements are not identical (they may be
    /// parallel); StdFail_NotDone when not done. `inter.is_empty()?`.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("IsEmpty"));
        }
        Ok(self.nbp == 0 && !self.iden)
    }

    /// IdenticalElements(): the elements are identical (two lines or two
    /// circles), or the gp element verifies the implicit equation;
    /// StdFail_NotDone when not done. `inter.identical_elements()?`.
    pub fn identical_elements(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("IdenticalElements"));
        }
        Ok(self.iden)
    }

    /// ParallelElements(): two lines or two circles are parallel;
    /// StdFail_NotDone when not done. `inter.parallel_elements()?`.
    pub fn parallel_elements(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("ParallelElements"));
        }
        Ok(self.para)
    }

    /// NbPoints(): the number of intersection points; StdFail_NotDone when not
    /// done. `inter.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("NbPoints"));
        }
        Ok(self.nbp)
    }

    /// Point(N): the intersection point of range `n`; StdFail_NotDone when not
    /// done, Standard_OutOfRange outside `1..=NbPoints()`. `inter.point(1)?`.
    pub fn point(&self, n: i32) -> Result<IntPoint, Failure> {
        if !self.done {
            return Err(not_done("Point"));
        }
        if n <= 0 || n > self.nbp {
            return Err(Failure::out_of_range(&format!(
                "IntAna2d_AnaIntersection::Point() - index {n} outside [1, {}]",
                self.nbp
            )));
        }
        Ok(self.lpnt[(n - 1) as usize])
    }
}
