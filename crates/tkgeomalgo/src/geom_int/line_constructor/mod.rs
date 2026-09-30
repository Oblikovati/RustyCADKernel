//! `GeomInt_LineConstructor`: the parts of an intersection line inside both domains,
//! as parameter ranges (docs/geomalgo-design.md section 20). The analytic and walking
//! lines are in `walking`, the conics (and `TreatCircle`) in `conic`, the restrictions
//! in `restriction`, the file-static helpers in `helpers`.
mod conic;
pub(super) mod helpers;
mod restriction;
mod walking;

use crate::int_patch::{IntPatchIType, IntPatchLine};
use crate::int_start::TheTopolTool;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point2;
use std::rc::Rc;

/// `GeomInt_LineConstructor`: splits a line into the parts inside both domains
/// (`seqp` holds the first and last parameter of each part).
///
/// ```
/// use rustycad_tkgeomalgo::geom_int::GeomIntLineConstructor;
/// let lc = GeomIntLineConstructor::new();
/// assert!(!lc.is_done());
/// assert!(lc.nb_parts().is_err());
/// ```
#[derive(Clone, Default)]
pub struct GeomIntLineConstructor {
    done: bool,
    seqp: Vec<f64>,
    my_dom1: Option<Rc<dyn TheTopolTool>>,
    my_dom2: Option<Rc<dyn TheTopolTool>>,
    my_hs1: Handle<GeomAdaptorSurface>,
    my_hs2: Handle<GeomAdaptorSurface>,
}

/// The StdFail_NotDone of the part accessors before a successful `Perform`.
fn not_done(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!("GeomInt_LineConstructor::{member}: no line was split"),
    )
}

impl GeomIntLineConstructor {
    /// `GeomInt_LineConstructor()`: not done; `GeomIntLineConstructor::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Load(D1, D2, S1, S2)`: the domains and the surfaces of the lines to split;
    /// `lc.load(d1, d2, &s1, &s2)`.
    pub fn load(
        &mut self,
        d1: Rc<dyn TheTopolTool>,
        d2: Rc<dyn TheTopolTool>,
        s1: &Handle<GeomAdaptorSurface>,
        s2: &Handle<GeomAdaptorSurface>,
    ) {
        self.my_dom1 = Some(d1);
        self.my_dom2 = Some(d2);
        self.my_hs1 = s1.clone();
        self.my_hs2 = s2.clone();
    }

    /// `IsDone()`; `lc.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `NbParts()`: StdFail_NotDone before a successful `Perform`; `lc.nb_parts()?`.
    pub fn nb_parts(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("NbParts"));
        }
        Ok(self.seqp.len() as i32 / 2)
    }

    /// `Part(I, WFirst, WLast)`: the parameter range of the part `i` (StdFail_NotDone
    /// before a successful `Perform`, Standard_OutOfRange outside the parts);
    /// `let (first, last) = lc.part(1)?;`.
    pub fn part(&self, i: i32) -> Result<(f64, f64), Failure> {
        if !self.done {
            return Err(not_done("Part"));
        }
        Ok((self.seqp_at(2 * i - 1)?, self.seqp_at(2 * i)?))
    }

    /// `seqp(i)`, 1-based with the Standard_OutOfRange of `NCollection_Sequence`.
    fn seqp_at(&self, i: i32) -> Result<f64, Failure> {
        if i < 1 || i as usize > self.seqp.len() {
            return Err(Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "GeomInt_LineConstructor::Part: index {i} outside 1..={}",
                    self.seqp.len()
                ),
            ));
        }
        Ok(self.seqp[(i - 1) as usize])
    }

    /// Appends the part `first..=last`.
    fn push_part(&mut self, first: f64, last: f64) {
        self.seqp.push(first);
        self.seqp.push(last);
    }

    /// The domain `k` (1 or 2), or Standard_NullObject before `Load`.
    fn dom(&self, k: i32) -> Result<Rc<dyn TheTopolTool>, Failure> {
        let dom = if k == 1 { &self.my_dom1 } else { &self.my_dom2 };
        dom.clone().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "GeomInt_LineConstructor::Perform() - the domains are not loaded; expected Load first",
            )
        })
    }

    /// The loaded surfaces, or Standard_NullObject before `Load` (a null dereference
    /// natively).
    fn surfaces(&self) -> Result<(&GeomAdaptorSurface, &GeomAdaptorSurface), Failure> {
        match (self.my_hs1.get(), self.my_hs2.get()) {
            (Some(s1), Some(s2)) => Ok((s1, s2)),
            _ => Err(Failure::with_message(
                "Standard_NullObject",
                "GeomInt_LineConstructor::Perform() - the surfaces are not loaded; expected Load first",
            )),
        }
    }

    /// Whether `(u1, v1)` is not OUT of the first domain and `(u2, v2)` not OUT of the
    /// second one (the second is not classified when the first answers OUT).
    fn in_both(&self, uv: [f64; 4], tol: f64) -> Result<bool, Failure> {
        if self
            .dom(1)?
            .classify(Point2::new(uv[0], uv[1]), tol, true)?
            == State::Out
        {
            return Ok(false);
        }
        Ok(self
            .dom(2)?
            .classify(Point2::new(uv[2], uv[3]), tol, true)?
            != State::Out)
    }

    /// `Perform(L)`: the parts of the line inside both domains; a restriction line on
    /// no point of the domains is not done;
    /// `lc.perform(&line)?`.
    pub fn perform(&mut self, l: &Handle<dyn IntPatchLine>) -> Result<(), Failure> {
        let tol: f64 = precision::p_confusion() * 35.0;
        match l.arc_type() {
            IntPatchIType::Analytic => self.perform_aline(l, tol),
            IntPatchIType::Walking => self.perform_wline(l, tol),
            IntPatchIType::Restriction => self.perform_rline(l, tol),
            _ => self.perform_gline(l, tol),
        }
    }
}
