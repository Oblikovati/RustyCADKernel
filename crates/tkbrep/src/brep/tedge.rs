// Rust translation of ModelingData/TKBRep/BRep/BRep_TEdge.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_TEdge`: the edge TShape with a tolerance, flags and curve representations.
use super::curve_on_2_surfaces::BRepCurveOn2Surfaces;
use super::curve_representation::BRepCurveRepresentation;
use super::dump::{dump_base, dump_class_name, dump_field};
use super::gcurve::BRepGCurve;
use crate::topods::{self, TShape, TShapeBase, dump_tshape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::real::real_epsilon;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, SharedState};
use std::io;

/// `ParameterMask`, `RangeMask` and `DegeneratedMask` of `myFlags`.
const PARAMETER_MASK: i32 = 1;
const RANGE_MASK: i32 = 2;
const DEGENERATED_MASK: i32 = 4;

/// The private fields of `BRep_TEdge`.
#[derive(Clone)]
struct TEdgeFields {
    tolerance: f64,
    flags: i32,
    curves: Vec<Handle<dyn BRepCurveRepresentation>>,
}

impl TEdgeFields {
    fn set_flag(&mut self, mask: i32, on: bool) {
        if on {
            self.flags |= mask;
        } else {
            self.flags &= !mask;
        }
    }
}

/// `BRep_TEdge`: an edge with its tolerance (`RealEpsilon()` when new), its
/// SameParameter, SameRange and Degenerated flags (the first two set when new) and
/// its list of curve representations.
///
/// ```
/// use rustycad_tkbrep::brep::BRepTEdge;
/// let edge = BRepTEdge::new();
/// assert!(edge.same_parameter() && edge.same_range() && !edge.degenerated());
/// ```
pub struct BRepTEdge {
    parent: topods::TEdge,
    fields: SharedState<TEdgeFields>,
}
impl_transient!(BRepTEdge, "BRep_TEdge", topods::TEdge, parent, interfaces: [dyn TShape]);

impl BRepTEdge {
    /// `BRep_TEdge()`: no representation, SameParameter and SameRange set; `BRepTEdge::new()`.
    pub fn new() -> Self {
        let mut fields = TEdgeFields {
            tolerance: real_epsilon(),
            flags: 0,
            curves: Vec::new(),
        };
        fields.set_flag(PARAMETER_MASK, true);
        fields.set_flag(RANGE_MASK, true);
        Self {
            parent: topods::TEdge::new(),
            fields: SharedState::new(fields),
        }
    }
    /// `Tolerance()`; `edge.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.fields.read().tolerance
    }
    /// `Tolerance(T)`; `edge.set_tolerance(1e-7)`.
    pub fn set_tolerance(&self, tolerance: f64) {
        self.fields.write().tolerance = tolerance;
    }
    /// `UpdateTolerance(T)`: keeps the larger tolerance; `edge.update_tolerance(1e-7)`.
    pub fn update_tolerance(&self, tolerance: f64) {
        let mut fields = self.fields.write();
        if tolerance > fields.tolerance {
            fields.tolerance = tolerance;
        }
    }
    /// `SameParameter()`; `edge.same_parameter()`.
    pub fn same_parameter(&self) -> bool {
        self.fields.read().flags & PARAMETER_MASK != 0
    }
    /// `SameParameter(S)`; `edge.set_same_parameter(false)`.
    pub fn set_same_parameter(&self, on: bool) {
        self.fields.write().set_flag(PARAMETER_MASK, on);
    }
    /// `SameRange()`; `edge.same_range()`.
    pub fn same_range(&self) -> bool {
        self.fields.read().flags & RANGE_MASK != 0
    }
    /// `SameRange(S)`; `edge.set_same_range(false)`.
    pub fn set_same_range(&self, on: bool) {
        self.fields.write().set_flag(RANGE_MASK, on);
    }
    /// `Degenerated()`; `edge.degenerated()`.
    pub fn degenerated(&self) -> bool {
        self.fields.read().flags & DEGENERATED_MASK != 0
    }
    /// `Degenerated(S)`; `edge.set_degenerated(true)`.
    pub fn set_degenerated(&self, on: bool) {
        self.fields.write().set_flag(DEGENERATED_MASK, on);
    }
    /// `Curves()`: a copy of the list of curve representations; `edge.curves()`.
    pub fn curves(&self) -> Vec<Handle<dyn BRepCurveRepresentation>> {
        self.fields.read().curves.clone()
    }
    /// `ChangeCurves()`: lends the stored list to `edit`, which may call the
    /// representations but not this edge; `edge.change_curves(|curves| curves.clear())`.
    pub fn change_curves<R>(
        &self,
        edit: impl FnOnce(&mut Vec<Handle<dyn BRepCurveRepresentation>>) -> R,
    ) -> R {
        edit(&mut self.fields.write().curves)
    }
}

impl Default for BRepTEdge {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether `EmptyCopy` copies the representation: the parametric curves and the
/// regularities, never the polygons.
fn is_copied_by_empty_copy(representation: &Handle<dyn BRepCurveRepresentation>) -> bool {
    representation.is_kind(<dyn BRepGCurve>::static_type())
        || representation.is_kind(BRepCurveOn2Surfaces::static_type())
}

impl TShape for BRepTEdge {
    fn tshape_base(&self) -> &TShapeBase {
        TShape::tshape_base(&self.parent)
    }
    /// A new edge with the same tolerance and flags and copies of the parametric
    /// curves and regularities (the polygons are not copied).
    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
        let copy = Self::new();
        copy.set_tolerance(self.tolerance());
        let mut copied = Vec::new();
        for representation in self.curves().iter().filter(|r| is_copied_by_empty_copy(r)) {
            if let Some(representation) = representation.get() {
                copied.push(representation.copy()?);
            }
        }
        copy.change_curves(|curves| curves.extend(copied));
        copy.set_degenerated(self.degenerated());
        copy.set_same_parameter(self.same_parameter());
        copy.set_same_range(self.same_range());
        Ok(Handle::new(copy).into_cast())
    }
    /// The class name, the `TopoDS_TShape` dump, the tolerance, the flags and each
    /// curve representation.
    fn dump_json(&self, output: &mut DumpStream, depth: i32) -> io::Result<()> {
        let address = std::ptr::from_ref(self).cast::<()>() as usize;
        dump_class_name(output, "BRep_TEdge")?;
        dump_base(output, depth, |nested, _| {
            dump_tshape(self.tshape_base(), address, nested)
        })?;
        let fields = self.fields.read().clone();
        dump::dump_real(output, "Tolerance", fields.tolerance)?;
        dump::dump_integer(output, "Flags", fields.flags)?;
        for curve in &fields.curves {
            let Some(curve) = curve.get() else { continue };
            dump_field(output, depth, "CurveRepresentation", |nested, d| {
                curve.dump_json(nested, d)
            })?;
        }
        Ok(())
    }
}
