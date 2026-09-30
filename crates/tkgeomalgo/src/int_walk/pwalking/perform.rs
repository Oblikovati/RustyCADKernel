// Rust translation of Perform and PerformFirstPoint of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform`: the first solution point from the start parameters, then the marching
//! loop ([`MarchState`]) until the line arrives.
use super::IntWalkPWalking;
use super::minimize::is_tangent_ext_check;
use crate::int_imp::IntImpConstIsoparametric;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, real};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::adaptor3d::h_surface_tool as tool;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::FunctionSetRoot;

/// The locals of the native marching loop, which persist across its iterations.
pub(super) struct MarchState {
    pub(super) param: Array1<'static, f64>,
    pub(super) sv_param: [f64; 4],
    pub(super) dp: [f64; 4],
    pub(super) choix_iso: IntImpConstIsoparametric,
    pub(super) rsnld: FunctionSetRoot,
    pub(super) arrive: bool,
    pub(super) deja_reparti: bool,
    pub(super) inc_key: i32,
    pub(super) reject_index: i32,
    pub(super) pf: Point3,
    pub(super) b_test_first_point: bool,
    pub(super) status: Status,
    pub(super) prev_status: Status,
    pub(super) no_test_deflection: bool,
    pub(super) level_of_empty: i32,
    pub(super) level_of_point_confondu: i32,
    pub(super) level_of_iter_without_append: i32,
    pub(super) nb_pas_ok_conseq: i32,
    /// `Epsilon` of each walking range: the snapping tolerance of new points.
    pub(super) tol: [f64; 4],
    pub(super) prev_nb_points: i32,
    /// `UFirst1, VFirst1, UFirst2, VFirst2` and the last parameters of the surfaces.
    pub(super) natural: ([f64; 4], [f64; 4]),
}

/// What the marching loop does after one iteration.
pub(super) enum Flow {
    /// The next iteration (the native `continue`, or the end of the body).
    Next,
    /// The native `break` out of the loop.
    Stop,
    /// The native `return` from Perform: the line stays not done.
    Abort,
}

/// Reads `ParDep(1..4)` (Standard_OutOfRange when the array does not hold them).
pub(super) fn read_four(par: &Array1<'_, f64>) -> Result<[f64; 4], Failure> {
    let mut values: [f64; 4] = [0.0; 4];
    for (i, value) in values.iter_mut().enumerate() {
        *value = *par.value(i as i32 + 1).map_err(errors::array_bounds)?;
    }
    Ok(values)
}

impl IntWalkPWalking {
    /// `PerformFirstPoint(ParDep, FirstPoint)`: only the first solution point near
    /// `par_dep` (u1 v1 u2 v2, 1-based); None when there is none.
    /// `let first = walk.perform_first_point(&start)?;`.
    pub fn perform_first_point(
        &mut self,
        par_dep: &Array1<'_, f64>,
    ) -> Result<Option<IntSurfPntOn2S>, Failure> {
        self.sens_cheminement = 1;
        self.close = false;
        let mut param: Array1<'static, f64> = one_based(4)?;
        let values: [f64; 4] = read_four(par_dep)?;
        (param[1], param[2], param[3], param[4]) = (values[0], values[1], values[2], values[3]);
        let mut rsnld =
            FunctionSetRoot::without_tolerance(self.my_intersection_on2s.function(), 100)?;
        self.my_intersection_on2s.perform(&param, &mut rsnld)?;
        if !self.my_intersection_on2s.is_done() || self.my_intersection_on2s.is_empty()? {
            return Ok(None);
        }
        Ok(Some(*self.my_intersection_on2s.point()?))
    }
    /// `Perform(ParDep)`: the line through the point near `par_dep`, with the steps
    /// estimated from the walking bounds. `walk.perform(&start)?`.
    pub fn perform(&mut self, par_dep: &Array1<'_, f64>) -> Result<(), Failure> {
        let (lower, upper) = (self.lower, self.upper);
        self.perform_bounded(par_dep, lower, upper)
    }
    /// `Perform(ParDep, u1min, v1min, u2min, v2min, u1max, v1max, u2max, v2max)`: the
    /// steps are estimated from the parameter box `[min, max]` (u1 v1 u2 v2).
    /// `walk.perform_bounded(&start, [0.0; 4], [1.0; 4])?`.
    pub fn perform_bounded(
        &mut self,
        par_dep: &Array1<'_, f64>,
        min: [f64; 4],
        max: [f64; 4],
    ) -> Result<(), Failure> {
        self.done = false;
        let (caro1, caro2) = self.surfaces();
        let first: [f64; 4] = [
            tool::first_u_parameter(&caro1)?,
            tool::first_v_parameter(&caro1)?,
            tool::first_u_parameter(&caro2)?,
            tool::first_v_parameter(&caro2)?,
        ];
        let last: [f64; 4] = [
            tool::last_u_parameter(&caro1)?,
            tool::last_v_parameter(&caro1)?,
            tool::last_u_parameter(&caro2)?,
            tool::last_v_parameter(&caro2)?,
        ];
        self.compute_pas_init([0, 1, 2, 3].map(|i| max[i] - min[i]))?;
        for i in 0..4 {
            if self.pasuv[i] > 10. {
                self.pasuv[i] = 10.;
            }
            (self.pas_init[i], self.pas_sav[i]) = (self.pasuv[i], self.pasuv[i]);
        }
        self.line = Handle::new(IntSurfLineOn2S::new(None));
        let Some(mut st) = self.first_point(par_dep, (first, last))? else {
            return Ok(());
        };
        while !st.arrive {
            match self.march_step(&mut st)? {
                Flow::Next => {}
                Flow::Stop => break,
                Flow::Abort => return Ok(()),
            }
        }
        self.done = true;
        Ok(())
    }
    /// The first solution point: the marching state, or None (the native returns)
    /// when there is no point, a tangent one, or tangent surfaces around it.
    fn first_point(
        &mut self,
        par_dep: &Array1<'_, f64>,
        natural: ([f64; 4], [f64; 4]),
    ) -> Result<Option<MarchState>, Failure> {
        let mut param: Array1<'static, f64> = one_based(4)?;
        let values: [f64; 4] = read_four(par_dep)?;
        (param[1], param[2], param[3], param[4]) = (values[0], values[1], values[2], values[3]);
        // The steps are modified during the marching.
        (self.pas_sav, self.pasuv) = (self.pas_init, self.pas_init);
        let mut rsnld =
            FunctionSetRoot::without_tolerance(self.my_intersection_on2s.function(), 100)?;
        let choix_iso: IntImpConstIsoparametric =
            self.my_intersection_on2s.perform(&param, &mut rsnld)?;
        let inter = &self.my_intersection_on2s;
        if !inter.is_done() || inter.is_empty()? || inter.is_tangent()? {
            return Ok(None);
        }
        self.start_on_first_point(choix_iso)?;
        let (u1, v1, u2, v2) = self.previous_point.parameters();
        (param[1], param[2], param[3], param[4]) = (u1, v1, u2, v2);
        let (caro1, caro2) = self.surfaces();
        if is_tangent_ext_check(
            &caro1,
            &caro2,
            [u1, v1, u2, v2],
            self.my_tol_tang,
            self.pasuv,
        )? {
            return Ok(None);
        }
        let first_point: IntSurfPntOn2S = self.previous_point;
        self.add_a_point(&first_point)?;
        let widths: [f64; 4] = [0, 1, 2, 3].map(|i| real::epsilon(self.upper[i] - self.lower[i]));
        Ok(Some(MarchState {
            param,
            sv_param: [0.0; 4],
            dp: [0.0; 4],
            choix_iso,
            rsnld,
            arrive: false,
            deja_reparti: false,
            inc_key: 0,
            reject_index: 0,
            pf: first_point.value(),
            b_test_first_point: true,
            status: Status::OK,
            prev_status: Status::OK,
            no_test_deflection: false,
            level_of_empty: 0,
            level_of_point_confondu: 0,
            level_of_iter_without_append: -1,
            nb_pas_ok_conseq: 0,
            tol: widths,
            prev_nb_points: self.line_ref()?.nb_points(),
            natural,
        }))
    }
    /// The first point becomes the previous point, with its tangents kept to restart
    /// in the other direction.
    fn start_on_first_point(&mut self, choix_iso: IntImpConstIsoparametric) -> Result<(), Failure> {
        let inter = &self.my_intersection_on2s;
        self.previous_point = *inter.point()?;
        self.previoustg = false;
        self.previousd = inter.direction()?;
        self.previousd1 = inter.direction_on_s1()?;
        self.previousd2 = inter.direction_on_s2()?;
        self.my_tangent_idx = 1;
        self.tgdir = self.previousd;
        (self.firstd1, self.firstd2) = (self.previousd1, self.previousd2);
        (self.tgfirst, self.tglast) = (false, false);
        self.choix_iso_sav = choix_iso;
        Ok(())
    }
}
