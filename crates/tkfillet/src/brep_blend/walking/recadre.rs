// Rust translation of BRepBlend_Walking::ArcToRecadre and Recadre (BRepBlend_Walking.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reframing of a section that left a domain: the restriction it crossed, and the
//! section ending on that restriction, solved by the inverse function.
use super::{BRepBlendWalking, Sol, math_vector, root_of, solver_for};
use crate::blend::BlendFuncInv;
use crate::blend_func::linalg::{vget, vset};
use crate::brep_blend::{BRepBlendBlendTool, BRepBlendHCurve2dTool};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dHVertex;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::{FunctionSetRoot, Vector};

/// A restriction arc of a domain.
pub(super) type Arc = Handle<dyn Adaptor2dCurve2d>;
/// A vertex of a domain.
pub(super) type Vertex = Handle<dyn Adaptor3dHVertex>;

/// The result of a successful `Recadre`: the section on the restriction
/// `(w, t, U, V)`, the index of the arc, and the vertex it ends on.
pub(super) struct Reframed {
    pub(super) solrst: Sol,
    pub(super) index: i32,
    pub(super) vertex: Option<Vertex>,
}

/// The arc found by `ArcToRecadre`, with the 2d points it was searched from.
struct ArcHit {
    index: i32,
    lastpt2d: Point2,
    pt2d: Point2,
}

/// One inverse solve of `Recadre`: the arc, its problem, the solution, the parameter on
/// the arc, the bounds of the arc, and whether the solution is a section.
struct Attempt {
    index: i32,
    problem: InverseProblem,
    solrst: Sol,
    pmin: f64,
    ends: (f64, f64),
    ok: bool,
}

/// The inverse problem on one arc: the curve, its bounds, the solver and the tolerance.
struct InverseProblem {
    thearc: Arc,
    bounds: (Vector<'static>, Vector<'static>),
    toler: Vector<'static>,
    solver: FunctionSetRoot,
}

impl BRepBlendWalking {
    /// `ArcToRecadre(OnFirst, Sol, PrevIndex, lpt2d, pt2d, ponarc)`: the index of the arc
    /// (other than `prev_index`) crossed by the segment from the previous point to the
    /// solution, or else the nearest one; 0 when none. `ponarc` gets the parameter on it.
    fn arc_to_recadre(
        &self,
        on_first: bool,
        the_sol: &Sol,
        prev_index: i32,
        ponarc: &mut f64,
    ) -> Result<ArcHit, Failure> {
        let (byinter, lastpt2d, pt2d) = self.search_points(on_first, the_sol)?;
        let iter = self.rec_domain(on_first);
        let (mut index_sol, mut nbarc, mut distmin) = (0, 0, f64::MAX);
        iter.init();
        while iter.more() {
            nbarc += 1;
            let arc = iter.value()?;
            let (prm, dist, okinter) =
                self.arc_distance(on_first, &arc, (pt2d, lastpt2d), byinter)?;
            if nbarc != prev_index && (dist < distmin || okinter) {
                (distmin, *ponarc, index_sol) = (dist, prm, nbarc);
                if okinter && prev_index == 0 {
                    break;
                }
            }
            iter.next();
        }
        Ok(ArcHit {
            index: index_sol,
            lastpt2d,
            pt2d,
        })
    }

    /// Whether the search goes by intersection (the line has points), the previous point
    /// (the origin otherwise) and the solution point on the surface.
    fn search_points(
        &self,
        on_first: bool,
        the_sol: &Sol,
    ) -> Result<(bool, Point2, Point2), Failure> {
        let byinter = crate::chfi_ds::lock(self.line_handle()?).nb_points() != 0;
        let (uprev, vprev) = if byinter {
            self.previous_uv(on_first)?
        } else {
            (0.0, 0.0)
        };
        let pt2d = if on_first {
            Point2::new(the_sol[0], the_sol[1])
        } else {
            Point2::new(the_sol[2], the_sol[3])
        };
        Ok((byinter, Point2::new(uprev, vprev), pt2d))
    }

    /// The parameter on the arc and the distance of the intersection of the step with
    /// it (when `byinter` and found: the last flag), else of the projection on it.
    fn arc_distance(
        &self,
        on_first: bool,
        arc: &Arc,
        (pt2d, lastpt2d): (Point2, Point2),
        byinter: bool,
    ) -> Result<(f64, f64, bool), Failure> {
        let surf = self.surface(on_first);
        if byinter && let Some((prm, dist)) = BRepBlendBlendTool::inters(pt2d, lastpt2d, surf, arc)?
        {
            return Ok((prm, dist, true));
        }
        let (prm, dist) = BRepBlendBlendTool::project(pt2d, surf, arc)?;
        Ok((prm, dist, false))
    }

    /// The parameters of the previous point on the first or the second surface.
    pub(super) fn previous_uv(&self, on_first: bool) -> Result<(f64, f64), Failure> {
        if on_first {
            self.previous_p.parameters_on_s1()
        } else {
            self.previous_p.parameters_on_s2()
        }
    }

    /// The arc of index `index` of a domain (`Init` then `Next` up to it).
    pub(super) fn nth_arc(&self, on_first: bool, index: i32) -> Result<Arc, Failure> {
        let iter = self.rec_domain(on_first);
        iter.init();
        let mut nbarc = 1;
        while nbarc < index {
            nbarc += 1;
            iter.next();
        }
        iter.value()
    }

    /// `Recadre(FInv, OnFirst, Sol, Solrst, Indexsol, IsVtx, Vtx, Extrap)`: the section
    /// ending on the restriction crossed by the solution `the_sol`, or `None`.
    pub(super) fn recadre(
        &mut self,
        func_inv: &mut dyn BlendFuncInv,
        on_first: bool,
        the_sol: &Sol,
        extrap: f64,
    ) -> Result<Option<Reframed>, Failure> {
        let mut pmin = 0.0;
        let hit = self.arc_to_recadre(on_first, the_sol, 0, &mut pmin)?;
        if hit.index == 0 {
            return Ok(None);
        }
        let mut attempt = self.first_attempt(func_inv, on_first, the_sol, &hit, (pmin, extrap))?;
        if !attempt.ok && !self.retry_attempt(func_inv, on_first, the_sol, &mut attempt)? {
            return Ok(None);
        }
        if !attempt.ok {
            return Ok(None);
        }
        let toler_1 = attempt.problem.toler_1()?;
        let vertex = self.vertex_at(
            on_first,
            &attempt.problem.thearc,
            attempt.solrst[0],
            toler_1,
        )?;
        Ok(Some(Reframed {
            solrst: attempt.solrst,
            index: attempt.index,
            vertex,
        }))
    }

    /// The inverse solve on the arc found first, from the singular point crossed by the
    /// step when it is a solution, else from the interpolated start.
    fn first_attempt(
        &self,
        func_inv: &mut dyn BlendFuncInv,
        on_first: bool,
        the_sol: &Sol,
        hit: &ArcHit,
        (pmin, extrap): (f64, f64),
    ) -> Result<Attempt, Failure> {
        let mut problem = self.inverse_problem(func_inv, on_first, hit.index, extrap)?;
        let ends = BRepBlendBlendTool::bounds(&problem.thearc)?;
        let pmin = snap_to_ends(pmin, ends);
        let (mut solrst, jalon) =
            self.recadre_start(on_first, the_sol, hit, pmin, &problem.thearc)?;
        let thearc = problem.thearc.clone();
        let mut ok = match jalon {
            Some(jalon) => {
                self.jalon_start(func_inv, on_first, jalon, pmin, &thearc, &mut solrst)?
            }
            None => false,
        };
        if !ok {
            ok = self.solve_inverse(func_inv, &mut problem, &mut solrst)?;
        }
        Ok(Attempt {
            index: hit.index,
            problem,
            solrst,
            pmin,
            ends,
            ok,
        })
    }

    /// En cas d'echecs, on regarde si un autre arc peut faire l'affaire (cas des sorties
    /// a proximite d'un vertex): false when the failure was near an end of the arc and
    /// no other arc is found.
    fn retry_attempt(
        &self,
        func_inv: &mut dyn BlendFuncInv,
        on_first: bool,
        the_sol: &Sol,
        attempt: &mut Attempt,
    ) -> Result<bool, Failure> {
        let (ufirst, ulast) = attempt.ends;
        let dist = (ulast - ufirst) / 100.0;
        let near_an_end =
            (attempt.pmin - ulast).abs() < dist || (attempt.pmin - ufirst).abs() < dist;
        if !near_an_end {
            return Ok(true);
        }
        let retry = self.arc_to_recadre(on_first, the_sol, attempt.index, &mut attempt.pmin)?;
        if retry.index == 0 {
            return Ok(false);
        }
        attempt.index = retry.index;
        attempt.problem = self.inverse_problem(func_inv, on_first, retry.index, 0.0)?;
        attempt.solrst[0] = attempt.pmin;
        attempt.ok = self.solve_inverse(func_inv, &mut attempt.problem, &mut attempt.solrst)?;
        Ok(true)
    }

    /// The inverse function set on the arc `index`, its bounds widened by `extrap` on
    /// the guide parameter, and its solver (35 iterations, a tenth of the tolerance).
    fn inverse_problem(
        &self,
        func_inv: &mut dyn BlendFuncInv,
        on_first: bool,
        index: i32,
        extrap: f64,
    ) -> Result<InverseProblem, Failure> {
        let thearc = self.nth_arc(on_first, index)?;
        let thecur = BRepBlendBlendTool::curve_on_surf(&thearc, self.surface(on_first));
        // Le probleme a resoudre
        func_inv.set_curve(on_first, &thecur)?;
        let (mut infb, mut supb) = (math_vector(&[0.0; 4])?, math_vector(&[0.0; 4])?);
        func_inv.get_bounds(&mut infb, &mut supb)?;
        let (inf2, sup2) = (vget(&infb, 2)?, vget(&supb, 2)?);
        vset(&mut infb, 2, inf2 - extrap)?;
        vset(&mut supb, 2, sup2 + extrap)?;
        let mut toler = math_vector(&[0.0; 4])?;
        // Il vaut mieux garder un peu de marge
        func_inv.get_tolerance(&mut toler, 0.1 * self.tolpoint3d)?;
        let solver = solver_for(func_inv, &toler, 35)?;
        toler *= 10.0; // Mais on fait les tests correctements
        Ok(InverseProblem {
            thearc: thecur,
            bounds: (infb, supb),
            toler,
            solver,
        })
    }

    /// The start of the inverse solve: interpolated between the previous point and the
    /// solution, or the solution itself on a new line; and the singular point (jalon)
    /// crossed by the step, if any.
    fn recadre_start(
        &self,
        on_first: bool,
        the_sol: &Sol,
        hit: &ArcHit,
        pmin: f64,
        thecur: &Arc,
    ) -> Result<(Sol, Option<usize>), Failure> {
        let other = if on_first {
            [the_sol[2], the_sol[3]]
        } else {
            [the_sol[0], the_sol[1]]
        };
        let byinter = crate::chfi_ds::lock(self.line_handle()?).nb_points() != 0;
        if !byinter {
            // sinon on initialise par le dernier point calcule
            return Ok(([pmin, self.param, other[0], other[1]], None));
        }
        let last_param = self.previous_p.parameter();
        if let Some(jalon) = self.crossed_jalon(last_param) {
            return Ok(([0.0; 4], Some(jalon)));
        }
        // Initialisation par Interpolation
        let pnt = BRepBlendHCurve2dTool::d0(thecur, pmin)?;
        let (u, v) = self.previous_uv(!on_first)?;
        let mut lambda = pnt.distance(hit.lastpt2d);
        if lambda > 1.0e-12 {
            lambda /= pnt.distance(hit.lastpt2d) + pnt.distance(hit.pt2d);
        } else {
            lambda = 0.0;
        }
        Ok((
            [
                pmin,
                (1.0 - lambda) * last_param + lambda * self.param,
                (1.0 - lambda) * u + lambda * other[0],
                (1.0 - lambda) * v + lambda * other[1],
            ],
            None,
        ))
    }

    /// Resolution...: the inverse solve from `solrst`, and whether its root is a solution.
    fn solve_inverse(
        &self,
        func_inv: &mut dyn BlendFuncInv,
        problem: &mut InverseProblem,
        solrst: &mut Sol,
    ) -> Result<bool, Failure> {
        let (inf, sup) = (&problem.bounds.0, &problem.bounds.1);
        let root = root_of(&mut problem.solver, func_inv, solrst, (inf, sup));
        func_inv.take_failure()?;
        let Some(root) = root? else {
            return Ok(false);
        };
        *solrst = root;
        func_inv.is_solution(&math_vector(solrst)?, self.tolpoint3d)
    }

    /// Classification topologique: the vertex of the arc whose ball holds the parameter
    /// `w` (or that lies in the ball of the reframing), if any.
    fn vertex_at(
        &self,
        on_first: bool,
        thearc: &Arc,
        w: f64,
        toler_1: f64,
    ) -> Result<Option<Vertex>, Failure> {
        let (ufirst, ulast) = BRepBlendBlendTool::bounds(thearc)?;
        let iter = self.rec_domain(on_first);
        iter.initialize_arc(thearc)?;
        iter.init_vertex_iterator()?;
        while iter.more_vertex() {
            let vtx = iter.vertex()?;
            let mut vtol = 0.4 * (ulast - ufirst).abs(); // Un majorant de la tolerance
            let vertex_tol = std_max(BRepBlendBlendTool::tolerance(&vtx, thearc)?, toler_1);
            if vtol > vertex_tol {
                vtol = vertex_tol;
            }
            if (BRepBlendBlendTool::parameter(&vtx, thearc)? - w).abs() <= vtol {
                // On est dans la boule du vertex ou le vertex est dans la "boule" du
                // recadrage
                return Ok(Some(vtx));
            }
            iter.next_vertex()?;
        }
        Ok(None)
    }
}

impl InverseProblem {
    /// `toler(1)` after the factor 10.
    fn toler_1(&self) -> Result<f64, Failure> {
        vget(&self.toler, 1)
    }
}

/// Pour aider a trouver les coins singuliers on recadre eventuellement le parametre:
/// `pmin` moved to an end of the arc within a thousandth of its range.
fn snap_to_ends(pmin: f64, (ufirst, ulast): (f64, f64)) -> f64 {
    let mut pmin = pmin;
    if (pmin - ufirst).abs() < (ulast - ufirst).abs() / 1000.0 {
        pmin = ufirst;
    }
    if (pmin - ulast).abs() < (ulast - ufirst).abs() / 1000.0 {
        pmin = ulast;
    }
    pmin
}
