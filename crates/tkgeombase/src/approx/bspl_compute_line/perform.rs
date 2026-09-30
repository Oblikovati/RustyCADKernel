// Rust translation of ModelingData/TKGeomBase/Approx/Approx_BSplComputeLine.gxx (Perform, Parameters, FindRealConstraints)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_BSplComputeLine::Perform`: the fit with the given knots, or with more
//! and more uniformly spread knots.
use super::BSplComputeLine;
use crate::app_par_curves::arrays::scratch;
use crate::app_par_curves::{Constraint, ConstraintCouple, LineTool};
use crate::approx::compute_line::owned_array;
use crate::approx::status::ParametrizationType;
use crate::approx::tangency::{cumulated_chords, line_tangency};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkmath::gp::{Vector2, Vector3};
use rustycad_tkmath::math::Vector;

impl BSplComputeLine {
    /// `Perform(Line)`: approximates `line`; `compute.perform(&line)?`.
    pub fn perform<L: LineTool + Clone>(&mut self, line: &L) -> Result<(), Failure> {
        self.find_real_constraints(line)?;
        let (first, last) = (line.first_point(), line.last_point()?);
        self.couples = [
            ConstraintCouple::new(first, self.real_first_c),
            ConstraintCouple::new(last, self.real_last_c),
        ];
        let the_param: Vector<'static> = self.initial_parameters(line, first, last)?;
        self.my_parameters = Some(owned_array(&the_param)?);
        self.alldone = false;
        if !self.options.cutting {
            return self.perform_fixed_knots(line, [first, last], &the_param);
        }
        self.perform_adding_knots(line, [first, last], &the_param)
    }

    // TheParam: the parametrization, or the stored initial parameters from the first
    // point on.
    pub(super) fn initial_parameters<L: LineTool>(
        &self,
        line: &L,
        first: i32,
        last: i32,
    ) -> Result<Vector<'static>, Failure> {
        let mut the_param: Vector<'static> = Vector::filled(first, last, 0.0)?;
        match &self.first_param {
            None => self.compute_parameters(line, first, last, &mut the_param)?,
            Some(params) => {
                for i in params.lower()..=params.upper() {
                    *the_param.change_value(i + first - 1)? = params[i];
                }
            }
        }
        Ok(the_param)
    }

    // "Case where no additional knots are desired": [0, 1] or the given knots.
    fn perform_fixed_knots<L: LineTool + Clone>(
        &mut self,
        line: &L,
        range: [i32; 2],
        the_param: &Vector<'static>,
    ) -> Result<(), Failure> {
        let (knots, mut mults) = match &self.knots {
            Some(knots) if self.has_knots => (knots.clone(), self.user_mults(knots)?),
            _ => {
                let mut knots: Array1<'static, f64> = scratch(2)?;
                (knots[1], knots[2]) = (0.0, 1.0);
                (knots, scratch(2)?)
            }
        };
        self.alldone = self.compute(line, range, the_param, &knots, &mut mults)?;
        Ok(())
    }

    // The given multiplicities, or an array Compute fills from the degree and the
    // continuity (`1..=Length()` of the knots).
    fn user_mults(&self, knots: &Array1<'static, f64>) -> Result<Array1<'static, i32>, Failure> {
        match &self.mults {
            Some(mults) if self.has_mults => Ok(mults.clone()),
            _ => scratch(knots.length()),
        }
    }

    // The iteration from the user knots, then from 2, 3... uniformly spread knots.
    fn perform_adding_knots<L: LineTool + Clone>(
        &mut self,
        line: &L,
        range: [i32; 2],
        the_param: &Vector<'static>,
    ) -> Result<(), Failure> {
        let mut begin: bool = true;
        let mut nbknots: i32 = 2;
        loop {
            (self.currenttol3d, self.currenttol2d) = (real_last(), real_last());
            if let (Some(knots), true) = (self.knots.clone(), self.has_knots && begin) {
                let mut mults: Array1<'static, i32> = self.user_mults(&knots)?;
                self.alldone = self.compute(line, range, the_param, &knots, &mut mults)?;
                begin = false;
            } else {
                let knots: Array1<'static, f64> = spread_knots(the_param, range, nbknots)?;
                let mut mults: Array1<'static, i32> = scratch(nbknots)?;
                self.alldone = self.compute(line, range, the_param, &knots, &mut mults)?;
            }
            if self.alldone {
                return Ok(());
            }
            nbknots += 1;
        }
    }

    /// `Parameters(Line, firstP, LastP, TheParameters)`: 0 at the first point (never
    /// scaled, so one point cannot divide by zero), 1 at the second of two points,
    /// else the chord-length or centripetal parameters scaled by the last one, or
    /// `(i - firstP) / (lastP - firstP)`.
    pub(super) fn compute_parameters<L: LineTool>(
        &self,
        line: &L,
        first_p: i32,
        last_p: i32,
        parameters: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        *parameters.change_value(first_p)? = 0.0;
        if last_p - first_p + 1 == 2 {
            *parameters.change_value(last_p)? = 1.0;
            return Ok(());
        }
        if self.par == ParametrizationType::IsoParametric {
            for i in first_p + 1..=last_p {
                *parameters.change_value(i)? =
                    (f64::from(i) - f64::from(first_p)) / f64::from(last_p - first_p);
            }
            return Ok(());
        }
        let centripetal: bool = self.par == ParametrizationType::Centripetal;
        cumulated_chords(line, [first_p, last_p], centripetal, parameters)?;
        for i in first_p + 1..=last_p {
            *parameters.change_value(i)? /= *parameters.value(last_p)?;
        }
        Ok(())
    }

    /// `FindRealConstraints(Line)`: a tangency (curvature) constraint stays only when
    /// the line has tangents at that end; the curvature test asks for the tangents
    /// again, not for the curvatures (native quirk).
    fn find_real_constraints<L: LineTool>(&mut self, line: &L) -> Result<(), Failure> {
        self.real_first_c = self.first_c;
        self.real_last_c = self.last_c;
        let mut tab_v: Array1<'static, Vector3> = scratch(line.nb_p3d()?)?;
        let mut tab_v2d: Array1<'static, Vector2> = scratch(line.nb_p2d()?)?;
        if self.first_c >= Constraint::TangencyPoint {
            let first: i32 = line.first_point();
            self.real_first_c =
                real_constraint(line, first, self.first_c, &mut tab_v, &mut tab_v2d)?;
        }
        if self.last_c >= Constraint::TangencyPoint {
            let last: i32 = line.last_point()?;
            self.real_last_c = real_constraint(line, last, self.last_c, &mut tab_v, &mut tab_v2d)?;
        }
        Ok(())
    }
}

// The constraint an end can hold: PassPoint without tangents, else the tangency,
// or the curvature when asked for and the (second) tangent query succeeds.
fn real_constraint<L: LineTool>(
    line: &L,
    index: i32,
    wanted: Constraint,
    tab_v: &mut Array1<'static, Vector3>,
    tab_v2d: &mut Array1<'static, Vector2>,
) -> Result<Constraint, Failure> {
    if !line_tangency(line, index, tab_v, tab_v2d)? {
        return Ok(Constraint::PassPoint);
    }
    if wanted == Constraint::CurvaturePoint && line_tangency(line, index, tab_v, tab_v2d)? {
        return Ok(Constraint::CurvaturePoint);
    }
    Ok(Constraint::TangencyPoint)
}

// The knots of the third case: 0, 1 and `nbknots - 2` knots interpolated in TheParam.
fn spread_knots(
    the_param: &Vector<'static>,
    [first, last]: [i32; 2],
    nbknots: i32,
) -> Result<Array1<'static, f64>, Failure> {
    let mut knots: Array1<'static, f64> = scratch(nbknots)?;
    knots[1] = 0.0;
    knots[nbknots] = 1.0;
    for i in 2..=nbknots - 1 {
        let l: f64 = f64::from(last - first) * f64::from(i - 1) / f64::from(nbknots - 1);
        let ll: i32 = l as i32;
        let a: f64 = l - f64::from(ll);
        let p1: f64 = *the_param.value(ll + first)?;
        let p2: f64 = *the_param.value(ll + 1 + first)?;
        knots[i] = (1. - a) * p1 + a * p2;
    }
    Ok(knots)
}
