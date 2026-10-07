// Rust translation of BRepBlend_Walking::PerformFirstSection(F, FInv, Pdep, Pmax, ParDep,
// Tol3d, TolGuide, RecOnS1, RecOnS2, Psol, ParSol) (BRepBlend_Walking.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The first section of a walking on the restrictions.
use super::extremities::CorrectedEnd;
use super::recadre::Reframed;
use super::{BRepBlendWalking, Sol, live, sol_of};
use crate::blend::{BlendFuncInv, BlendFunction, Status};
use crate::brep_blend::{BRepBlendExtremity, BRepBlendHCurve2dTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::Vector;

/// The reframings of the first section on both restrictions.
type FirstReframings = (Option<Reframed>, Option<Reframed>);

impl BRepBlendWalking {
    /// `PerformFirstSection(F, FInv, Pdep, Pmax, ParDep, Tol3d, TolGuide, RecOnS1,
    /// RecOnS2, Psol, ParSol)`: the first section on the restrictions (on the first one
    /// when `rec_on_s1`, on the second when `rec_on_s2`): its guide parameter and its
    /// solution, or `None`. Standard_Failure when it is not a section.
    pub fn perform_first_section_on_restrictions(
        &mut self,
        func: &mut dyn BlendFunction,
        func_inv: &mut dyn BlendFuncInv,
        (pdep, pmax): (f64, f64),
        par_dep: &Vector<'_>,
        tols: (f64, f64),
        rec_on: (bool, bool),
    ) -> Result<Option<(f64, Sol)>, Failure> {
        self.start_first_section(tols);
        self.sens = if pmax - pdep >= 0.0 { 1.0 } else { -1.0 };
        let extrapol = (pmax - pdep).abs() / 50.0; // 2%
        self.param = pdep;
        func.set_param(self.param)?;
        if self.solve_first(func, &sol_of(par_dep)?)?.is_none() {
            return Ok(None);
        }
        let recs = self.first_reframings(func_inv, rec_on, extrapol)?;
        let (rec1, rec2) = (recs.0.as_ref(), recs.1.as_ref());
        if rec1.is_none() && rec2.is_none() {
            return Ok(None);
        }
        let (state, par_sol, corrected) = self.first_section_on(pmax, rec1, rec2)?;
        let psol = self.param;
        self.sol = par_sol;
        func.set_param(self.param)?;
        let state = self.test_arret(func, state, false, true, false)?;
        self.first_section_ends(state, (rec1, rec2), &corrected)?;
        Ok(Some((psol, par_sol)))
    }

    /// The reframings of the solution on the restrictions asked for, the guide bounds
    /// widened by `extrapol`.
    fn first_reframings(
        &mut self,
        func_inv: &mut dyn BlendFuncInv,
        (rec_on_s1, rec_on_s2): (bool, bool),
        extrapol: f64,
    ) -> Result<FirstReframings, Failure> {
        let sol = self.sol;
        let rec1 = if rec_on_s1 {
            self.recadre(func_inv, true, &sol, extrapol)?
        } else {
            None
        };
        let rec2 = if rec_on_s2 {
            self.recadre(func_inv, false, &sol, extrapol)?
        } else {
            None
        };
        Ok((rec1, rec2))
    }

    /// The status, the solution and the corrected end of the first section on the
    /// restrictions; `param` gets its guide parameter.
    fn first_section_on(
        &mut self,
        pmax: f64,
        rec1: Option<&Reframed>,
        rec2: Option<&Reframed>,
    ) -> Result<(Status, Sol, CorrectedEnd), Failure> {
        let mut corrected = CorrectedEnd {
            u: 0.0,
            v: 0.0,
            point: Point3::default(),
        };
        let w1 = rec1.map_or(pmax, |r| r.solrst[1]);
        let w2 = rec2.map_or(pmax, |r| r.solrst[1]);
        let (state, par_sol) = match (rec1, rec2) {
            (Some(r1), Some(r2)) => self.first_on_both(r1, r2, (w1, w2))?,
            (Some(r1), None) => self.first_on_one(true, r1, &mut corrected)?,
            (None, Some(r2)) => self.first_on_one(false, r2, &mut corrected)?,
            (None, None) => return Err(no_reframing()),
        };
        Ok((state, par_sol, corrected))
    }

    /// Both reframings: the section on both restrictions when they meet the guide at the
    /// same parameter, else on the larger one ("on garde le plus grand").
    fn first_on_both(
        &mut self,
        r1: &Reframed,
        r2: &Reframed,
        (w1, w2): (f64, f64),
    ) -> Result<(Status, Sol), Failure> {
        if (w1 - w2).abs() <= self.tolgui {
            // sol sur 1 et 2 a la fois
            self.param = w1;
            return Ok((
                Status::OnRst12,
                [r2.solrst[2], r2.solrst[3], r1.solrst[2], r1.solrst[3]],
            ));
        }
        if self.sens * (w2 - w1) < 0.0 {
            // sol sur 1
            self.param = w1;
            return Ok((Status::OnRst1, self.on_rst_sol(true, r1)?));
        }
        // sol sur 2
        self.param = w2;
        Ok((Status::OnRst2, self.on_rst_sol(false, r2)?))
    }

    /// One reframing: the section on its restriction, with its other end corrected.
    fn first_on_one(
        &mut self,
        on_first: bool,
        r: &Reframed,
        corrected: &mut CorrectedEnd,
    ) -> Result<(Status, Sol), Failure> {
        self.param = r.solrst[1];
        let par_sol = self.on_rst_sol(on_first, r)?;
        self.correct_first_end(on_first, &par_sol, corrected)?;
        Ok((
            if on_first {
                Status::OnRst1
            } else {
                Status::OnRst2
            },
            par_sol,
        ))
    }

    /// The solution of a section on the restriction of `on_first`: the point of the arc
    /// at the parameter of the reframing, and the parameters on the other surface.
    fn on_rst_sol(&self, on_first: bool, r: &Reframed) -> Result<Sol, Failure> {
        let arc = self.nth_arc(on_first, r.index)?;
        let p2d = BRepBlendHCurve2dTool::value(&arc, r.solrst[0])?;
        Ok(if on_first {
            [p2d.x(), p2d.y(), r.solrst[2], r.solrst[3]]
        } else {
            [r.solrst[2], r.solrst[3], p2d.x(), p2d.y()]
        })
    }

    /// CorrectExtremityOnOneRst on the first section: `CorrectedParam` is the member, the
    /// flag `ToCorrectOnRst*` is set (and never reset) on success.
    fn correct_first_end(
        &mut self,
        on_first: bool,
        par_sol: &Sol,
        corrected: &mut CorrectedEnd,
    ) -> Result<(), Failure> {
        let (pnt_uv, other_uv) = if on_first {
            ((par_sol[0], par_sol[1]), (par_sol[2], par_sol[3]))
        } else {
            ((par_sol[2], par_sol[3]), (par_sol[0], par_sol[1]))
        };
        let the_pnt_on_rst = live(self.surface(on_first))?.value(pnt_uv.0, pnt_uv.1)?;
        let rst = if on_first { 1 } else { 2 };
        let correction =
            self.correct_extremity_on_one_rst(rst, other_uv, self.param, the_pnt_on_rst)?;
        if let Some(param) = correction.new_param {
            self.corrected_param = param;
        }
        let Some(end) = correction.end else {
            return Ok(());
        };
        *corrected = end;
        if on_first {
            self.to_correct_on_rst1 = true;
        } else {
            self.to_correct_on_rst2 = true;
        }
        Ok(())
    }

    /// The extremities of the first section on the restrictions, by its status.
    fn first_section_ends(
        &mut self,
        state: Status,
        (rec1, rec2): (Option<&Reframed>, Option<&Reframed>),
        corrected: &CorrectedEnd,
    ) -> Result<(), Failure> {
        let (mut ext1, mut ext2) = (BRepBlendExtremity::new(), BRepBlendExtremity::new());
        match (state, rec1, rec2) {
            (Status::OnRst1, Some(r1), _) => {
                self.make_extremity(&mut ext1, true, r1.index, r1.solrst[0], r1.vertex.as_ref())?;
                ext2 = self.free_first_end(false, corrected);
            }
            (Status::OnRst2, _, Some(r2)) => {
                ext1 = self.free_first_end(true, corrected);
                self.make_extremity(&mut ext2, false, r2.index, r2.solrst[0], r2.vertex.as_ref())?;
            }
            (Status::OnRst12, Some(r1), Some(r2)) => {
                self.make_extremity(&mut ext1, true, r1.index, r1.solrst[0], r1.vertex.as_ref())?;
                self.make_extremity(&mut ext2, false, r2.index, r2.solrst[0], r2.vertex.as_ref())?;
            }
            _ => return Err(first_section_failure(state)),
        }
        self.set_line_ends(ext1, ext2, self.sens >= 0.0)
    }

    /// The end of the first section on the surface without restriction (`on_first`):
    /// the corrected point when the flag of the other restriction is set (even by an
    /// earlier call), else the previous point. The 4-argument `SetValue` is the curve one
    /// `(P, W, Param, Tol)`.
    fn free_first_end(&self, on_first: bool, corrected: &CorrectedEnd) -> BRepBlendExtremity {
        let flag = if on_first {
            self.to_correct_on_rst2
        } else {
            self.to_correct_on_rst1
        };
        let (p, u, v) = match (flag, on_first) {
            (true, _) => (corrected.point, corrected.u, corrected.v),
            (false, true) => (self.previous_p.point_on_s1(), self.sol[0], self.sol[1]),
            (false, false) => (self.previous_p.point_on_s2(), self.sol[2], self.sol[3]),
        };
        let mut ext = BRepBlendExtremity::new();
        ext.set_value_on_curve(p, u, v, self.tolpoint3d);
        ext
    }
}

/// The case without reframing, excluded by the caller.
fn no_reframing() -> Failure {
    Failure::with_message(
        "Standard_Failure",
        "BRepBlend_Walking::PerformFirstSection: no reframing; expected one on a restriction",
    )
}

/// The Standard_Failure of a first section that is not on a restriction.
fn first_section_failure(state: Status) -> Failure {
    Failure::with_message(
        "Standard_Failure",
        &format!(
            "BRepBlend_Walking::PerformFirstSection : echec (status {state:?}; expected a section on a restriction)"
        ),
    )
}
