// Rust translation of ModelingAlgorithms/TKHLR/Contap/Contap_ContAna.hxx/.lxx/.cxx (the class, the sphere and the cylinder) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contap_ContAna: the contour lines of the elementary quadrics, in closed form.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Line3, Point3, Sphere3};

/// The file constant `Tolpetit`: a smaller sphere silhouette radius is no contour.
pub(super) const TOLPETIT: f64 = 1.0e-8;

/// Contap_ContAna: the silhouette (seen along a direction or from an eye) and the draft
/// lines (along a direction at an angle) of a sphere, a cylinder or a cone: a circle
/// for the sphere, 0 or 2 lines for the cylinder, 0, 2 or 4 lines for the cone.
///
/// ```
/// use rustycad_tkhlr::contap::ContapContAna;
/// use rustycad_tkmath::geom_abs::CurveType;
/// use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Sphere3};
/// let sphere = Sphere3::new(Axis3::default(), 2.0).unwrap();
/// let mut contour = ContapContAna::new();
/// contour.perform_sphere(&sphere, Direction3::Z).unwrap();
/// assert_eq!(contour.nb_contours().unwrap(), 1);
/// assert_eq!(contour.type_contour().unwrap(), CurveType::Circle);
/// assert_eq!(contour.circle().unwrap().radius(), 2.0);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ContapContAna {
    pub(super) done: bool,
    pub(super) nb_sol: i32,
    pub(super) typ_l: CurveType,
    pub(super) pt: [Point3; 4],
    pub(super) dir: [Direction3; 4],
    pub(super) prm: f64,
}

impl Default for ContapContAna {
    fn default() -> Self {
        Self::new()
    }
}

/// The failure of an accessor before a Perform (StdFail_NotDone).
fn not_done(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!("Contap_ContAna::{member}: no Perform yet, expected a done computation"),
    )
}

/// The Standard_DomainError of an accessor used on the wrong kind of solution.
fn wrong_solution(member: &str, typ_l: CurveType, nb_sol: i32) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!("Contap_ContAna::{member}: {nb_sol} contours of type {typ_l:?}"),
    )
}

impl ContapContAna {
    /// Contap_ContAna(): not done, no solution; `ContapContAna::new()`.
    pub fn new() -> Self {
        Self {
            done: false,
            nb_sol: 0,
            typ_l: CurveType::OtherCurve,
            pt: [Point3::default(); 4],
            dir: [Direction3::default(); 4],
            prm: 0.0,
        }
    }

    /// IsDone; `contour.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// NbContours: StdFail_NotDone before a Perform; `contour.nb_contours()?`.
    pub fn nb_contours(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("NbContours"));
        }
        Ok(self.nb_sol)
    }

    /// TypeContour: StdFail_NotDone before a Perform, Standard_DomainError without
    /// contour; `contour.type_contour()?`.
    pub fn type_contour(&self) -> Result<CurveType, Failure> {
        if !self.done {
            return Err(not_done("TypeContour"));
        }
        if self.nb_sol == 0 {
            return Err(wrong_solution("TypeContour", self.typ_l, self.nb_sol));
        }
        Ok(self.typ_l)
    }

    /// Circle: StdFail_NotDone before a Perform, Standard_DomainError unless the
    /// contour is a circle; `contour.circle()?`.
    pub fn circle(&self) -> Result<Circle3, Failure> {
        if !self.done {
            return Err(not_done("Circle"));
        }
        if self.typ_l != CurveType::Circle || self.nb_sol == 0 {
            return Err(wrong_solution("Circle", self.typ_l, self.nb_sol));
        }
        Circle3::new(
            Axis2::with_x_direction(self.pt[0], self.dir[0], self.dir[1])?,
            self.prm,
        )
    }

    /// Line(Index): StdFail_NotDone before a Perform, Standard_DomainError unless the
    /// contours are lines, Standard_OutOfRange outside `1..=NbContours()`;
    /// `contour.line(1)?`.
    pub fn line(&self, index: i32) -> Result<Line3, Failure> {
        if !self.done {
            return Err(not_done("Line"));
        }
        if self.typ_l != CurveType::Line || self.nb_sol == 0 {
            return Err(wrong_solution("Line", self.typ_l, self.nb_sol));
        }
        if index <= 0 || index > self.nb_sol {
            return Err(Failure::out_of_range(&format!(
                "Contap_ContAna::Line: index {index} outside 1..={}",
                self.nb_sol
            )));
        }
        let k = (index - 1) as usize;
        Ok(Line3::new(self.pt[k], self.dir[k]))
    }
}

/// The X direction of the circle of a sphere normal to `d`: `d` crossed with the sphere
/// X axis, or with its Y axis when they are nearly parallel.
pub(super) fn circle_x_direction(s: &Sphere3, d: Direction3) -> Result<Direction3, Failure> {
    if d.dot(s.x_axis().direction()).abs() < 0.9999999999999 {
        return d.crossed(s.x_axis().direction());
    }
    d.crossed(s.y_axis().direction())
}

/// The cosines and sines of the two roots of `Coefcos cos(t) + Coefsin sin(t) =
/// Coefcst`, with `prm = sqrt(norm1 - Coefcst^2)` and `norm1 = Coefcos^2 + Coefsin^2`.
pub(super) fn trigonometric_roots(
    coefcos: f64,
    coefsin: f64,
    coefcst: f64,
    prm: f64,
    norm1: f64,
) -> [(f64, f64); 2] {
    let cost0 = (coefcos * coefcst - coefsin * prm) / norm1;
    let cost1 = (coefcos * coefcst + coefsin * prm) / norm1;
    let sint0 = (coefcos * prm + coefsin * coefcst) / norm1;
    let sint1 = (-coefcos * prm + coefsin * coefcst) / norm1;
    [(cost0, sint0), (cost1, sint1)]
}
