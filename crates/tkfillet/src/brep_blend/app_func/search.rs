//! `SearchPoint` and `SearchLocation` of `BRepBlend_AppFuncRoot`: the solution of the
//! blend function at a parameter of the line, started from the computed points.
use super::{AppFuncState, BRepBlendAppFuncRoot, DynFunctionSet, LineKind};
use crate::blend::{BlendAppFunction, BlendPoint};
use crate::blend_func::linalg::{vget, vset};
use crate::brep_blend::BRepBlendLine;
use crate::chfi_ds::lock;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;

impl LineKind {
    /// `Vec(Sol, Pnt)`: the variables of the function at a line point.
    pub(crate) fn vec(self, sol: &mut Vector<'_>, pnt: &BlendPoint) -> Result<(), Failure> {
        let values: Vec<f64> = match self {
            LineKind::Surfaces => {
                let ((u1, v1), (u2, v2)) = (pnt.parameters_on_s1()?, pnt.parameters_on_s2()?);
                vec![u1, v1, u2, v2]
            }
            LineKind::SurfaceRestriction => {
                let (u, v) = pnt.parameters_on_s()?;
                vec![u, v, pnt.parameter_on_c()?]
            }
            LineKind::Restrictions => vec![pnt.parameter_on_c1()?, pnt.parameter_on_c2()?],
        };
        for (i, value) in (1..).zip(values) {
            vset(sol, i, value)?;
        }
        Ok(())
    }

    /// `Point(Func, Param, Sol, Pnt)`: the line point of a solution.
    pub(crate) fn point(
        self,
        func: &dyn BlendAppFunction,
        param: f64,
        sol: &Vector<'_>,
        pnt: &mut BlendPoint,
    ) -> Result<(), Failure> {
        let (pt1, pt2) = (func.pnt1(), func.pnt2());
        match self {
            LineKind::Surfaces => pnt.set_on_surfaces(
                pt1,
                pt2,
                param,
                [vget(sol, 1)?, vget(sol, 2)?, vget(sol, 3)?, vget(sol, 4)?],
            ),
            LineKind::SurfaceRestriction => pnt.set_on_surface_curve(
                pt1,
                pt2,
                param,
                [vget(sol, 1)?, vget(sol, 2)?, vget(sol, 3)?],
            ),
            LineKind::Restrictions => {
                pnt.set_on_curves(pt1, pt2, param, vget(sol, 1)?, vget(sol, 2)?)
            }
        }
        Ok(())
    }
}

/// `SearchLocation(Param, FirstIndex, LastIndex, ParamIndex)`: the binary search of the
/// interval of the line containing `param`; `(true, index)` when a point has exactly that
/// parameter, else `(false, index of the lower bound)`.
pub(crate) fn search_location(
    line: &BRepBlendLine,
    param: f64,
    (first, last): (i32, i32),
) -> Result<(bool, i32), Failure> {
    let (mut ideb, mut ifin) = (first, last);
    if param == line.point(ideb)?.parameter() {
        return Ok((true, ideb));
    }
    if param == line.point(ifin)?.parameter() {
        return Ok((true, ifin));
    }
    while ideb + 1 != ifin {
        let idemi = (ideb + ifin) / 2;
        let valeur = line.point(idemi)?.parameter();
        if valeur < param {
            ideb = idemi;
        } else if valeur > param {
            ifin = idemi;
        } else {
            return Ok((true, idemi));
        }
    }
    Ok((false, ideb))
}

impl BRepBlendAppFuncRoot {
    /// `SearchPoint(Func, Param, Pnt)`: the line point at `param` (already computed, or
    /// interpolated from its neighbours and refined by the solver, then inserted in the
    /// line when the solve took more than 3 iterations); false outside the line or when
    /// the solver fails. The point is kept in `myPnt`.
    pub(crate) fn search_point(
        &self,
        func: &mut dyn BlendAppFunction,
        param: f64,
        state: &mut AppFuncState,
    ) -> Result<bool, Failure> {
        let (trouve, index) = {
            let line = lock(&self.line);
            let (i1, i2) = (1, line.nb_points());
            if param < line.point(i1)?.parameter() || param > line.point(i2)?.parameter() {
                return Ok(false);
            }
            let (trouve, index) = search_location(&line, param, (i1, i2))?;
            self.initial_point(&line, param, (trouve, index), state)?;
            (trouve, index)
        };
        func.set_param(param)?;
        func.get_bounds(&mut state.x1, &mut state.x2)?;
        let AppFuncState {
            solver,
            xinit,
            x1,
            x2,
            sol,
            pnt,
            ..
        } = state;
        solver.perform_bounded(&mut DynFunctionSet(&mut *func), xinit, x1, x2, false)?;
        func.take_failure()?;
        if !solver.is_done() {
            return Ok(false);
        }
        solver.copy_root(sol)?;
        self.kind.point(&*func, param, sol, pnt)?;
        if !trouve && solver.iterations()? > 3 {
            lock(&self.line).insert_before(index + 1, *pnt)?;
        }
        Ok(true)
    }

    /// `XInit`: the variables of the point found, or their linear interpolation between
    /// the points around `param`.
    fn initial_point(
        &self,
        line: &BRepBlendLine,
        param: f64,
        (trouve, index): (bool, i32),
        state: &mut AppFuncState,
    ) -> Result<(), Failure> {
        if trouve {
            state.pnt = *line.point(index)?;
            return self.kind.vec(&mut state.xinit, &state.pnt);
        }
        state.pnt = *line.point(index)?;
        self.kind.vec(&mut state.x1, &state.pnt)?;
        let t1 = state.pnt.parameter();
        state.pnt = *line.point(index + 1)?;
        self.kind.vec(&mut state.x2, &state.pnt)?;
        let t2 = state.pnt.parameter();
        let parammt1 = (param - t1) / (t2 - t1);
        let t2m_param = (t2 - param) / (t2 - t1);
        for i in 1..=state.xinit.length() {
            vset(
                &mut state.xinit,
                i,
                vget(&state.x2, i)? * parammt1 + vget(&state.x1, i)? * t2m_param,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_of(parameters: &[f64]) -> BRepBlendLine {
        let mut line = BRepBlendLine::new();
        for t in parameters {
            let mut p = BlendPoint::default();
            p.set_parameter(*t);
            line.append(p);
        }
        line
    }

    #[test]
    fn the_location_is_the_lower_point_or_the_exact_one() {
        let line = line_of(&[0., 1., 2., 3.]);
        assert_eq!(search_location(&line, 1.5, (1, 4)).unwrap(), (false, 2));
        assert_eq!(search_location(&line, 2., (1, 4)).unwrap(), (true, 3));
        assert_eq!(search_location(&line, 0., (1, 4)).unwrap(), (true, 1));
        assert_eq!(search_location(&line, 3., (1, 4)).unwrap(), (true, 4));
    }

    #[test]
    fn the_variables_follow_the_line_kind() {
        let mut p = BlendPoint::default();
        p.set_on_curves(Default::default(), Default::default(), 1., 0.25, 0.75);
        let mut sol = crate::blend_func::linalg::vector(2).unwrap();
        LineKind::Restrictions.vec(&mut sol, &p).unwrap();
        assert_eq!(
            (vget(&sol, 1).unwrap(), vget(&sol, 2).unwrap()),
            (0.25, 0.75)
        );
        assert!(LineKind::Surfaces.vec(&mut sol, &p).is_err());
    }
}
