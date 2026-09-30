// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_Intersection`: the front end of the surface-surface intersection
//! (docs/geomalgo-design.md section 18). It classifies the pair of surfaces and runs the
//! implicit-implicit, implicit-parametric or parametric-parametric intersection, then
//! purges the walking lines. The accessors and the tolerances are here; the static
//! singular-point members are in `singular.rs`.
mod classify;
mod geom_geom;
mod infinite;
mod param_param;
mod perform;
mod perform_other;
mod pl_inter;
mod singular;
pub use perform::IntPatchIntersectionOptions;

use super::{IntPatchLine, IntPatchPoint};
use crate::int_start::TheTopolTool;
use crate::int_start::failures::sequence_value;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;

/// `IntPatch_Intersection`: the intersection lines and points of two surfaces (or of a
/// surface with itself), with the tangency and the orientation of tangent faces.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchIntersection;
/// let mut inter = IntPatchIntersection::new();
/// inter.set_tolerances(1.0e-9, 1.0, 0.1, 100.0);
/// assert!(!inter.is_done());
/// assert!(inter.nb_lines().is_err());
/// ```
#[derive(Default)]
pub struct IntPatchIntersection {
    done: bool,
    empt: bool,
    tgte: bool,
    oppo: bool,
    spnt: Vec<IntPatchPoint>,
    slin: Vec<Handle<dyn IntPatchLine>>,
    my_tol_arc: f64,
    my_tol_tang: f64,
    my_uv_max_step: f64,
    my_fleche: f64,
    my_is_start_pnt: bool,
    /// `myU1Start, myV1Start, myU2Start, myV2Start`.
    my_start: [f64; 4],
}

/// The StdFail_NotDone of a result read before a Perform.
fn not_done(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!(
            "IntPatch_Intersection::{member}() - the intersection is not done; expected a Perform"
        ),
    )
}

/// Clamps a tolerance to `[1e-8, 0.5]`, the small bound first.
fn clamp_tolerance(tol: f64) -> f64 {
    // `clamp` keeps a NaN, as the two native tests do.
    tol.clamp(1e-8, 0.5)
}

impl IntPatchIntersection {
    /// The empty constructor: not done, empty, every tolerance 0.
    /// `IntPatchIntersection::new()`.
    pub fn new() -> Self {
        Self {
            empt: true,
            ..Self::default()
        }
    }

    /// `SetTolerances(TolArc, TolTang, UVMaxStep, Fleche)`: the arc and tangency
    /// tolerances are clamped to `[1e-8, 0.5]`, the deflection to `[1e-3, 10]` and the
    /// maximal parametric step to at most 0.5. `inter.set_tolerances(1e-7, 1e-7, 0.1, 0.01)`.
    pub fn set_tolerances(&mut self, tol_arc: f64, tol_tang: f64, uv_max_step: f64, fleche: f64) {
        self.my_tol_arc = clamp_tolerance(tol_arc);
        self.my_tol_tang = clamp_tolerance(tol_tang);
        self.my_uv_max_step = uv_max_step;
        self.my_fleche = fleche;
        self.my_fleche = self.my_fleche.clamp(1e-3, 10.0);
        if self.my_uv_max_step > 0.5 {
            self.my_uv_max_step = 0.5;
        }
    }

    /// `IsDone()`. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `IsEmpty()`: StdFail_NotDone unless done. `inter.is_empty()?`.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("IsEmpty"));
        }
        Ok(self.empt)
    }

    /// `TangentFaces()`: whether the surfaces are tangent along their whole domain;
    /// StdFail_NotDone unless done. `inter.tangent_faces()?`.
    pub fn tangent_faces(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("TangentFaces"));
        }
        Ok(self.tgte)
    }

    /// `OppositeFaces()`: whether tangent faces have opposite normals; StdFail_NotDone
    /// unless done, Standard_DomainError for faces that are not tangent.
    /// `inter.opposite_faces()?`.
    pub fn opposite_faces(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("OppositeFaces"));
        }
        if !self.tgte {
            return Err(Failure::with_message(
                "Standard_DomainError",
                "IntPatch_Intersection::OppositeFaces() - the faces are not tangent; expected TangentFaces()",
            ));
        }
        Ok(self.oppo)
    }

    /// `NbPnts()`: the isolated points; StdFail_NotDone unless done. `inter.nb_pnts()?`.
    pub fn nb_pnts(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("NbPnts"));
        }
        Ok(self.spnt.len() as i32)
    }

    /// `Point(Index)`: StdFail_NotDone unless done, Standard_OutOfRange outside
    /// `1..=NbPnts()`. `inter.point(1)?`.
    pub fn point(&self, index: i32) -> Result<IntPatchPoint, Failure> {
        if !self.done {
            return Err(not_done("Point"));
        }
        sequence_value(&self.spnt, index, "IntPatch_Intersection::Point").cloned()
    }

    /// `NbLines()`: StdFail_NotDone unless done. `inter.nb_lines()?`.
    pub fn nb_lines(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("NbLines"));
        }
        Ok(self.slin.len() as i32)
    }

    /// `Line(Index)`: StdFail_NotDone unless done, Standard_OutOfRange outside
    /// `1..=NbLines()`. `inter.line(1)?`.
    pub fn line(&self, index: i32) -> Result<Handle<dyn IntPatchLine>, Failure> {
        if !self.done {
            return Err(not_done("Line"));
        }
        sequence_value(&self.slin, index, "IntPatch_Intersection::Line").cloned()
    }

    /// `SequenceOfLine()`: the lines, done or not. `inter.sequence_of_line().len()`.
    pub fn sequence_of_line(&self) -> &[Handle<dyn IntPatchLine>] {
        &self.slin
    }

    /// `Dump(Mode, S1, D1, S2, D2)`: does nothing (its body is compiled only with
    /// `DUMPOFIntPatch_Intersection`, which the native build does not define).
    /// `inter.dump(0, &s1, &*d1, &s2, &*d2)`.
    pub fn dump(
        &self,
        _mode: i32,
        _s1: &Adaptor3dSurfaceHandle,
        _d1: &(dyn TheTopolTool + 'static),
        _s2: &Adaptor3dSurfaceHandle,
        _d2: &(dyn TheTopolTool + 'static),
    ) {
    }
}
