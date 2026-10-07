// Rust translation of BRepBlend_RstRstLineBuilder.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_RstRstLineBuilder`: the marching of a blend function between two
//! restrictions (curves on two surfaces) along the guide, stopping on the ends of the
//! restrictions or where the section leaves one of them.
mod arret;
mod grounding;
mod internal;
mod perform;
mod recadre;
#[cfg(test)]
mod tests;

use super::marching::{self, WalkingDomain};
use super::{BRepBlendLine, BRepBlendLineHandle};
use crate::blend::{BlendPoint, DecrochStatus};
use crate::chfi_ds::{lock, shared};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dSurface};

pub use perform::{RstRstInverses, RstRstWalkSettings};

/// The class name in the failures.
const CLASS: &str = "BRepBlend_RstRstLineBuilder";

/// The variables of a section `(U, V)`: the parameters on both restrictions.
type Sol = [f64; 2];
/// A restriction arc.
type Arc = Handle<dyn Adaptor2dCurve2d>;
/// A vertex of a domain.
type Vertex = Handle<dyn Adaptor3dHVertex>;

/// The `IsVtx`, `Vtx` outputs of the reframings: kept between the steps as the native
/// locals are.
#[derive(Clone, Default)]
struct VertexFlag {
    is_vtx: bool,
    vtx: Vertex,
}

impl VertexFlag {
    /// The vertex when the flag is set.
    fn vertex(&self) -> Option<&Vertex> {
        self.is_vtx.then_some(&self.vtx)
    }

    /// The result of a vertex search: the flag, and the last vertex examined (the
    /// vertex found, or the last one of the arc).
    fn set(&mut self, search: marching::VertexSearch) {
        self.is_vtx = search.found;
        if let Some(vtx) = search.last {
            self.vtx = vtx;
        }
    }
}

/// `BRepBlend_RstRstLineBuilder`: the line of a `Blend_RstRstFunction` between the
/// restriction `rst1` of `surf1` (domain `domain1`) and the restriction `rst2` of
/// `surf2` (domain `domain2`).
///
/// ```
/// use rustycad_tkfillet::brep_blend::BRepBlendRstRstLineBuilder;
/// assert!(std::mem::size_of::<BRepBlendRstRstLineBuilder>() > 0);
/// ```
pub struct BRepBlendRstRstLineBuilder {
    done: bool,
    line: Option<BRepBlendLineHandle>,
    sol: Sol,
    surf: [Handle<dyn Adaptor3dSurface>; 2],
    domain: [WalkingDomain; 2],
    rst: [Arc; 2],
    tolpoint3d: f64,
    tolgui: f64,
    pasmax: f64,
    fleche: f64,
    param: f64,
    previous_p: BlendPoint,
    rebrou: bool,
    iscomplete: bool,
    comptra: bool,
    sens: f64,
    decrochdeb: DecrochStatus,
    decrochfin: DecrochStatus,
}

impl BRepBlendRstRstLineBuilder {
    /// `BRepBlend_RstRstLineBuilder(Surf1, Rst1, Domain1, Surf2, Rst2, Domain2)`;
    /// `BRepBlendRstRstLineBuilder::new((&s1, &rst1, &d1), (&s2, &rst2, &d2))`.
    pub fn new(
        (surf1, rst1, domain1): (&Handle<dyn Adaptor3dSurface>, &Arc, &WalkingDomain),
        (surf2, rst2, domain2): (&Handle<dyn Adaptor3dSurface>, &Arc, &WalkingDomain),
    ) -> Self {
        Self {
            done: false,
            line: None,
            sol: [0.0; 2],
            surf: [surf1.clone(), surf2.clone()],
            domain: [domain1.clone(), domain2.clone()],
            rst: [rst1.clone(), rst2.clone()],
            tolpoint3d: 0.0,
            tolgui: 0.0,
            pasmax: 0.0,
            fleche: 0.0,
            param: 0.0,
            previous_p: BlendPoint::default(),
            rebrou: false,
            iscomplete: false,
            comptra: false,
            sens: 0.0,
            decrochdeb: DecrochStatus::NoDecroch,
            decrochfin: DecrochStatus::NoDecroch,
        }
    }

    /// `IsDone()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `Line()`: the computed line; StdFail_NotDone before a successful `Perform`.
    pub fn line(&self) -> Result<&BRepBlendLineHandle, Failure> {
        if !self.done {
            return Err(marching::not_done(CLASS, "Line"));
        }
        self.line_handle()
    }

    /// `Decroch1Start()`: whether the line starts where the section leaves the first
    /// restriction.
    pub fn decroch1_start(&self) -> bool {
        matches!(
            self.decrochdeb,
            DecrochStatus::DecrochRst1 | DecrochStatus::DecrochBoth
        )
    }

    /// `Decroch2Start()`: whether the line starts where the section leaves the second
    /// restriction.
    pub fn decroch2_start(&self) -> bool {
        matches!(
            self.decrochdeb,
            DecrochStatus::DecrochRst2 | DecrochStatus::DecrochBoth
        )
    }

    /// `Decroch1End()`: whether the line ends where the section leaves the first
    /// restriction.
    pub fn decroch1_end(&self) -> bool {
        matches!(
            self.decrochfin,
            DecrochStatus::DecrochRst1 | DecrochStatus::DecrochBoth
        )
    }

    /// `Decroch2End()`: whether the line ends where the section leaves the second
    /// restriction.
    pub fn decroch2_end(&self) -> bool {
        matches!(
            self.decrochfin,
            DecrochStatus::DecrochRst2 | DecrochStatus::DecrochBoth
        )
    }

    /// The line, which the native code dereferences without a check.
    fn line_handle(&self) -> Result<&BRepBlendLineHandle, Failure> {
        self.line.as_ref().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepBlend_RstRstLineBuilder: null line; expected a line built by Perform",
            )
        })
    }

    /// A new empty line (`line = new BRepBlend_Line()`).
    fn new_line(&mut self) {
        self.line = Some(shared(BRepBlendLine::new()));
    }

    /// The previous point appended to the line (prepended when marching backward).
    fn push_previous(&self) -> Result<(), Failure> {
        let mut line = lock(self.line_handle()?);
        if self.sens > 0.0 {
            line.append(self.previous_p);
        } else {
            line.prepend(self.previous_p);
        }
        Ok(())
    }

    /// The surface of a side (0 or 1) behind its handle.
    fn surface(&self, side: usize) -> Result<&dyn Adaptor3dSurface, Failure> {
        marching::live_surface(&self.surf[side], CLASS)
    }

    /// The restriction of a side (0 or 1) behind its handle.
    fn live_rst(&self, side: usize) -> Result<&dyn Adaptor2dCurve2d, Failure> {
        marching::live_rst(&self.rst[side], CLASS)
    }
}
