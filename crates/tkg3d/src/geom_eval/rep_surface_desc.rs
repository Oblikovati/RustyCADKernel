// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_RepSurfaceDesc.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_RepSurfaceDesc`: descriptors that redirect the evaluation of a
//! surface to another surface (its "evaluation representation").
//!
//! The native descriptors expose public mutable fields through shared handles;
//! here each descriptor keeps them behind a lock with getters and setters.
use crate::geom::GeomSurface;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Handle, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};

/// Descriptor kind used for dispatch (`Base::Kind`), the same enumeration as the
/// curve descriptors.
pub type RepSurfaceKind = rustycad_tkg2d::geom2d_eval::RepCurveKind;

/// Affine parameter map with an optional U/V swap (`Map2d`);
/// `Map2d::default().map(1.0, 2.0) == (1.0, 2.0)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Map2d {
    /// Scale of the representation's U.
    pub scale_u: f64,
    /// Offset of the representation's U.
    pub offset_u: f64,
    /// Scale of the representation's V.
    pub scale_v: f64,
    /// Offset of the representation's V.
    pub offset_v: f64,
    /// Whether the representation's U is the owner's V and conversely.
    pub swap_uv: bool,
}
impl Default for Map2d {
    fn default() -> Self {
        Self {
            scale_u: 1.0,
            offset_u: 0.0,
            scale_v: 1.0,
            offset_v: 0.0,
            swap_uv: false,
        }
    }
}
impl Map2d {
    /// Identity within Precision::Confusion and no swap (`IsIdentity`); `Map2d::default().is_identity()`.
    pub fn is_identity(&self) -> bool {
        (self.scale_u - 1.0).abs() < precision::confusion()
            && (self.scale_v - 1.0).abs() < precision::confusion()
            && self.offset_u.abs() < precision::confusion()
            && self.offset_v.abs() < precision::confusion()
            && !self.swap_uv
    }
    /// Both scales not null within Precision::Confusion (`IsValid`); `Map2d::default().is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.scale_u.abs() > precision::confusion() && self.scale_v.abs() > precision::confusion()
    }
    /// The representation's parameters of `(u, v)` (`Map`); `map.map(0.5, 0.5)`.
    pub fn map(&self, u: f64, v: f64) -> (f64, f64) {
        if self.swap_uv {
            return (
                self.scale_u * v + self.offset_u,
                self.scale_v * u + self.offset_v,
            );
        }
        (
            self.scale_u * u + self.offset_u,
            self.scale_v * v + self.offset_v,
        )
    }
}

/// Parameter rectangle (`Domain2d`); `Domain2d::default().contains(0.5, 0.5)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Domain2d {
    /// First U parameter.
    pub u_first: f64,
    /// Last U parameter.
    pub u_last: f64,
    /// First V parameter.
    pub v_first: f64,
    /// Last V parameter.
    pub v_last: f64,
}
impl Default for Domain2d {
    fn default() -> Self {
        Self {
            u_first: 0.0,
            u_last: 1.0,
            v_first: 0.0,
            v_last: 1.0,
        }
    }
}
impl Domain2d {
    /// Whether `(u, v)` lies in the rectangle widened by Precision::Confusion
    /// (`Contains`); `domain.contains(1.0, 0.0)`.
    pub fn contains(&self, u: f64, v: f64) -> bool {
        (u >= self.u_first - precision::confusion())
            && (u <= self.u_last + precision::confusion())
            && (v >= self.v_first - precision::confusion())
            && (v <= self.v_last + precision::confusion())
    }
}

/// Abstract descriptor (`GeomEval_RepSurfaceDesc::Base`); `desc.kind()`.
pub trait RepSurfaceDesc: Transient {
    /// The descriptor kind (`GetKind`); `desc.kind()`.
    fn kind(&self) -> RepSurfaceKind;
    /// The surface used for evaluation (`Representation`); `desc.representation()`.
    fn representation(&self) -> Handle<dyn GeomSurface>;
    /// Replace the surface used for evaluation; `desc.set_representation(surface)`.
    fn set_representation(&self, surface: Handle<dyn GeomSurface>);
    /// Derivative limit (`MaxDerivOrder`), `None` for a full descriptor; `desc.max_deriv_order()`.
    fn max_deriv_order(&self) -> Option<usize>;
    /// Domain and parameter map of a mapped descriptor; `desc.mapping()`.
    fn mapping(&self) -> Option<(Option<Domain2d>, Map2d)>;
}
impl_transient_interface!(dyn RepSurfaceDesc, "Base", StandardTransient);

/// `GeomEval_RepSurfaceDesc::Full`; `Handle::new(RepSurfaceDescFull::new(surface))`.
pub struct RepSurfaceDescFull {
    parent: StandardTransient,
    representation: SharedState<Handle<dyn GeomSurface>>,
}
impl_transient!(RepSurfaceDescFull, "Full", dyn RepSurfaceDesc, parent, interfaces: [dyn RepSurfaceDesc]);
impl RepSurfaceDescFull {
    /// Descriptor evaluating through `representation`; `RepSurfaceDescFull::new(surface)`.
    pub fn new(representation: Handle<dyn GeomSurface>) -> Self {
        Self {
            parent: StandardTransient::default(),
            representation: SharedState::new(representation),
        }
    }
}
impl RepSurfaceDesc for RepSurfaceDescFull {
    fn kind(&self) -> RepSurfaceKind {
        RepSurfaceKind::Full
    }
    fn representation(&self) -> Handle<dyn GeomSurface> {
        self.representation.read().clone()
    }
    fn set_representation(&self, surface: Handle<dyn GeomSurface>) {
        *self.representation.write() = surface;
    }
    fn max_deriv_order(&self) -> Option<usize> {
        None
    }
    fn mapping(&self) -> Option<(Option<Domain2d>, Map2d)> {
        None
    }
}

/// `GeomEval_RepSurfaceDesc::DerivBounded` (default `MaxDerivOrder` 3);
/// `Handle::new(RepSurfaceDescDerivBounded::new(surface, 2))`.
pub struct RepSurfaceDescDerivBounded {
    parent: StandardTransient,
    state: SharedState<(Handle<dyn GeomSurface>, usize)>,
}
impl_transient!(
    RepSurfaceDescDerivBounded,
    "DerivBounded",
    dyn RepSurfaceDesc,
    parent,
    interfaces: [dyn RepSurfaceDesc]
);
impl RepSurfaceDescDerivBounded {
    /// Descriptor limited to `max_deriv_order`; `RepSurfaceDescDerivBounded::new(surface, 3)`.
    pub fn new(representation: Handle<dyn GeomSurface>, max_deriv_order: usize) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new((representation, max_deriv_order)),
        }
    }
    /// Change the derivative limit; `desc.set_max_deriv_order(2)`.
    pub fn set_max_deriv_order(&self, order: usize) {
        self.state.write().1 = order;
    }
}
impl RepSurfaceDesc for RepSurfaceDescDerivBounded {
    fn kind(&self) -> RepSurfaceKind {
        RepSurfaceKind::DerivBounded
    }
    fn representation(&self) -> Handle<dyn GeomSurface> {
        self.state.read().0.clone()
    }
    fn set_representation(&self, surface: Handle<dyn GeomSurface>) {
        self.state.write().0 = surface;
    }
    fn max_deriv_order(&self) -> Option<usize> {
        Some(self.state.read().1)
    }
    fn mapping(&self) -> Option<(Option<Domain2d>, Map2d)> {
        None
    }
}

struct MappedState {
    representation: Handle<dyn GeomSurface>,
    max_deriv_order: usize,
    domain: Option<Domain2d>,
    param_map: Map2d,
}

/// `GeomEval_RepSurfaceDesc::Mapped`: optional bounded domain and parameter map;
/// `Handle::new(RepSurfaceDescMapped::new(surface, 3, None, Map2d::default()))`.
pub struct RepSurfaceDescMapped {
    parent: StandardTransient,
    state: SharedState<MappedState>,
}
impl_transient!(
    RepSurfaceDescMapped,
    "Mapped",
    dyn RepSurfaceDesc,
    parent,
    interfaces: [dyn RepSurfaceDesc]
);
impl RepSurfaceDescMapped {
    /// Descriptor with a domain (`None` = full) and a parameter map;
    /// `RepSurfaceDescMapped::new(surface, 3, None, Map2d::default())`.
    pub fn new(
        representation: Handle<dyn GeomSurface>,
        max_deriv_order: usize,
        domain: Option<Domain2d>,
        param_map: Map2d,
    ) -> Self {
        let state = MappedState {
            representation,
            max_deriv_order,
            domain,
            param_map,
        };
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Change the derivative limit; `desc.set_max_deriv_order(2)`.
    pub fn set_max_deriv_order(&self, order: usize) {
        self.state.write().max_deriv_order = order;
    }
    /// Change the domain; `desc.set_domain(Some(Domain2d::default()))`.
    pub fn set_domain(&self, domain: Option<Domain2d>) {
        self.state.write().domain = domain;
    }
    /// Change the parameter map; `desc.set_param_map(Map2d::default())`.
    pub fn set_param_map(&self, param_map: Map2d) {
        self.state.write().param_map = param_map;
    }
}
impl RepSurfaceDesc for RepSurfaceDescMapped {
    fn kind(&self) -> RepSurfaceKind {
        RepSurfaceKind::Mapped
    }
    fn representation(&self) -> Handle<dyn GeomSurface> {
        self.state.read().representation.clone()
    }
    fn set_representation(&self, surface: Handle<dyn GeomSurface>) {
        self.state.write().representation = surface;
    }
    fn max_deriv_order(&self) -> Option<usize> {
        Some(self.state.read().max_deriv_order)
    }
    fn mapping(&self) -> Option<(Option<Domain2d>, Map2d)> {
        let state = self.state.read();
        Some((state.domain, state.param_map))
    }
}
