// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_TrihedronWithGuide.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_TrihedronWithGuide`: a trihedron law steered by a guide curve.
use super::trihedron_law::GeomFillTrihedronLaw;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;

/// The protected fields of `GeomFill_TrihedronWithGuide`.
#[derive(Clone, Default)]
pub struct GuideFields {
    /// `myGuide`.
    pub guide: Handle<dyn Adaptor3dCurve>,
    /// `myTrimG`.
    pub trim_g: Handle<dyn Adaptor3dCurve>,
    /// `myCurPointOnGuide`.
    pub cur_point_on_guide: Point3,
}

/// The protected data of `GeomFill_TrihedronWithGuide`, which each guided law
/// embeds; `GeomFillTrihedronWithGuideBase::default()`.
#[derive(Default)]
pub struct GeomFillTrihedronWithGuideBase {
    fields: SharedState<GuideFields>,
}

impl GeomFillTrihedronWithGuideBase {
    /// A copy of the protected fields; `base.fields().guide`.
    pub fn fields(&self) -> GuideFields {
        self.fields.read().clone()
    }

    /// Edits the protected fields under the lock; `base.edit(|f| f.guide = g)`.
    pub fn edit<R>(&self, change: impl FnOnce(&mut GuideFields) -> R) -> R {
        change(&mut self.fields.write())
    }
}

/// `GeomFill_TrihedronWithGuide`: reached through
/// `Handle<dyn GeomFillTrihedronWithGuide>`, a trihedron law as well
/// (docs/geomalgo-design.md, section 14);
/// `law.guide()` returns the guide curve.
pub trait GeomFillTrihedronWithGuide: GeomFillTrihedronLaw {
    /// The protected `myGuide`, `myTrimG`, `myCurPointOnGuide`.
    fn guide_base(&self) -> &GeomFillTrihedronWithGuideBase;

    /// `Guide()`: the base returns `myGuide`.
    fn guide(&self) -> Handle<dyn Adaptor3dCurve> {
        self.guide_base().fields().guide
    }

    /// `Origine(Param1, Param2)`: the parameters that match the path and the guide.
    fn origine(&self, param1: f64, param2: f64) -> Result<(), Failure>;

    /// `CurrentPointOnGuide()` (non-virtual): the guide point of the last evaluation.
    fn current_point_on_guide(&self) -> Point3 {
        self.guide_base().fields().cur_point_on_guide
    }
}
impl_transient_interface!(
    dyn GeomFillTrihedronWithGuide,
    "GeomFill_TrihedronWithGuide",
    dyn GeomFillTrihedronLaw
);
