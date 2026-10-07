// Rust translation of BRepBlend_Walking.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_Walking`: the marching of a blend function between two surfaces along the
//! guide, from a first section to the restrictions (or a bound) where it stops.
mod arret;
mod continu;
mod extremities;
mod first_on_rst;
mod first_section;
mod internal;
mod jalons;
mod recadre;
mod status;
#[cfg(test)]
mod tests;

use super::app_func::DynFunctionSet;
use super::{BRepBlendLine, BRepBlendLineHandle};
use crate::blend::BlendPoint;
use crate::blend_func::linalg::vector;
use crate::chfi_ds::{ElSpine, shared};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::int_start::TheTopolTool;
use rustycad_tkmath::math::{FunctionSetRoot, FunctionSetWithDerivatives, Vector};
use std::rc::Rc;

pub use first_section::WalkingTolerances;

/// The domain of a surface (`occ::handle<Adaptor3d_TopolTool>`): its restrictions and
/// vertices, shared with the caller, which keeps the iterators inside.
pub type WalkingDomain = Rc<dyn TheTopolTool>;

/// The variables of a section `(U1, V1, U2, V2)` (the `math_Vector(1, 4)` of the native
/// class).
type Sol = [f64; 4];

/// `BRepBlend_Walking`: the walking of a `Blend_Function` along the guide of a fillet.
///
/// It keeps the last computed point (`previousP`), the line, the solution, the singular
/// points (`jalons`), the surfaces, their domains (for the classification and for the
/// restrictions to stop on), the guide and the walking settings.
///
/// ```
/// use rustycad_tkfillet::brep_blend::BRepBlendWalking;
/// assert!(std::mem::size_of::<BRepBlendWalking>() > 0);
/// ```
pub struct BRepBlendWalking {
    previous_p: BlendPoint,
    line: Option<BRepBlendLineHandle>,
    sol: Sol,
    jalons: Vec<BlendPoint>,
    surf1: Handle<dyn Adaptor3dSurface>,
    surf2: Handle<dyn Adaptor3dSurface>,
    domain1: WalkingDomain,
    domain2: WalkingDomain,
    recdomain1: WalkingDomain,
    recdomain2: WalkingDomain,
    hguide: Handle<ElSpine>,
    to_correct_on_rst1: bool,
    to_correct_on_rst2: bool,
    corrected_param: f64,
    tolpoint3d: f64,
    tolgui: f64,
    pasmax: f64,
    fleche: f64,
    param: f64,
    sens: f64,
    done: bool,
    rebrou: bool,
    iscomplete: bool,
    comptra: bool,
    clason_s1: bool,
    clason_s2: bool,
    check2d: bool,
    check: bool,
    twistflag1: bool,
    twistflag2: bool,
}

impl BRepBlendWalking {
    /// `BRepBlend_Walking(Surf1, Surf2, Domain1, Domain2, HGuide)`: the domains also serve
    /// as the restrictions to stop on; `BRepBlendWalking::new(&s1, &s2, &d1, &d2, &guide)`.
    pub fn new(
        surf1: &Handle<dyn Adaptor3dSurface>,
        surf2: &Handle<dyn Adaptor3dSurface>,
        domain1: &WalkingDomain,
        domain2: &WalkingDomain,
        hguide: &Handle<ElSpine>,
    ) -> Self {
        Self {
            previous_p: BlendPoint::default(),
            line: None,
            sol: [0.0; 4],
            jalons: Vec::new(),
            surf1: surf1.clone(),
            surf2: surf2.clone(),
            domain1: domain1.clone(),
            domain2: domain2.clone(),
            recdomain1: domain1.clone(),
            recdomain2: domain2.clone(),
            hguide: hguide.clone(),
            to_correct_on_rst1: false,
            to_correct_on_rst2: false,
            corrected_param: 0.0,
            tolpoint3d: 0.0,
            tolgui: 0.0,
            pasmax: 0.0,
            fleche: 0.0,
            param: 0.0,
            sens: 0.0,
            done: false,
            rebrou: false,
            iscomplete: false,
            comptra: false,
            clason_s1: true,
            clason_s2: true,
            check2d: true,
            check: true,
            twistflag1: false,
            twistflag2: false,
        }
    }

    /// `SetDomainsToRecadre(RecDomain1, RecDomain2)`: the restrictions to stop on, when
    /// they differ from the classification domains; `walking.set_domains_to_recadre(&a, &b)`.
    pub fn set_domains_to_recadre(&mut self, domain1: &WalkingDomain, domain2: &WalkingDomain) {
        self.recdomain1 = domain1.clone();
        self.recdomain2 = domain2.clone();
    }

    /// `AddSingularPoint(P)`: a singular point computed before the walking, kept sorted
    /// by parameter; `walking.add_singular_point(&point)`.
    pub fn add_singular_point(&mut self, p: &BlendPoint) {
        if self.jalons.is_empty() {
            self.jalons.push(*p);
            return;
        }
        let tp = p.parameter();
        let mut ti = self.jalons[0].parameter();
        let (mut ii, mut jj) = (1, 1);
        while ii <= self.jalons.len() && tp > ti {
            jj = ii;
            ti = self.jalons[jj - 1].parameter();
            ii += 1;
        }
        let at = if tp > ti { jj } else { jj - 1 };
        self.jalons.insert(at, *p);
    }

    /// `ClassificationOnS1(C)`: whether the points are classified on the first domain.
    pub fn classification_on_s1(&mut self, c: bool) {
        self.clason_s1 = c;
    }

    /// `ClassificationOnS2(C)`: whether the points are classified on the second domain.
    pub fn classification_on_s2(&mut self, c: bool) {
        self.clason_s2 = c;
    }

    /// `Check2d(C)`: whether the deflection is also checked in the parametric spaces.
    pub fn check_2d(&mut self, c: bool) {
        self.check2d = c;
    }

    /// `Check(C)`: whether the deflection is checked at all.
    pub fn check(&mut self, c: bool) {
        self.check = c;
    }

    /// `TwistOnS1()`: whether a section twisted on the first surface.
    pub fn twist_on_s1(&self) -> bool {
        self.twistflag1
    }

    /// `TwistOnS2()`: whether a section twisted on the second surface.
    pub fn twist_on_s2(&self) -> bool {
        self.twistflag2
    }

    /// `IsDone()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `Line()`: the computed line; StdFail_NotDone before a successful `Perform`.
    /// `let line = walking.line()?;`.
    pub fn line(&self) -> Result<&BRepBlendLineHandle, Failure> {
        if !self.done {
            return Err(not_done("Line"));
        }
        self.line_handle()
    }

    /// The line, which the native code dereferences without a check.
    fn line_handle(&self) -> Result<&BRepBlendLineHandle, Failure> {
        self.line.as_ref().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepBlend_Walking: null line; expected a line built by Perform",
            )
        })
    }

    /// A new empty line (`line = new BRepBlend_Line()`).
    fn new_line(&mut self) {
        self.line = Some(shared(BRepBlendLine::new()));
    }

    /// The previous point with the current solution on both surfaces.
    fn previous_parameters(&mut self) -> Result<(), Failure> {
        let ((u1, v1), (u2, v2)) = (
            self.previous_p.parameters_on_s1()?,
            self.previous_p.parameters_on_s2()?,
        );
        self.sol = [u1, v1, u2, v2];
        Ok(())
    }

    /// The domain to stop on: the first or the second one.
    fn rec_domain(&self, on_first: bool) -> &WalkingDomain {
        if on_first {
            &self.recdomain1
        } else {
            &self.recdomain2
        }
    }

    /// The first or the second surface.
    fn surface(&self, on_first: bool) -> &Handle<dyn Adaptor3dSurface> {
        if on_first { &self.surf1 } else { &self.surf2 }
    }
}

/// The StdFail_NotDone of a member called before a successful walking.
fn not_done(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!("BRepBlend_Walking::{member}: the walking is not done; expected Perform first"),
    )
}

/// The surface behind a handle; a null handle is dereferenced natively.
fn live(surface: &Handle<dyn Adaptor3dSurface>) -> Result<&dyn Adaptor3dSurface, Failure> {
    surface.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "BRepBlend_Walking: null surface; expected an Adaptor3d_Surface",
        )
    })
}

/// The guide behind its handle.
fn live_guide(guide: &Handle<ElSpine>) -> Result<&ElSpine, Failure> {
    guide.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "BRepBlend_Walking: null guide; expected a ChFiDS_ElSpine",
        )
    })
}

/// A `math_Vector(1, n)` holding `values`; `math_vector(&[1.0, 2.0])?`.
fn math_vector(values: &[f64]) -> Result<Vector<'static>, Failure> {
    let mut v = vector(values.len() as i32)?;
    for (i, value) in (1..).zip(values) {
        crate::blend_func::linalg::vset(&mut v, i, *value)?;
    }
    Ok(v)
}

/// The four values of a `math_Vector(1, 4)`.
fn sol_of(v: &Vector<'_>) -> Result<Sol, Failure> {
    let get = |i| crate::blend_func::linalg::vget(v, i);
    Ok([get(1)?, get(2)?, get(3)?, get(4)?])
}

/// `math_FunctionSetRoot(F, Tolerance, NbIterations)`.
fn solver_for<T: FunctionSetWithDerivatives + ?Sized>(
    func: &mut T,
    tolerance: &Vector<'_>,
    nb_iterations: i32,
) -> Result<FunctionSetRoot, Failure> {
    FunctionSetRoot::new(&DynFunctionSet(func), tolerance, nb_iterations)
}

/// `rsnld.Perform(F, Start, Inf, Sup)`: the root when the solver is done.
fn root_of<T: FunctionSetWithDerivatives + ?Sized>(
    solver: &mut FunctionSetRoot,
    func: &mut T,
    start: &Sol,
    (inf, sup): (&Vector<'_>, &Vector<'_>),
) -> Result<Option<Sol>, Failure> {
    let start = math_vector(start)?;
    solver.perform_bounded(&mut DynFunctionSet(func), &start, inf, sup, false)?;
    if !solver.is_done() {
        return Ok(None);
    }
    Ok(Some(sol_of(solver.root()?)?))
}
