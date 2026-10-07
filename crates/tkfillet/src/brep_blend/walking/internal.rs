// Rust translation of BRepBlend_Walking::InternalPerform and evalpinit
// (BRepBlend_Walking.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The marching loop: the steps along the guide, each one solved, classified on the
//! domains, reframed on the restrictions it crosses, and tested.
use super::first_section::SolveSetup;
use super::recadre::Reframed;
use super::{BRepBlendWalking, Sol, live, live_guide, root_of, sol_of, solver_for};
use crate::blend::{BlendFuncInv, BlendFunction, BlendPoint, Status};
use crate::brep_blend::{BRepBlendExtremity, BRepBlendHCurve2dTool};
use crate::chfi_ds::lock;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeombase::gce::{MakePln, Root};
use rustycad_tkmath::gp::{self, Point2, Point3, Vector3};
use rustycad_tkmath::math::FunctionSetRoot;

/// `CosRef3D` of InternalPerform: the largest angle of the guide tangent in one step.
const COS_REF_3D: f64 = 0.88;
/// `TolProd` of the guide vertex check on both restrictions.
const TOL_PROD: f64 = 1.0e-5;

/// The reframings of a step: on which restriction, at which guide parameter, whether to
/// control them, and whether one failed (`recad1`, `recad2`, `w1`, `w2`, `control`,
/// `echecrecad`).
#[derive(Default)]
struct Reframings {
    recad: [bool; 2],
    w: [f64; 2],
    control: bool,
    echec: bool,
}

/// The state of the marching loop (the locals of InternalPerform).
pub(super) struct March {
    pub(super) setup: SolveSetup,
    pub(super) solver: FunctionSetRoot,
    pub(super) bounds: (Sol, Sol),
    pub(super) parinit: Sol,
    pub(super) stepw: f64,
    pub(super) parprec: f64,
    pub(super) prev_tg_on_guide: Vector3,
    pub(super) situ: (State, State),
    pub(super) rec1: Option<Reframed>,
    pub(super) rec2: Option<Reframed>,
    pub(super) ext1: BRepBlendExtremity,
    pub(super) ext2: BRepBlendExtremity,
    pub(super) arrive: bool,
}

impl BRepBlendWalking {
    /// `InternalPerform(F, FInv, Bound)`: the walking from the current point up to the
    /// guide parameter `bound`, the restrictions, or a stop.
    pub(super) fn internal_perform(
        &mut self,
        func: &mut dyn BlendFunction,
        func_inv: &mut dyn BlendFuncInv,
        bound: f64,
    ) -> Result<(), Failure> {
        let stepw = self.first_step()?;
        let parprec = self.param;
        let (_, tg_on_guide) = live_guide(&self.hguide)?.d1(parprec)?;
        if self.sens * (parprec - bound) >= -self.tolgui {
            return Ok(());
        }
        let setup = self.solve_setup(func)?;
        let solver = solver_for(func, &setup.tolerance, 30)?;
        let bounds = (sol_of(&setup.inf)?, sol_of(&setup.sup)?);
        let mut march = March {
            setup,
            solver,
            bounds,
            parinit: self.sol,
            stepw,
            parprec,
            prev_tg_on_guide: tg_on_guide,
            situ: (State::In, State::In),
            rec1: None,
            rec2: None,
            ext1: BRepBlendExtremity::new(),
            ext2: BRepBlendExtremity::new(),
            arrive: false,
        };
        self.param = parprec + self.sens * march.stepw;
        if self.sens * (self.param - bound) > 0.0 {
            march.stepw = self.sens * (bound - parprec) * 0.5;
            self.param = parprec + self.sens * march.stepw;
        }
        self.evalpinit(&mut march)?;
        while !march.arrive {
            self.march_step(func, func_inv, bound, &mut march)?;
        }
        Ok(())
    }

    /// The first step: the last one of the line when it is not too small, else the
    /// largest step.
    fn first_step(&self) -> Result<f64, Failure> {
        let line = lock(self.line_handle()?);
        let nbp = line.nb_points();
        if nbp < 2 {
            return Ok(self.pasmax);
        }
        // On reprend le dernier step s'il n est pas trop petit.
        let stepw = if self.sens < 0.0 {
            line.point(2)?.parameter() - line.point(1)?.parameter()
        } else {
            line.point(nbp)?.parameter() - line.point(nbp - 1)?.parameter()
        };
        Ok(std_max(stepw, 100.0 * self.tolgui))
    }

    /// One pass of the loop: the guide deflection check, the solve, the status, and its
    /// consequences on the line.
    fn march_step(
        &mut self,
        func: &mut dyn BlendFunction,
        func_inv: &mut dyn BlendFuncInv,
        bound: f64,
        march: &mut March,
    ) -> Result<(), Failure> {
        let (_, tg_on_guide) = live_guide(&self.hguide)?.d1(self.param)?;
        // Check deflection on guide
        let cosi = march.prev_tg_on_guide.dot(tg_on_guide);
        let cosi2 = if cosi < gp::resolution() {
            0.0 // angle >= pi/2 or null magnitude
        } else {
            cosi * cosi / march.prev_tg_on_guide.square_magnitude() / tg_on_guide.square_magnitude()
        };
        if cosi2 < COS_REF_3D {
            // angle 3d too great; the line ends are not set on this exit
            march.stepw /= 2.0;
            self.param = march.parprec + self.sens * march.stepw;
            if march.stepw.abs() < self.tolgui {
                (march.ext1, march.ext2) = self.previous_ends()?;
                march.arrive = true;
            }
            return Ok(());
        }
        march.prev_tg_on_guide = tg_on_guide;
        let mut bonpoint = self.solve_step(func, march)?;
        let mut state = Status::StepTooLarge;
        if bonpoint {
            let one_point = lock(self.line_handle()?).nb_points() == 1;
            if one_point && (march.situ.0 != State::In || march.situ.1 != State::In) {
                bonpoint = false;
            }
        }
        if bonpoint {
            state = self.step_status(func, func_inv, march)?;
        }
        self.apply_status(state, bound, march)?;
        if march.arrive {
            let (ext1, ext2) = (march.ext1.clone(), march.ext2.clone());
            self.set_line_ends(ext1, ext2, self.sens <= 0.0)?;
        }
        Ok(())
    }

    /// The solve at `param` from `parinit`, and the classification of the root; false
    /// when the solver fails.
    fn solve_step(
        &mut self,
        func: &mut dyn BlendFunction,
        march: &mut March,
    ) -> Result<bool, Failure> {
        func.set_param(self.param)?;
        let bounds = (&march.setup.inf, &march.setup.sup);
        let root = root_of(&mut march.solver, func, &march.parinit, bounds);
        func.take_failure()?;
        let Some(root) = root? else {
            return Ok(false);
        };
        self.sol = root;
        march.situ = (
            self.situation(true, &march.setup.tolerance)?,
            self.situation(false, &march.setup.tolerance)?,
        );
        Ok(true)
    }

    /// The status of a solved step: the reframings on the restrictions it left, their
    /// control, the section they give, and its test.
    fn step_status(
        &mut self,
        func: &mut dyn BlendFunction,
        func_inv: &mut dyn BlendFuncInv,
        march: &mut March,
    ) -> Result<Status, Failure> {
        let mut r = self.reframe_leaving(func_inv, march)?;
        self.arbitrate(&mut r);
        if r.control {
            self.control_reframings(march, &mut r.recad, &mut r.echec)?;
        }
        if r.recad[0] || r.recad[1] {
            r.echec = false;
        }
        if r.echec {
            // Ou bien le pas max est mal regle. On divise. Sinon echec recadrage: on sort
            // avec PointsConfondus.
            let too_large = march.stepw > 2.0 * self.tolgui;
            return Ok(if too_large {
                Status::StepTooLarge
            } else {
                Status::SamePoints
            });
        }
        let state = self.reframed_section(march, r.recad, r.w)?;
        if r.recad[0] || r.recad[1] {
            func.set_param(self.param)?;
            // Il vaut mieux un pas non orthodoxe que pas de recadrage!! PMN
            let testdefl = march.stepw.abs() > 3.0 * self.tolgui;
            return self.test_arret(func, state, testdefl, false, true);
        }
        self.test_arret(func, state, true, true, false)
    }

    /// The inverse problems on the surfaces the solution left (OUT or ON); a reframing
    /// in the direction of the walking beyond a tolerance means the wrong solution was
    /// taken: the step is halved.
    fn reframe_leaving(
        &mut self,
        func_inv: &mut dyn BlendFuncInv,
        march: &mut March,
    ) -> Result<Reframings, Failure> {
        let mut r = Reframings::default();
        let leaving = [is_out_or_on(march.situ.0), is_out_or_on(march.situ.1)];
        for (k, on_first) in [(0, true), (1, false)] {
            if !leaving[k] {
                continue;
            }
            let sol = self.sol;
            let Some(reframed) = self.recadre(func_inv, on_first, &sol, 0.0)? else {
                r.echec = true;
                continue;
            };
            let wtemp = reframed.solrst[1];
            if (self.param - wtemp) / self.sens >= -10.0 * self.tolgui {
                (r.w[k], r.recad[k], r.control) = (wtemp, true, true);
            } else {
                r.echec = true;
                march.stepw /= 2.0;
            }
            *(if k == 0 {
                &mut march.rec1
            } else {
                &mut march.rec2
            }) = Some(reframed);
        }
        Ok(r)
    }

    /// Que faut il controler: of two reframings far apart on the guide, the first one
    /// met is kept; close ones need no control (it could fail, cf model blend10; the
    /// tolerance is large so that the next walking can set a few sections).
    fn arbitrate(&self, r: &mut Reframings) {
        if !(r.recad[0] && r.recad[1]) {
            return;
        }
        if (r.w[0] - r.w[1]).abs() <= 10.0 * self.tolgui {
            r.control = false;
        } else if self.sens * (r.w[0] - r.w[1]) < 0.0 {
            r.recad[1] = false; // sol sur 1 ?
        } else {
            r.recad[0] = false; // sol sur 2 ?
        }
    }

    /// Controle effectif des recadrages: a reframed section whose end on the other
    /// surface is out of its domain is rejected.
    fn control_reframings(
        &self,
        march: &March,
        recad: &mut [bool; 2],
        echecrecad: &mut bool,
    ) -> Result<(), Failure> {
        let t = sol_of(&march.setup.tolerance)?;
        if recad[0] && self.clason_s2 {
            let s = &march.rec1.as_ref().map_or([0.0; 4], |r| r.solrst);
            let situ =
                self.recdomain2
                    .classify(Point2::new(s[2], s[3]), std_min(t[2], t[3]), true)?;
            if situ == State::Out {
                recad[0] = false;
                *echecrecad = true;
            }
        } else if recad[1] && self.clason_s1 {
            let s = &march.rec2.as_ref().map_or([0.0; 4], |r| r.solrst);
            // The native code takes min(tolerance(1), tolerance(1)).
            let situ =
                self.recdomain1
                    .classify(Point2::new(s[2], s[3]), std_min(t[0], t[0]), true)?;
            if situ == State::Out {
                recad[1] = false;
                *echecrecad = true;
            }
        }
        Ok(())
    }

    /// The section given by the reframings (on both restrictions, on one, or none):
    /// `param` and `sol` set on it, and its status.
    fn reframed_section(
        &mut self,
        march: &March,
        recad: [bool; 2],
        w: [f64; 2],
    ) -> Result<Status, Failure> {
        let solrst = |r: &Option<Reframed>| r.as_ref().map_or([0.0; 4], |r| r.solrst);
        let (s1, s2) = (solrst(&march.rec1), solrst(&march.rec2));
        match recad {
            [true, true] => {
                self.section_on_both(w, s1[0], s2[0])?;
                Ok(Status::OnRst12)
            }
            [true, false] => {
                self.param = w[0];
                self.section_on_one(true, march.rec1.as_ref().map_or(0, |r| r.index), &s1)?;
                Ok(Status::OnRst1)
            }
            [false, true] => {
                self.param = w[1];
                self.section_on_one(false, march.rec2.as_ref().map_or(0, |r| r.index), &s2)?;
                Ok(Status::OnRst2)
            }
            [false, false] => Ok(Status::Ok),
        }
    }

    /// Sol sur 1 et 2 a la fois: the section through the arcs (to avoid problems with
    /// the periodic surfaces), at the mean parameter or at a saved end of the guide when
    /// the section is in the plane normal to the guide there.
    fn section_on_both(&mut self, w: [f64; 2], wrst1: f64, wrst2: f64) -> Result<(), Failure> {
        self.param = (w[0] + w[1]) / 2.0;
        let p2d = BRepBlendHCurve2dTool::value(&self.recdomain1.value()?, wrst1)?;
        (self.sol[0], self.sol[1]) = (p2d.x(), p2d.y());
        let pnt1 = live(&self.surf1)?.value(self.sol[0], self.sol[1])?;
        let p2d = BRepBlendHCurve2dTool::value(&self.recdomain2.value()?, wrst2)?;
        (self.sol[2], self.sol[3]) = (p2d.x(), p2d.y());
        let pnt2 = live(&self.surf2)?.value(self.sol[2], self.sol[3])?;
        let mut the_param = self.saved_guide_end(pnt1, pnt2)?;
        let uv = (self.sol[2], self.sol[3]);
        let correction = self.correct_extremity_on_one_rst(1, uv, self.param, pnt1)?;
        if let Some((new_param, _)) = correction.corrected()
            && (self.param - new_param).abs() < (self.param - the_param).abs()
        {
            the_param = new_param;
        }
        if !precision::is_infinite(the_param) {
            self.param = the_param;
        }
        Ok(())
    }

    /// The original first or last parameter of the guide where the plane through the
    /// guide point and both restriction points is normal to the guide (the closest one
    /// to `param` when both are); Precision::Infinite when none.
    fn saved_guide_end(&self, pnt1: Point3, pnt2: Point3) -> Result<f64, Failure> {
        let guide = live_guide(&self.hguide)?;
        let saved = [guide.saved_first_parameter(), guide.saved_last_parameter()];
        let mut same_dirs = [false; 2];
        for (ind, &saved_param) in saved.iter().enumerate() {
            if precision::is_infinite(saved_param) {
                continue;
            }
            same_dirs[ind] = self.normal_at_guide(saved_param, pnt1, pnt2)?;
        }
        Ok(match same_dirs {
            [true, true] if (self.param - saved[0]).abs() < (self.param - saved[1]).abs() => {
                saved[0]
            }
            [true, true] => saved[1],
            [true, false] => saved[0],
            [false, true] => saved[1],
            [false, false] => precision::infinite(),
        })
    }

    /// Whether the tangent to the guide at `t` is parallel to the normal of the plane
    /// through the guide point and the restriction points.
    fn normal_at_guide(&self, t: f64, pnt1: Point3, pnt2: Point3) -> Result<bool, Failure> {
        let (pnt0, mut dir0) = live_guide(&self.hguide)?.d1(t)?;
        let length = dir0.magnitude();
        if length <= gp::resolution() {
            return Ok(false);
        }
        dir0 /= length;
        let builder = MakePln::through_points(pnt0, pnt1, pnt2)?;
        if !builder.is_done() {
            return Ok(false);
        }
        let dir_plane = Vector3::from(builder.value()?.axis().direction());
        Ok(dir0.crossed(dir_plane).magnitude() <= TOL_PROD)
    }

    /// Sol sur 1 (or sur 2): the section through the arc `index`, its end on the other
    /// surface corrected at a vertex of the guide.
    fn section_on_one(&mut self, on_first: bool, index: i32, solrst: &Sol) -> Result<(), Failure> {
        let arc = self.nth_arc(on_first, index)?;
        let p2d = BRepBlendHCurve2dTool::value(&arc, solrst[0])?;
        let (k, other) = if on_first { (0, 2) } else { (2, 0) };
        (self.sol[k], self.sol[k + 1]) = (p2d.x(), p2d.y());
        (self.sol[other], self.sol[other + 1]) = (solrst[2], solrst[3]);
        let the_pnt_on_rst = live(self.surface(on_first))?.value(p2d.x(), p2d.y())?;
        let uv = (self.sol[other], self.sol[other + 1]);
        let rst = if on_first { 1 } else { 2 };
        let correction = self.correct_extremity_on_one_rst(rst, uv, self.param, the_pnt_on_rst)?;
        if let Some((new_param, end)) = correction.corrected() {
            self.param = new_param;
            (self.sol[other], self.sol[other + 1]) = (end.u, end.v);
        }
        Ok(())
    }

    /// The extremities of a stop at the previous point, with its tangents.
    pub(super) fn previous_ends(
        &self,
    ) -> Result<(BRepBlendExtremity, BRepBlendExtremity), Failure> {
        let previous = &self.previous_p;
        let [u1, v1, u2, v2] = self.sol;
        let (w, tol) = (previous.parameter(), self.tolpoint3d);
        let mut ext1 = BRepBlendExtremity::new();
        let mut ext2 = BRepBlendExtremity::new();
        ext1.set_value(previous.point_on_s1(), u1, v1, w, tol);
        ext2.set_value(previous.point_on_s2(), u2, v2, w, tol);
        if !previous.is_tangency_point() {
            ext1.set_tangent(previous.tangent_on_s1()?);
            ext2.set_tangent(previous.tangent_on_s2()?);
        }
        Ok((ext1, ext2))
    }

    /// `evalpinit(parinit, previousP, parprec, param, infbound, supbound, classonS1,
    /// classonS2)`: the start of the next solve, extrapolated along the 2d tangents of
    /// the previous point when it stays in the bounds (on the classified surfaces).
    pub(super) fn evalpinit(&self, march: &mut March) -> Result<(), Failure> {
        let previous: &BlendPoint = &self.previous_p;
        let (u1, v1) = previous.parameters_on_s1()?;
        let (u2, v2) = previous.parameters_on_s2()?;
        march.parinit = [u1, v1, u2, v2];
        if previous.is_tangency_point() {
            return Ok(());
        }
        let (d1, d2) = (previous.tangent2d_on_s1()?, previous.tangent2d_on_s2()?);
        let step = self.param - march.parprec;
        let guess = [
            u1 + step * d1.x(),
            v1 + step * d1.y(),
            u2 + step * d2.x(),
            v2 + step * d2.y(),
        ];
        let (inf, sup) = march.bounds;
        let outside = |i: usize| guess[i] < inf[i] || guess[i] > sup[i];
        let inside1 = !self.clason_s1 || !(outside(0) || outside(1));
        let inside2 = !self.clason_s2 || !(outside(2) || outside(3));
        if inside1 && inside2 {
            march.parinit = guess;
        }
        // else: on ne joue pas au plus malin
        Ok(())
    }
}

/// Whether a classification asks for a reframing (OUT or ON).
fn is_out_or_on(state: State) -> bool {
    state == State::Out || state == State::On
}
