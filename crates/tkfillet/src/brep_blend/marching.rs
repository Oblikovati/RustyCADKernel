// Rust translation of the code shared by BRepBlend_Walking, BRepBlend_SurfRstLineBuilder and
// BRepBlend_RstRstLineBuilder (their .cxx files) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! What the three marching algorithms of BRepBlend share: the solves of their functions,
//! the 3d deflection test of a step (the "controls 3d of Blend_CSWalking"), and the
//! failures of their members.
use super::BRepBlendBlendTool;
use super::app_func::DynFunctionSet;
use crate::blend::{BlendPoint, Status};
use crate::blend_func::linalg::{vector, vget, vset};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dHVertex;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::int_start::TheTopolTool;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{FunctionSetRoot, FunctionSetWithDerivatives, Vector};
use std::rc::Rc;

/// The domain of a surface (`occ::handle<Adaptor3d_TopolTool>`): its restrictions and
/// vertices, shared with the caller, which keeps the iterators inside.
pub type WalkingDomain = Rc<dyn TheTopolTool>;

/// `CosRef3D` of the 3d deflection test: rule set by tests in U4, 11.478 degrees.
const COS_REF_3D: f64 = 0.98;

/// A `math_Vector(1, n)` holding `values`; `math_vector(&[1.0, 2.0])?`.
pub(crate) fn math_vector(values: &[f64]) -> Result<Vector<'static>, Failure> {
    let mut v = vector(values.len() as i32)?;
    for (i, value) in (1..).zip(values) {
        vset(&mut v, i, *value)?;
    }
    Ok(v)
}

/// The `N` values of a `math_Vector(1, N)`.
pub(crate) fn values_of<const N: usize>(v: &Vector<'_>) -> Result<[f64; N], Failure> {
    let mut out = [0.0; N];
    for (i, value) in (1..).zip(out.iter_mut()) {
        *value = vget(v, i)?;
    }
    Ok(out)
}

/// `math_FunctionSetRoot(F, Tolerance, NbIterations)`.
pub(crate) fn solver_for<T: FunctionSetWithDerivatives + ?Sized>(
    func: &mut T,
    tolerance: &Vector<'_>,
    nb_iterations: i32,
) -> Result<FunctionSetRoot, Failure> {
    FunctionSetRoot::new(&DynFunctionSet(func), tolerance, nb_iterations)
}

/// `rsnld.Perform(F, Start, Inf, Sup)`: the root when the solver is done.
pub(crate) fn root_of<T: FunctionSetWithDerivatives + ?Sized, const N: usize>(
    solver: &mut FunctionSetRoot,
    func: &mut T,
    start: &[f64; N],
    (inf, sup): (&Vector<'_>, &Vector<'_>),
) -> Result<Option<[f64; N]>, Failure> {
    let start = math_vector(start)?;
    solver.perform_bounded(&mut DynFunctionSet(func), &start, inf, sup, false)?;
    if !solver.is_done() {
        return Ok(None);
    }
    Ok(Some(values_of(solver.root()?)?))
}

/// The surface behind a handle; a null handle is dereferenced natively.
pub(crate) fn live_surface<'a>(
    surface: &'a Handle<dyn Adaptor3dSurface>,
    class: &str,
) -> Result<&'a dyn Adaptor3dSurface, Failure> {
    surface.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("{class}: null surface; expected an Adaptor3d_Surface"),
        )
    })
}

/// The restriction behind a handle; a null handle is dereferenced natively.
pub(crate) fn live_rst<'a>(
    rst: &'a Handle<dyn Adaptor2dCurve2d>,
    class: &str,
) -> Result<&'a dyn Adaptor2dCurve2d, Failure> {
    rst.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("{class}: null restriction; expected an Adaptor2d_Curve2d"),
        )
    })
}

/// The StdFail_NotDone of a member called before a successful marching.
pub(crate) fn not_done(class: &str, member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!("{class}::{member}: the marching is not done; expected Perform first"),
    )
}

/// The 3d and 2d data of one end of a line point, for the deflection tests.
pub(crate) struct EndData {
    pub(crate) point: Point3,
    pub(crate) tangent: Vector3,
    pub(crate) uv: (f64, f64),
    pub(crate) tangent2d: Vector2,
}

/// The point, the tangents (when `tangent`) and the parameters (when `with_2d`) of the
/// first (`PointOnS1`, `PointOnS`, `PointOnC1`) or the second end of a line point.
pub(crate) fn end_data(
    point: &BlendPoint,
    on_first: bool,
    tangent: bool,
    with_2d: bool,
) -> Result<EndData, Failure> {
    let mut data = EndData {
        point: if on_first {
            point.point_on_s1()
        } else {
            point.point_on_s2()
        },
        tangent: Vector3::default(),
        uv: (0.0, 0.0),
        tangent2d: Vector2::default(),
    };
    if tangent {
        data.tangent = if on_first {
            point.tangent_on_s1()?
        } else {
            point.tangent_on_s2()?
        };
    }
    if !with_2d {
        return Ok(data);
    }
    data.uv = if on_first {
        point.parameters_on_s1()?
    } else {
        point.parameters_on_s2()?
    };
    if tangent {
        data.tangent2d = if on_first {
            point.tangent2d_on_s1()?
        } else {
            point.tangent2d_on_s2()?
        };
    }
    Ok(data)
}

/// The settings of the 3d deflection test: the 3d tolerance, the direction of the
/// marching and the requested deflection.
#[derive(Clone, Copy)]
pub(crate) struct Deflection {
    pub(crate) tolpoint3d: f64,
    pub(crate) sens: f64,
    pub(crate) fleche: f64,
}

impl Deflection {
    /// The chord of the step and its square length, or SamePoints when it is below a
    /// hundredth of the tolerance ("it can be necessary to force same point").
    pub(crate) fn chord(&self, cur: &EndData, prev: &EndData) -> Result<(Vector3, f64), Status> {
        let corde = Vector3::between(prev.point, cur.point);
        let norme = corde.square_magnitude();
        let toler3d = 0.01 * self.tolpoint3d;
        if norme <= toler3d * toler3d {
            return Err(Status::SamePoints);
        }
        Ok((corde, norme))
    }

    /// The 3d angles of the chord with the previous and the current tangents (when they
    /// exist): `None` when the step passes them.
    pub(crate) fn check_angles(
        &self,
        (corde, norme): (Vector3, f64),
        (cur, prev): (&EndData, &EndData),
        (cur_tangent, prev_tangent): (bool, bool),
    ) -> Option<Status> {
        let toler3d = 0.01 * self.tolpoint3d;
        if prev_tangent {
            let prev_norme = prev.tangent.square_magnitude();
            if prev_norme <= toler3d * toler3d {
                return Some(Status::SamePoints);
            }
            let cosi = self.sens * corde.dot(prev.tangent);
            if cosi < 0.0 {
                // angle 3d > pi/2 --> retour arriere
                return Some(Status::Backward);
            }
            if cosi * cosi / prev_norme / norme < COS_REF_3D {
                return Some(Status::StepTooLarge);
            }
        }
        if cur_tangent {
            let cosi = self.sens * corde.dot(cur.tangent);
            let cosi2 = cosi * cosi / cur.tangent.square_magnitude() / norme;
            if cosi2 < COS_REF_3D || cosi < 0.0 {
                return Some(Status::StepTooLarge);
            }
        }
        None
    }

    /// Estimation de la fleche courante: the deflection of the step against the
    /// requested one (both points have tangents).
    pub(crate) fn check_fleche(
        &self,
        cur: &EndData,
        prev: &EndData,
        norme: f64,
    ) -> Result<Status, Failure> {
        let gap = *prev.tangent.normalized()?.xyz() - *cur.tangent.normalized()?.xyz();
        let fleche_courante = gap.square_modulus() * norme / 64.0;
        if fleche_courante <= 0.25 * self.fleche * self.fleche {
            return Ok(Status::StepTooSmall);
        }
        if fleche_courante > self.fleche * self.fleche {
            // pas trop grand : commentaire interessant
            return Ok(Status::StepTooLarge);
        }
        Ok(Status::Ok)
    }

    /// The 3d deflection test of a step on one end (`CheckDeflectionOnSurf`,
    /// `CheckDeflectionOnRst` of the line builders): the chord, the angles, the
    /// deflection. `deflection.check_3d(&cur, &prev, (true, true))?`.
    pub(crate) fn check_3d(
        &self,
        cur: &EndData,
        prev: &EndData,
        tangents: (bool, bool),
    ) -> Result<Status, Failure> {
        let (corde, norme) = match self.chord(cur, prev) {
            Ok(chord) => chord,
            Err(status) => return Ok(status),
        };
        if let Some(status) = self.check_angles((corde, norme), (cur, prev), tangents) {
            return Ok(status);
        }
        if !tangents.0 || !tangents.1 {
            return Ok(Status::Ok);
        }
        self.check_fleche(cur, prev, norme)
    }
}

/// The step status on two ends with a backward one turned too large; the flag tells
/// that one was backward (`rebrou`).
pub(crate) fn without_backward((s1, s2): (Status, Status)) -> ((Status, Status), bool) {
    let fix = |s: Status| {
        if s == Status::Backward {
            Status::StepTooLarge
        } else {
            s
        }
    };
    (
        (fix(s1), fix(s2)),
        s1 == Status::Backward || s2 == Status::Backward,
    )
}

/// The search of `ArcToRecadre`: the arc of a domain that a step from the previous point
/// (when the line has points) to the point `pt2d` crosses, or else the nearest one.
pub(crate) struct ArcSearch<'a> {
    pub(crate) domain: &'a WalkingDomain,
    pub(crate) surf: &'a Handle<dyn Adaptor3dSurface>,
    pub(crate) pt2d: Point2,
    pub(crate) previous: Option<(f64, f64)>,
}

impl ArcSearch<'_> {
    /// The previous point, the origin on a new line (`lastpt2d`).
    pub(crate) fn lastpt2d(&self) -> Point2 {
        let (u, v) = self.previous.unwrap_or((0.0, 0.0));
        Point2::new(u, v)
    }

    /// The index of the arc (other than `prev_index`), 0 when none; `ponarc` gets the
    /// parameter on it. The first intersection ends a first search.
    pub(crate) fn run(&self, prev_index: i32, ponarc: &mut f64) -> Result<i32, Failure> {
        let (mut index_sol, mut nbarc, mut distmin) = (0, 0, f64::MAX);
        self.domain.init();
        while self.domain.more() {
            nbarc += 1;
            let (prm, dist, okinter) = self.arc_distance(&self.domain.value()?)?;
            if nbarc != prev_index && (dist < distmin || okinter) {
                (distmin, *ponarc, index_sol) = (dist, prm, nbarc);
                if okinter && prev_index == 0 {
                    break;
                }
            }
            self.domain.next();
        }
        Ok(index_sol)
    }

    /// The parameter on the arc and the distance of the intersection of the step with
    /// it (when the line has points and it is found: the last flag), else of the
    /// projection on it.
    fn arc_distance(
        &self,
        arc: &Handle<dyn Adaptor2dCurve2d>,
    ) -> Result<(f64, f64, bool), Failure> {
        if self.previous.is_some()
            && let Some((prm, dist)) =
                BRepBlendBlendTool::inters(self.pt2d, self.lastpt2d(), self.surf, arc)?
        {
            return Ok((prm, dist, true));
        }
        let (prm, dist) = BRepBlendBlendTool::project(self.pt2d, self.surf, arc)?;
        Ok((prm, dist, false))
    }
}

/// The vertex of the domain on the arc whose parameter is within its tolerance of `w`
/// (the vertex loops of the line builders), and the last vertex examined, which the
/// native `Vtx` output keeps when none matches.
pub(crate) struct VertexSearch {
    pub(crate) found: bool,
    pub(crate) last: Option<Handle<dyn Adaptor3dHVertex>>,
}

/// The vertex loop of the line builders on the arc `arc` of `domain` at `w`.
pub(crate) fn vertex_at_parameter(
    domain: &WalkingDomain,
    arc: &Handle<dyn Adaptor2dCurve2d>,
    w: f64,
) -> Result<VertexSearch, Failure> {
    domain.initialize_arc(arc)?;
    domain.init_vertex_iterator()?;
    let mut last = None;
    while domain.more_vertex() {
        let vtx = domain.vertex()?;
        let matched = (BRepBlendBlendTool::parameter(&vtx, arc)? - w).abs()
            <= BRepBlendBlendTool::tolerance(&vtx, arc)?;
        last = Some(vtx);
        if matched {
            return Ok(VertexSearch { found: true, last });
        }
        domain.next_vertex()?;
    }
    Ok(VertexSearch { found: false, last })
}

/// The arc of index `index` of a domain (`Init` then `Next` up to it).
pub(crate) fn nth_arc(
    domain: &WalkingDomain,
    index: i32,
) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
    domain.init();
    let mut nbarc = 1;
    while nbarc < index {
        nbarc += 1;
        domain.next();
    }
    domain.value()
}

/// The end of the step tests: the status and whether the point is kept as the previous
/// one. A step accepted on one end returns `state`; too small on both ends, the status
/// StepTooSmall (or `state`); else SamePoints (or `state`), the point not kept.
pub(crate) fn step_outcome((s1, s2): (Status, Status), state: Status) -> (Status, bool) {
    if s1 == Status::Ok || s2 == Status::Ok {
        return (state, true);
    }
    if s1 == Status::StepTooSmall && s2 == Status::StepTooSmall {
        return (
            if state == Status::Ok {
                Status::StepTooSmall
            } else {
                state
            },
            true,
        );
    }
    (
        if state == Status::Ok {
            Status::SamePoints
        } else {
            state
        },
        false,
    )
}

/// The tangent of the line at the previous point on its first or second end: its own
/// one, or the chord from the point before when it is a tangency point; `None` on a
/// line too short for a chord (the transitions are then left undecided).
pub(crate) fn line_tangent(
    line: &super::BRepBlendLine,
    previous: &BlendPoint,
    sens: f64,
    on_first: bool,
) -> Result<Option<Vector3>, Failure> {
    if !previous.is_tangency_point() {
        let tangent = if on_first {
            previous.tangent_on_s1()?
        } else {
            previous.tangent_on_s2()?
        };
        return Ok(Some(tangent));
    }
    if line.nb_points() < 2 {
        return Ok(None);
    }
    let index = if sens < 0.0 { 2 } else { line.nb_points() - 1 };
    let prevprev = *line.point(index)?;
    let chord = if on_first {
        Vector3::between(prevprev.point_on_s1(), previous.point_on_s1())
    } else {
        Vector3::between(prevprev.point_on_s2(), previous.point_on_s2())
    };
    Ok(Some(chord))
}
