// Rust translation of BRepBlend_SurfRstLineBuilder.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_SurfRstLineBuilder`: the marching of a blend function between a surface
//! and a restriction (a curve on another surface) along the guide, stopping on the
//! domain of the surface, on the ends of the restriction, or where the section leaves
//! the restriction.
mod arret;
mod extremity;
mod grounding;
mod internal;
mod perform;
mod recadre;
#[cfg(test)]
mod tests;

use super::marching::{self, ArcSearch, WalkingDomain};
use super::{BRepBlendLine, BRepBlendLineHandle};
use crate::blend::BlendPoint;
use crate::chfi_ds::{lock, shared};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dSurface};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;

pub use perform::{SurfRstInverses, SurfRstWalkSettings};

/// The class name in the failures.
const CLASS: &str = "BRepBlend_SurfRstLineBuilder";

/// The variables of a section `(U, V, W)`: the point on the surface and the parameter on
/// the restriction.
type Sol = [f64; 3];
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

    /// The result of a vertex search: the flag, and the vertex when found.
    fn set(&mut self, found: Option<Vertex>) {
        self.is_vtx = found.is_some();
        if let Some(vtx) = found {
            self.vtx = vtx;
        }
    }
}

/// `BRepBlend_SurfRstLineBuilder`: the line of a `Blend_SurfRstFunction` between the
/// surface `surf1` (domain `domain1`) and the restriction `rst` of the surface `surf2`
/// (domain `domain2`).
///
/// ```
/// use rustycad_tkfillet::brep_blend::BRepBlendSurfRstLineBuilder;
/// assert!(std::mem::size_of::<BRepBlendSurfRstLineBuilder>() > 0);
/// ```
pub struct BRepBlendSurfRstLineBuilder {
    done: bool,
    line: Option<BRepBlendLineHandle>,
    sol: Sol,
    surf1: Handle<dyn Adaptor3dSurface>,
    domain1: WalkingDomain,
    surf2: Handle<dyn Adaptor3dSurface>,
    rst: Arc,
    domain2: WalkingDomain,
    tolpoint3d: f64,
    tolpoint2d: f64,
    tolgui: f64,
    pasmax: f64,
    fleche: f64,
    param: f64,
    previous_p: BlendPoint,
    rebrou: bool,
    iscomplete: bool,
    comptra: bool,
    sens: f64,
    decrochdeb: bool,
    decrochfin: bool,
}

impl BRepBlendSurfRstLineBuilder {
    /// `BRepBlend_SurfRstLineBuilder(Surf1, Domain1, Surf2, Rst, Domain2)`;
    /// `BRepBlendSurfRstLineBuilder::new(&s1, &d1, &s2, &rst, &d2)`.
    pub fn new(
        surf1: &Handle<dyn Adaptor3dSurface>,
        domain1: &WalkingDomain,
        surf2: &Handle<dyn Adaptor3dSurface>,
        rst: &Arc,
        domain2: &WalkingDomain,
    ) -> Self {
        Self {
            done: false,
            line: None,
            sol: [0.0; 3],
            surf1: surf1.clone(),
            domain1: domain1.clone(),
            surf2: surf2.clone(),
            rst: rst.clone(),
            domain2: domain2.clone(),
            tolpoint3d: 0.0,
            tolpoint2d: 0.0,
            tolgui: 0.0,
            pasmax: 0.0,
            fleche: 0.0,
            param: 0.0,
            previous_p: BlendPoint::default(),
            rebrou: false,
            iscomplete: false,
            comptra: false,
            sens: 0.0,
            decrochdeb: false,
            decrochfin: false,
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

    /// `DecrochStart()`: whether the line starts where the section leaves the restriction.
    pub fn decroch_start(&self) -> bool {
        self.decrochdeb
    }

    /// `DecrochEnd()`: whether the line ends where the section leaves the restriction.
    pub fn decroch_end(&self) -> bool {
        self.decrochfin
    }

    /// `ArcToRecadre(Sol, PrevIndex, lastpt2d, pt2d, ponarc)`: the index of the arc of
    /// the surface domain (other than `prev_index`) crossed by the step to the solution
    /// `(U, V)` of `sol`, or else the nearest one (0 when none), with the previous point
    /// and the solution point; `ponarc` gets the parameter on the arc.
    /// `let (index, last, pt) = builder.arc_to_recadre(&sol, 0, &mut w)?;`.
    pub fn arc_to_recadre(
        &self,
        sol: &Vector<'_>,
        prev_index: i32,
        ponarc: &mut f64,
    ) -> Result<(i32, Point2, Point2), Failure> {
        let values: Sol = marching::values_of(sol)?;
        self.arc_search(&values, prev_index, ponarc)
    }

    /// The search of ArcToRecadre from the solution `sol`.
    fn arc_search(
        &self,
        sol: &Sol,
        prev_index: i32,
        ponarc: &mut f64,
    ) -> Result<(i32, Point2, Point2), Failure> {
        let byinter = lock(self.line_handle()?).nb_points() != 0;
        let previous = if byinter {
            Some(self.previous_p.parameters_on_s()?)
        } else {
            None
        };
        let pt2d = Point2::new(sol[0], sol[1]);
        let search = ArcSearch {
            domain: &self.domain1,
            surf: &self.surf1,
            pt2d,
            previous,
        };
        let index = search.run(prev_index, ponarc)?;
        Ok((index, search.lastpt2d(), pt2d))
    }

    /// The line, which the native code dereferences without a check.
    fn line_handle(&self) -> Result<&BRepBlendLineHandle, Failure> {
        self.line.as_ref().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepBlend_SurfRstLineBuilder: null line; expected a line built by Perform",
            )
        })
    }

    /// A new empty line (`line = new BRepBlend_Line()`).
    fn new_line(&mut self) {
        self.line = Some(shared(BRepBlendLine::new()));
    }

    /// The solution of the previous point `(U, V, W)`.
    fn previous_parameters(&mut self) -> Result<(), Failure> {
        let (u, v) = self.previous_p.parameters_on_s()?;
        self.sol = [u, v, self.previous_p.parameter_on_c()?];
        Ok(())
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

    /// The surface behind its handle.
    fn surface(&self, first: bool) -> Result<&dyn Adaptor3dSurface, Failure> {
        marching::live_surface(if first { &self.surf1 } else { &self.surf2 }, CLASS)
    }

    /// The restriction behind its handle.
    fn live_rst(&self) -> Result<&dyn Adaptor2dCurve2d, Failure> {
        marching::live_rst(&self.rst, CLASS)
    }
}
