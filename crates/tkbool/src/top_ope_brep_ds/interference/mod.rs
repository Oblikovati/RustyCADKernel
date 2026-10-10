// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_Interference.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_Interference`: the attachment of a new geometry (point, curve,
//! surface or shape) on a support of the data structure, with its transition. The
//! derived interferences embed it, so a handle of any of them casts to
//! `Handle<TopOpeBRepDSInterference>` (and back) as natively.
mod curve_point;
mod edge_vertex;
mod face_edge;
mod shape_shape;
mod solid_surface;
mod surface_curve;

pub use curve_point::TopOpeBRepDSCurvePointInterference;
pub use edge_vertex::TopOpeBRepDSEdgeVertexInterference;
pub use face_edge::TopOpeBRepDSFaceEdgeInterference;
pub use shape_shape::TopOpeBRepDSShapeShapeInterference;
pub use solid_surface::TopOpeBRepDSSolidSurfaceInterference;
pub use surface_curve::TopOpeBRepDSSurfaceCurveInterference;

use super::{TopOpeBRepDSKind, TopOpeBRepDSTransition};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};

/// The fields of an interference, shared by the handles of the data structure.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct InterferenceFields {
    transition: TopOpeBRepDSTransition,
    support: i32,
    geometry: i32,
    support_type: TopOpeBRepDSKind,
    geometry_type: TopOpeBRepDSKind,
}

/// An interference (`TopOpeBRepDS_Interference`); every member takes `&self`, the
/// interferences being shared and modified through handles;
/// `let i = TopOpeBRepDSInterference::new(&t, TopOpeBRepDSKind::Face, 1, TopOpeBRepDSKind::Curve, 2);`.
pub struct TopOpeBRepDSInterference {
    parent: StandardTransient,
    fields: SharedState<InterferenceFields>,
}
impl_transient!(
    TopOpeBRepDSInterference,
    "TopOpeBRepDS_Interference",
    StandardTransient,
    parent
);

impl Default for TopOpeBRepDSInterference {
    /// `TopOpeBRepDS_Interference()`: natively the indices and kinds are uninitialized;
    /// they are 0 and POINT here (the zeroed storage).
    fn default() -> Self {
        Self::with_fields(InterferenceFields::default())
    }
}

/// Standard_NullObject of a null interference handle (natively a null dereference).
pub(crate) fn null_interference(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{member}: null interference; expected an interference handle"),
        "TopOpeBRepDS_Interference",
    )
}

impl TopOpeBRepDSInterference {
    /// The interference of the fields.
    fn with_fields(fields: InterferenceFields) -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }

    /// `TopOpeBRepDS_Interference(Transition, SupportType, Support, GeometryType,
    /// Geometry)`; `TopOpeBRepDSInterference::new(&t, TopOpeBRepDSKind::Edge, 1, TopOpeBRepDSKind::Point, 1)`.
    pub fn new(
        t: &TopOpeBRepDSTransition,
        st: TopOpeBRepDSKind,
        s: i32,
        gt: TopOpeBRepDSKind,
        g: i32,
    ) -> Self {
        Self::with_fields(InterferenceFields {
            transition: *t,
            support: s,
            geometry: g,
            support_type: st,
            geometry_type: gt,
        })
    }

    /// `TopOpeBRepDS_Interference(I)`: a copy of the transition, support and geometry
    /// of `i`; Standard_NullObject for a null handle;
    /// `TopOpeBRepDSInterference::from_interference(&handle)?`.
    pub fn from_interference(i: &Handle<TopOpeBRepDSInterference>) -> Result<Self, Failure> {
        let other = i
            .get()
            .ok_or_else(|| null_interference("TopOpeBRepDS_Interference"))?;
        Ok(Self::with_fields(other.fields.snapshot()))
    }

    /// `Transition()`; `i.transition()`.
    pub fn transition(&self) -> TopOpeBRepDSTransition {
        self.fields.read().transition
    }

    /// `ChangeTransition()`: the transition modified in place by `f`;
    /// `i.change_transition(|t| t.set_index(2))`.
    pub fn change_transition<R>(&self, f: impl FnOnce(&mut TopOpeBRepDSTransition) -> R) -> R {
        f(&mut self.fields.write().transition)
    }

    /// `Transition(T)`; `i.set_transition(&t)`.
    pub fn set_transition(&self, t: &TopOpeBRepDSTransition) {
        self.fields.write().transition = *t;
    }

    /// `GKGSKS(GK, G, SK, S)`: the geometry kind and index, then the support kind and
    /// index; `let (gk, g, sk, s) = i.gkgsks();`.
    pub fn gkgsks(&self) -> (TopOpeBRepDSKind, i32, TopOpeBRepDSKind, i32) {
        let f = self.fields.read();
        (f.geometry_type, f.geometry, f.support_type, f.support)
    }

    /// `SupportType()`; `i.support_type()`.
    pub fn support_type(&self) -> TopOpeBRepDSKind {
        self.fields.read().support_type
    }

    /// `Support()`; `i.support()`.
    pub fn support(&self) -> i32 {
        self.fields.read().support
    }

    /// `GeometryType()`; `i.geometry_type()`.
    pub fn geometry_type(&self) -> TopOpeBRepDSKind {
        self.fields.read().geometry_type
    }

    /// `Geometry()`; `i.geometry()`.
    pub fn geometry(&self) -> i32 {
        self.fields.read().geometry
    }

    /// `SetGeometry(GI)` (the same as `Geometry(G)`); `i.set_geometry_index(3)`.
    pub fn set_geometry_index(&self, gi: i32) {
        self.fields.write().geometry = gi;
    }

    /// `SupportType(ST)`; `i.set_support_type(TopOpeBRepDSKind::Face)`.
    pub fn set_support_type(&self, st: TopOpeBRepDSKind) {
        self.fields.write().support_type = st;
    }

    /// `Support(S)`; `i.set_support(2)`.
    pub fn set_support(&self, s: i32) {
        self.fields.write().support = s;
    }

    /// `GeometryType(GT)`; `i.set_geometry_type(TopOpeBRepDSKind::Curve)`.
    pub fn set_geometry_type(&self, gt: TopOpeBRepDSKind) {
        self.fields.write().geometry_type = gt;
    }

    /// `Geometry(G)`; `i.set_geometry(3)`.
    pub fn set_geometry(&self, g: i32) {
        self.fields.write().geometry = g;
    }

    /// `HasSameSupport(Other)`: same support kind and index; Standard_NullObject for a
    /// null handle; `i.has_same_support(&other)?`.
    pub fn has_same_support(
        &self,
        other: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<bool, Failure> {
        let o = other
            .get()
            .ok_or_else(|| null_interference("TopOpeBRepDS_Interference::HasSameSupport"))?;
        Ok(self.support_type() == o.support_type() && self.support() == o.support())
    }

    /// `HasSameGeometry(Other)`: same geometry kind and index; Standard_NullObject for a
    /// null handle; `i.has_same_geometry(&other)?`.
    pub fn has_same_geometry(
        &self,
        other: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<bool, Failure> {
        let o = other
            .get()
            .ok_or_else(|| null_interference("TopOpeBRepDS_Interference::HasSameGeometry"))?;
        Ok(self.geometry_type() == o.geometry_type() && self.geometry() == o.geometry())
    }
}
