// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_RepCurveDesc.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_RepCurveDesc`: descriptors that redirect the evaluation of a curve
//! to another curve (its "evaluation representation").
//!
//! The native descriptors expose public mutable fields through shared handles;
//! here each descriptor keeps them behind a lock with getters and setters.
use crate::geom2d::Geom2dCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};

/// Affine parameter map `uRep = Scale * u + Offset` (`Map1d`); `Map1d::default().map(2.0) == 2.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Map1d {
    /// Scale factor.
    pub scale: f64,
    /// Offset.
    pub offset: f64,
}
impl Default for Map1d {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: 0.0,
        }
    }
}
impl Map1d {
    /// Identity within Precision::Confusion (`IsIdentity`); `Map1d::default().is_identity()`.
    pub fn is_identity(&self) -> bool {
        (self.scale - 1.0).abs() < precision::confusion()
            && self.offset.abs() < precision::confusion()
    }
    /// Scale not null within Precision::Confusion (`IsValid`); `Map1d::default().is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.scale.abs() > precision::confusion()
    }
    /// `Scale * u + Offset` (`Map`); `map.map(0.5)`.
    pub fn map(&self, u: f64) -> f64 {
        self.scale * u + self.offset
    }
}

/// Parameter interval (`Domain1d`); `Domain1d::default().contains(0.5)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Domain1d {
    /// First parameter.
    pub first: f64,
    /// Last parameter.
    pub last: f64,
}
impl Default for Domain1d {
    fn default() -> Self {
        Self {
            first: 0.0,
            last: 1.0,
        }
    }
}
impl Domain1d {
    /// Whether `u` lies in the interval widened by Precision::Confusion (`Contains`);
    /// `domain.contains(1.0)`.
    pub fn contains(&self, u: f64) -> bool {
        (u >= self.first - precision::confusion()) && (u <= self.last + precision::confusion())
    }
}

/// Descriptor kind used for dispatch (`Base::Kind`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepCurveKind {
    /// Fully equivalent: no derivative limit, no domain, no map.
    Full,
    /// Full domain and identity map, derivatives limited to `MaxDerivOrder`.
    DerivBounded,
    /// Derivative limit, optional domain and affine parameter map.
    Mapped,
}

/// Abstract descriptor (`Geom2dEval_RepCurveDesc::Base`); `desc.kind()`.
pub trait RepCurveDesc: Transient {
    /// The descriptor kind (`GetKind`); `desc.kind()`.
    fn kind(&self) -> RepCurveKind;
    /// The curve used for evaluation (`Representation`); `desc.representation()`.
    fn representation(&self) -> Handle<dyn Geom2dCurve>;
    /// Replace the curve used for evaluation; `desc.set_representation(curve)`.
    fn set_representation(&self, curve: Handle<dyn Geom2dCurve>);
    /// Derivative limit (`MaxDerivOrder`), `None` for a full descriptor; `desc.max_deriv_order()`.
    fn max_deriv_order(&self) -> Option<usize>;
    /// Domain and parameter map of a mapped descriptor; `desc.mapping()`.
    fn mapping(&self) -> Option<(Option<Domain1d>, Map1d)>;
}
impl_transient_interface!(dyn RepCurveDesc, "Base", StandardTransient);

/// `Geom2dEval_RepCurveDesc::Full`; `Handle::new(RepCurveDescFull::new(curve))`.
pub struct RepCurveDescFull {
    parent: StandardTransient,
    representation: SharedState<Handle<dyn Geom2dCurve>>,
}
impl_transient!(RepCurveDescFull, "Full", dyn RepCurveDesc, parent, interfaces: [dyn RepCurveDesc]);
impl RepCurveDescFull {
    /// Descriptor evaluating through `representation`; `RepCurveDescFull::new(curve)`.
    pub fn new(representation: Handle<dyn Geom2dCurve>) -> Self {
        Self {
            parent: StandardTransient::default(),
            representation: SharedState::new(representation),
        }
    }
}
impl RepCurveDesc for RepCurveDescFull {
    fn kind(&self) -> RepCurveKind {
        RepCurveKind::Full
    }
    fn representation(&self) -> Handle<dyn Geom2dCurve> {
        self.representation.read().clone()
    }
    fn set_representation(&self, curve: Handle<dyn Geom2dCurve>) {
        *self.representation.write() = curve;
    }
    fn max_deriv_order(&self) -> Option<usize> {
        None
    }
    fn mapping(&self) -> Option<(Option<Domain1d>, Map1d)> {
        None
    }
}

struct BoundedState {
    representation: Handle<dyn Geom2dCurve>,
    max_deriv_order: usize,
}

/// `Geom2dEval_RepCurveDesc::DerivBounded` (default `MaxDerivOrder` 3);
/// `Handle::new(RepCurveDescDerivBounded::new(curve, 2))`.
pub struct RepCurveDescDerivBounded {
    parent: StandardTransient,
    state: SharedState<BoundedState>,
}
impl_transient!(
    RepCurveDescDerivBounded,
    "DerivBounded",
    dyn RepCurveDesc,
    parent,
    interfaces: [dyn RepCurveDesc]
);
impl RepCurveDescDerivBounded {
    /// Descriptor limited to `max_deriv_order`; `RepCurveDescDerivBounded::new(curve, 3)`.
    pub fn new(representation: Handle<dyn Geom2dCurve>, max_deriv_order: usize) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(BoundedState {
                representation,
                max_deriv_order,
            }),
        }
    }
    /// Change the derivative limit; `desc.set_max_deriv_order(2)`.
    pub fn set_max_deriv_order(&self, order: usize) {
        self.state.write().max_deriv_order = order;
    }
}
impl RepCurveDesc for RepCurveDescDerivBounded {
    fn kind(&self) -> RepCurveKind {
        RepCurveKind::DerivBounded
    }
    fn representation(&self) -> Handle<dyn Geom2dCurve> {
        self.state.read().representation.clone()
    }
    fn set_representation(&self, curve: Handle<dyn Geom2dCurve>) {
        self.state.write().representation = curve;
    }
    fn max_deriv_order(&self) -> Option<usize> {
        Some(self.state.read().max_deriv_order)
    }
    fn mapping(&self) -> Option<(Option<Domain1d>, Map1d)> {
        None
    }
}

struct MappedState {
    representation: Handle<dyn Geom2dCurve>,
    max_deriv_order: usize,
    domain: Option<Domain1d>,
    param_map: Map1d,
}

/// `Geom2dEval_RepCurveDesc::Mapped`: optional bounded domain and affine parameter map;
/// `Handle::new(RepCurveDescMapped::new(curve, 3, None, Map1d::default()))`.
pub struct RepCurveDescMapped {
    parent: StandardTransient,
    state: SharedState<MappedState>,
}
impl_transient!(
    RepCurveDescMapped,
    "Mapped",
    dyn RepCurveDesc,
    parent,
    interfaces: [dyn RepCurveDesc]
);
impl RepCurveDescMapped {
    /// Descriptor with a domain (`None` = full) and a parameter map;
    /// `RepCurveDescMapped::new(curve, 3, None, Map1d { scale: 2.0, offset: 0.0 })`.
    pub fn new(
        representation: Handle<dyn Geom2dCurve>,
        max_deriv_order: usize,
        domain: Option<Domain1d>,
        param_map: Map1d,
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
    /// Change the domain; `desc.set_domain(Some(Domain1d::default()))`.
    pub fn set_domain(&self, domain: Option<Domain1d>) {
        self.state.write().domain = domain;
    }
    /// Change the parameter map; `desc.set_param_map(Map1d::default())`.
    pub fn set_param_map(&self, param_map: Map1d) {
        self.state.write().param_map = param_map;
    }
}
impl RepCurveDesc for RepCurveDescMapped {
    fn kind(&self) -> RepCurveKind {
        RepCurveKind::Mapped
    }
    fn representation(&self) -> Handle<dyn Geom2dCurve> {
        self.state.read().representation.clone()
    }
    fn set_representation(&self, curve: Handle<dyn Geom2dCurve>) {
        self.state.write().representation = curve;
    }
    fn max_deriv_order(&self) -> Option<usize> {
        Some(self.state.read().max_deriv_order)
    }
    fn mapping(&self) -> Option<(Option<Domain1d>, Map1d)> {
        let state = self.state.read();
        Some((state.domain, state.param_map))
    }
}
