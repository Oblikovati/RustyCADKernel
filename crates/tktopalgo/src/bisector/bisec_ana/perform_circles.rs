// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecAna.cxx (Perform, case 1: circle - circle) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bisector of two circles: the mixed circles, then the GccAna solutions.
use super::perform_curves::CurvesContext;
use super::solutions::{Trimmed, degenerate, line_solution, trim_conic, trimmed};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkgeomalgo::gcc_ana::Circ2dBisec;
use rustycad_tkgeomalgo::gcc_int::{Bisec, IType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, StandardDirection2};

/// The solution the loop keeps: the bisector, its first parameter, its sense and
/// the distance from the point of passage.
pub(super) struct Chosen {
    pub solution: Handle<dyn Bisec>,
    pub first: f64,
    pub sense: bool,
    pub distance: f64,
}

// The two circles after the swap and the reframing: `circle1` has the larger radius.
struct Reframed {
    circle1: Circle2,
    circle2: Circle2,
    tangent: bool,
}

impl CurvesContext {
    /// Case 1 of Perform: two circles.
    pub(super) fn bisect_circles(
        &self,
        c1: Circle2,
        c2: Circle2,
    ) -> Result<Option<Trimmed>, Failure> {
        let pre_conf: f64 = precision::confusion();
        if c1.location().is_equal(c2.location(), pre_conf)
            && (c1.radius() - c2.radius()).abs() <= pre_conf
        {
            return self.mixed_circles(&c1).map(Some);
        }
        let reframed: Reframed = self.reframe(c1, c2)?;
        let bisector = Circ2dBisec::new(reframed.circle1, reframed.circle2);
        let Some(chosen) = self.choose_circle_solution(&bisector)? else {
            return Ok(None);
        };
        if chosen.solution.arc_type() == IType::Lin {
            return self.trim_circles_line(&chosen, &reframed).map(Some);
        }
        trim_conic(&*chosen.solution, chosen.first, chosen.sense)
    }

    // The mixed circles: the line through the middle of the ends and the centre.
    fn mixed_circles(&self, c1: &Circle2) -> Result<Trimmed, Failure> {
        let p1: Point2 = self.first_curve.value(self.first_curve.last_parameter())?;
        let p2: Point2 = self
            .second_curve
            .value(self.second_curve.first_parameter())?;
        let pmil = Point2::new((p1.x() + p2.x()) / 2., (p1.y() + p2.y()) / 2.);
        let line: Line2 = mixed_circles_line(c1, p1, pmil)?;
        let result = self.sense_pass(&*line_solution(line))?;
        let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(line)).cast();
        if !result.sense {
            return trimmed(curve, result.parameter, -precision::infinite(), true);
        }
        let parameter2: f64 = elclib::parameter(&line, c1.location())? + 1.0e-8;
        trimmed(curve, result.parameter, parameter2, true)
    }

    // The swap (the first circle gets the larger radius) and, on the curve, the
    // reframing of almost tangent circles into tangent ones.
    fn reframe(&self, c1: Circle2, c2: Circle2) -> Result<Reframed, Failure> {
        let (mut circle1, mut circle2) = if c1.radius() < c2.radius() {
            (c2, c1)
        } else {
            (c1, c2)
        };
        let (radius1, radius2) = (circle1.radius(), circle2.radius());
        let entre_axe: f64 = circle1.location().distance(circle2.location());
        let mut d1: f64 = 0.5 * (radius1 - entre_axe - radius2);
        // KHROMOV, Oct 2002: the tangents must be parallel too.
        if self.oncurve
            && d1.abs() < precision::confusion()
            && self.tan1.is_parallel(self.tan2, 1.0e-8)?
        {
            // C2 included in C1 and tangent.
            circle1.set_radius(radius1 - d1)?;
            circle2.set_radius(radius2 + d1)?;
            return Ok(Reframed {
                circle1,
                circle2,
                tangent: true,
            });
        }
        d1 = 0.5 * (radius1 - entre_axe + radius2);
        if self.oncurve
            && d1.abs() < precision::confusion()
            && self.tan1.is_parallel(self.tan2, 1.0e-8)?
        {
            // C2 and C1 tangent and disconnected.
            circle1.set_radius(radius1 - d1)?;
            circle2.set_radius(radius2 - d1)?;
            return Ok(Reframed {
                circle1,
                circle2,
                tangent: true,
            });
        }
        Ok(Reframed {
            circle1,
            circle2,
            tangent: false,
        })
    }

    // The loop over the GccAna solutions: the one closest to the point of passage
    // among those whose on-curve status is correct.
    fn choose_circle_solution(&self, bisector: &Circ2dBisec) -> Result<Option<Chosen>, Failure> {
        if !bisector.is_done() {
            return Ok(None);
        }
        let mut chosen: Option<Chosen> = None;
        let mut distancemini: f64 = precision::infinite();
        for i in 1..=bisector.nb_solutions()? {
            let solution = degenerate(bisector.this_solution(i)?, self.tolerance)?;
            let ok: bool = !self.oncurve || self.status_pass(&*solution)?.ok;
            if !ok {
                continue;
            }
            let result = self.sense_pass(&*solution)?;
            if result.distance <= distancemini {
                distancemini = result.distance;
                chosen = Some(Chosen {
                    solution,
                    first: result.parameter,
                    sense: result.sense,
                    distance: result.distance,
                });
            }
        }
        Ok(chosen)
    }

    // A line solution of two circles, cut between them when they are tangent.
    fn trim_circles_line(&self, chosen: &Chosen, reframed: &Reframed) -> Result<Trimmed, Failure> {
        let gpline: Line2 = chosen.solution.line()?;
        let mut curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(gpline)).cast();
        let mut first: f64 = chosen.first;
        let mut second: f64 = if chosen.sense {
            precision::infinite()
        } else {
            -precision::infinite()
        };
        if self.oncurve && reframed.tangent {
            if let Some((line, parameter)) = self.corrected_line(chosen, &reframed.circle2)? {
                curve = Handle::new(Geom2dLine::from_lin2d(line)).cast();
                first = parameter;
            }
            if self.tan1.dot(self.tan2) < 0. {
                second = tangent_circles_end(gpline, reframed, first, chosen.sense, second)?;
            }
        }
        trimmed(curve, first, second, true)
    }

    // skv, IDEM(Airbus) Apr 2004: when the line passes too far from the point, the
    // line from its location through the point, if it passes by the second centre.
    fn corrected_line(
        &self,
        chosen: &Chosen,
        circle2: &Circle2,
    ) -> Result<Option<(Line2, f64)>, Failure> {
        if chosen.distance <= self.tolerance {
            return Ok(None);
        }
        let aploc: Point2 = chosen.solution.line()?.location();
        let new_dir = Direction2::from_xy(self.apoint.xy() - aploc.xy())?;
        let new_lin = Line2::new(aploc, new_dir);
        let a_tol_conf: f64 = 1.0e-3;
        // The new distance (hope it is 0) replaces the minimum natively, but nothing
        // reads the minimum afterwards.
        if new_lin.distance(circle2.location()) <= a_tol_conf {
            return Ok(Some((new_lin, elclib::parameter(&new_lin, self.apoint)?)));
        }
        Ok(None)
    }
}

// The line of mixed circles (skv, IDEM(Airbus) Jul 2005): through the middle point
// and the centre, or normal to the radius at the end when they coincide, or along
// X for a null radius.
fn mixed_circles_line(c1: &Circle2, p1: Point2, pmil: Point2) -> Result<Line2, Failure> {
    let pre_conf: f64 = precision::confusion();
    let center: Point2 = c1.location();
    if !center.is_equal(pmil, pre_conf) {
        let dir = Direction2::new(center.x() - pmil.x(), center.y() - pmil.y())?;
        return Ok(Line2::new(pmil, dir));
    }
    if c1.radius() >= pre_conf {
        let dir = Direction2::new(p1.y() - center.y(), center.x() - p1.x())?;
        return Ok(Line2::new(center, dir));
    }
    Ok(Line2::new(
        center,
        Direction2::from_standard(StandardDirection2::X),
    ))
}

// The end of a line bisecting tangent circles "flat and not turn back": 1e-8 past
// the first centre it meets on its side (the parameters of the original line).
fn tangent_circles_end(
    gpline: Line2,
    reframed: &Reframed,
    first: f64,
    sense: bool,
    second: f64,
) -> Result<f64, Failure> {
    let par1: f64 = elclib::parameter(&gpline, reframed.circle1.location())?;
    let par2: f64 = elclib::parameter(&gpline, reframed.circle2.location())?;
    let (min_par, max_par) = (std_min(par1, par2), std_max(par1, par2));
    if !sense {
        if max_par < first {
            return Ok(max_par - 1.0E-8);
        }
        if min_par < first {
            return Ok(min_par - 1.0E-8);
        }
        return Ok(second);
    }
    if min_par > first {
        return Ok(min_par + 1.0E-8);
    }
    if max_par > first {
        return Ok(max_par + 1.0E-8);
    }
    Ok(second)
}
