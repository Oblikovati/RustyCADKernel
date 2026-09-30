// Rust translation of the GccAna branches of Geom2dGcc_Circ2d3Tan.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GccAna_Circ2d3Tan calls of Geom2dGcc_Circ2d3Tan: the circles come first in
//! the GccAna argument order, and the qualifiers and results are mapped back.
use super::Circ2d3Tan;
use crate::gcc_ana::Circ2d3Tan as Ana;
use crate::gcc_ent::{QualifiedCirc, QualifiedLin};
use crate::geom2d_gcc::{QualifiedCurve, circle_of, line_of, point_of};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dPoint;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point2;

// A line or circle argument as GccAna takes it.
#[derive(Clone, Copy)]
enum Analytic {
    Circ(QualifiedCirc),
    Lin(QualifiedLin),
}

// The GccEnt argument of a line or circle curve; None for another curve.
fn analytic_of(q: &QualifiedCurve) -> Result<Option<Analytic>, Failure> {
    Ok(match q.curve().get_type()? {
        CurveType::Circle => Some(Analytic::Circ(QualifiedCirc::new(
            circle_of(q.curve())?,
            q.qualifier(),
        ))),
        CurveType::Line => Some(Analytic::Lin(QualifiedLin::new(
            line_of(q.curve())?,
            q.qualifier(),
        ))),
        _ => None,
    })
}

impl Circ2d3Tan {
    // GccAna_Circ2d3Tan of three lines and circles; false for another curve. The
    // (line, circle, circle) case copies the GccAna qualifiers into the slots
    // (3, 1, 2) where the results come from the ranks (3, 1, 2), as natively.
    pub(super) fn three_analytic(
        &mut self,
        qs: [&QualifiedCurve; 3],
        tol: f64,
    ) -> Result<bool, Failure> {
        use Analytic::{Circ, Lin};
        let (Some(a1), Some(a2), Some(a3)) = (
            analytic_of(qs[0])?,
            analytic_of(qs[1])?,
            analytic_of(qs[2])?,
        ) else {
            return Ok(false);
        };
        let (ana, into, ranks) = match (a1, a2, a3) {
            (Circ(c1), Circ(c2), Circ(c3)) => {
                (Ana::circ_circ_circ(c1, c2, c3, tol)?, [0, 1, 2], [1, 2, 3])
            }
            (Circ(c1), Circ(c2), Lin(l3)) => {
                (Ana::circ_circ_lin(c1, c2, l3, tol)?, [0, 1, 2], [1, 2, 3])
            }
            (Circ(c1), Lin(l2), Circ(c3)) => {
                (Ana::circ_circ_lin(c1, c3, l2, tol)?, [0, 2, 1], [1, 3, 2])
            }
            (Circ(c1), Lin(l2), Lin(l3)) => {
                (Ana::circ_lin_lin(c1, l2, l3, tol)?, [0, 1, 2], [1, 2, 3])
            }
            (Lin(l1), Circ(c2), Circ(c3)) => {
                (Ana::circ_circ_lin(c2, c3, l1, tol)?, [2, 0, 1], [3, 1, 2])
            }
            (Lin(l1), Circ(c2), Lin(l3)) => {
                (Ana::circ_lin_lin(c2, l1, l3, tol)?, [1, 0, 2], [2, 1, 3])
            }
            (Lin(l1), Lin(l2), Circ(c3)) => {
                (Ana::circ_lin_lin(c3, l2, l1, tol)?, [2, 1, 0], [3, 2, 1])
            }
            (Lin(l1), Lin(l2), Lin(l3)) => {
                (Ana::lin_lin_lin(l1, l2, l3, tol)?, [0, 1, 2], [1, 2, 3])
            }
        };
        self.store_ana(&ana, into, ranks)?;
        Ok(true)
    }

    // GccAna_Circ2d3Tan of two lines or circles and a point; false for another curve.
    pub(super) fn two_analytic_point(
        &mut self,
        qs: [&QualifiedCurve; 2],
        point: &Handle<dyn Geom2dPoint>,
        tol: f64,
    ) -> Result<bool, Failure> {
        use Analytic::{Circ, Lin};
        let (Some(a1), Some(a2)) = (analytic_of(qs[0])?, analytic_of(qs[1])?) else {
            return Ok(false);
        };
        let (ana, into, ranks) = match (a1, a2) {
            (Circ(c1), Circ(c2)) => (
                Ana::circ_circ_pnt(c1, c2, point_of(point)?, tol)?,
                [0, 1, 2],
                [1, 2, 3],
            ),
            (Circ(c1), Lin(l2)) => (
                Ana::circ_lin_pnt(c1, l2, point_of(point)?, tol)?,
                [0, 1, 2],
                [1, 2, 3],
            ),
            (Lin(l1), Circ(c2)) => (
                Ana::circ_lin_pnt(c2, l1, point_of(point)?, tol)?,
                [1, 0, 2],
                [2, 1, 3],
            ),
            (Lin(l1), Lin(l2)) => (
                Ana::lin_lin_pnt(l1, l2, point_of(point)?, tol)?,
                [0, 1, 2],
                [1, 2, 3],
            ),
        };
        self.store_ana(&ana, into, ranks)?;
        Ok(true)
    }

    // GccAna_Circ2d3Tan of a line or circle and two points; false for another curve.
    pub(super) fn one_analytic_points(
        &mut self,
        q1: &QualifiedCurve,
        points: [&Handle<dyn Geom2dPoint>; 2],
        tol: f64,
    ) -> Result<bool, Failure> {
        let Some(a1) = analytic_of(q1)? else {
            return Ok(false);
        };
        let (p1, p2): (Point2, Point2) = (point_of(points[0])?, point_of(points[1])?);
        let ana = match a1 {
            Analytic::Circ(c1) => Ana::circ_pnt_pnt(c1, p1, p2, tol)?,
            Analytic::Lin(l1) => Ana::lin_pnt_pnt(l1, p1, p2, tol)?,
        };
        self.store_ana(&ana, [0, 1, 2], [1, 2, 3])?;
        Ok(true)
    }
}
