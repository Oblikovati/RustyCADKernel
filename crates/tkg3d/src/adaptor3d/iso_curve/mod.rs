// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_IsoCurve.hxx/.cxx (construction, Load, accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor3d_IsoCurve`: an isoparametric curve of an adapted surface.
mod curve;
mod kinds;
mod shape;
use super::surface::Adaptor3dSurface;
use crate::geom::errors::{no_such_object, null_object};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::IsoType;

/// The fields of one `Adaptor3d_IsoCurve`.
#[derive(Clone)]
pub(super) struct IsoState {
    pub(super) surface: Handle<dyn Adaptor3dSurface>,
    pub(super) iso: IsoType,
    pub(super) first: f64,
    pub(super) last: f64,
    pub(super) parameter: f64,
}
impl Default for IsoState {
    fn default() -> Self {
        Self {
            surface: Handle::default(),
            iso: IsoType::NoneIso,
            first: 0.0,
            last: 0.0,
            parameter: 0.0,
        }
    }
}
impl IsoState {
    /// The adapted surface; natively a null one is dereferenced; `state.surface()?`.
    pub(super) fn surface(&self) -> Result<&dyn Adaptor3dSurface, Failure> {
        self.surface.get().ok_or_else(|| {
            null_object("Adaptor3d_IsoCurve: no surface is loaded; expected Load(S) first")
        })
    }
}

/// The `NoneIso` failure of the iso-dependent members; `none_iso()`.
pub(super) fn none_iso() -> Failure {
    no_such_object("Adaptor3d_IsoCurve:NoneIso")
}

/// The U or V isoparametric curve of an adapted surface on `[first, last]`
/// (`Adaptor3d_IsoCurve`): an iso-U curve is parameterized by V at U = Parameter,
/// an iso-V curve by U; `Adaptor3dIsoCurve::with_iso(&surface, IsoType::IsoU, 0.5)?`.
#[derive(Default)]
pub struct Adaptor3dIsoCurve {
    parent: StandardTransient,
    state: SharedState<IsoState>,
}
impl_transient!(
    Adaptor3dIsoCurve,
    "Adaptor3d_IsoCurve",
    dyn crate::adaptor3d::Adaptor3dCurve,
    parent,
    interfaces: [dyn crate::adaptor3d::Adaptor3dCurve]
);

impl Adaptor3dIsoCurve {
    /// An iso curve on `surface` with no iso loaded (`Adaptor3d_IsoCurve(S)`);
    /// `Adaptor3dIsoCurve::with_surface(&surface)`.
    pub fn with_surface(surface: &Handle<dyn Adaptor3dSurface>) -> Self {
        Self::from_state(IsoState {
            surface: surface.clone(),
            ..IsoState::default()
        })
    }
    /// The iso `iso` of parameter `parameter` over the surface range
    /// (`Adaptor3d_IsoCurve(S, Iso, Param)`); Standard_NoSuchObject for `NoneIso`;
    /// `Adaptor3dIsoCurve::with_iso(&surface, IsoType::IsoV, 0.0)?`.
    pub fn with_iso(
        surface: &Handle<dyn Adaptor3dSurface>,
        iso: IsoType,
        parameter: f64,
    ) -> Result<Self, Failure> {
        let curve = Self::with_surface(surface);
        curve.load_iso(iso, parameter)?;
        Ok(curve)
    }
    /// The iso on `[w_first, w_last]` clipped to the surface range
    /// (`Adaptor3d_IsoCurve(S, Iso, Param, WFirst, WLast)`);
    /// `Adaptor3dIsoCurve::with_iso_range(&surface, IsoType::IsoU, 0.5, (0.0, 1.0))?`.
    pub fn with_iso_range(
        surface: &Handle<dyn Adaptor3dSurface>,
        iso: IsoType,
        parameter: f64,
        (w_first, w_last): (f64, f64),
    ) -> Result<Self, Failure> {
        let curve = Self::from_state(IsoState {
            surface: surface.clone(),
            iso,
            first: w_first,
            last: w_last,
            parameter,
        });
        curve.load_iso_range(iso, parameter, (w_first, w_last))?;
        Ok(curve)
    }
    fn from_state(state: IsoState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// A copy of the fields: members call the surface without holding this lock.
    pub(super) fn snapshot(&self) -> IsoState {
        self.state.snapshot()
    }
    /// Set the surface and forget the iso (`Load(S)`); `curve.load_surface(&surface)`.
    pub fn load_surface(&self, surface: &Handle<dyn Adaptor3dSurface>) {
        let mut state = self.state.write();
        state.surface = surface.clone();
        state.iso = IsoType::NoneIso;
    }
    /// Load the iso over the whole surface range (`Load(Iso, Param)`);
    /// Standard_NoSuchObject for `NoneIso`; `curve.load_iso(IsoType::IsoU, 0.5)?`.
    pub fn load_iso(&self, iso: IsoType, parameter: f64) -> Result<(), Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.state.read().surface.clone();
        let adapted: &dyn Adaptor3dSurface = surface
            .get()
            .ok_or_else(|| null_object("Adaptor3d_IsoCurve::Load: no surface is loaded"))?;
        let range: (f64, f64) = match iso {
            IsoType::IsoU => (adapted.first_v_parameter()?, adapted.last_v_parameter()?),
            IsoType::IsoV => (adapted.first_u_parameter()?, adapted.last_u_parameter()?),
            IsoType::NoneIso => return Err(none_iso()),
        };
        self.load_iso_range(iso, parameter, range)
    }
    /// Load the iso on `[w_first, w_last]`, clipped to the surface range; on a
    /// periodic surface the parameter (or the range) is brought into the first
    /// period (`Load(Iso, Param, WFirst, WLast)`). A `NoneIso` is clipped like an
    /// iso-V curve; `curve.load_iso_range(IsoType::IsoU, 0.5, (0.0, 1.0))?`.
    pub fn load_iso_range(
        &self,
        iso: IsoType,
        parameter: f64,
        (w_first, w_last): (f64, f64),
    ) -> Result<(), Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = {
            let mut state = self.state.write();
            (state.iso, state.parameter) = (iso, parameter);
            (state.first, state.last) = (w_first, w_last);
            state.surface.clone()
        };
        let adapted: &dyn Adaptor3dSurface = surface
            .get()
            .ok_or_else(|| null_object("Adaptor3d_IsoCurve::Load: no surface is loaded"))?;
        let clipped: (f64, f64) = clip(adapted, iso, (w_first, w_last))?;
        self.store_range(clipped);
        let (parameter, range) = adjust_u(adapted, iso, parameter, clipped)?;
        self.store_adjusted(parameter, range);
        let (parameter, range) = adjust_v(adapted, iso, parameter, range)?;
        self.store_adjusted(parameter, range);
        Ok(())
    }
    fn store_range(&self, (first, last): (f64, f64)) {
        let mut state = self.state.write();
        (state.first, state.last) = (first, last);
    }
    fn store_adjusted(&self, parameter: f64, range: (f64, f64)) {
        let mut state = self.state.write();
        state.parameter = parameter;
        (state.first, state.last) = range;
    }
    /// The adapted surface (`Surface`); `curve.surface()`.
    pub fn surface(&self) -> Handle<dyn Adaptor3dSurface> {
        self.state.read().surface.clone()
    }
    /// The iso direction (`Iso`); `curve.iso()`.
    pub fn iso(&self) -> IsoType {
        self.state.read().iso
    }
    /// The fixed parameter (`Parameter`); `curve.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.state.read().parameter
    }
}

// The range clipped to the surface range of the curve parameter (V for an iso-U
// curve, U otherwise).
fn clip(
    surface: &dyn Adaptor3dSurface,
    iso: IsoType,
    (first, last): (f64, f64),
) -> Result<(f64, f64), Failure> {
    if iso == IsoType::IsoU {
        return Ok((
            std_max(first, surface.first_v_parameter()?),
            std_min(last, surface.last_v_parameter()?),
        ));
    }
    Ok((
        std_max(first, surface.first_u_parameter()?),
        std_min(last, surface.last_u_parameter()?),
    ))
}

// ElCLib::AdjustPeriodic over the first U period: the parameter of an iso-U curve,
// the range otherwise (the native `dummy` second output is discarded).
fn adjust_u(
    surface: &dyn Adaptor3dSurface,
    iso: IsoType,
    parameter: f64,
    range: (f64, f64),
) -> Result<(f64, (f64, f64)), Failure> {
    if !surface.is_u_periodic()? {
        return Ok((parameter, range));
    }
    let first: f64 = surface.first_u_parameter()?;
    let period_end: f64 = surface.first_u_parameter()? + surface.u_period()?;
    let tolerance: f64 = surface.u_resolution(precision::confusion())?;
    if iso == IsoType::IsoU {
        let (adjusted, _) =
            elclib::adjust_periodic(first, period_end, tolerance, parameter, parameter);
        return Ok((adjusted, range));
    }
    let adjusted = elclib::adjust_periodic(first, period_end, tolerance, range.0, range.1);
    Ok((parameter, adjusted))
}

// The same in V: the parameter of an iso-V curve, the range otherwise.
fn adjust_v(
    surface: &dyn Adaptor3dSurface,
    iso: IsoType,
    parameter: f64,
    range: (f64, f64),
) -> Result<(f64, (f64, f64)), Failure> {
    if !surface.is_v_periodic()? {
        return Ok((parameter, range));
    }
    let first: f64 = surface.first_v_parameter()?;
    let period_end: f64 = surface.first_v_parameter()? + surface.v_period()?;
    let tolerance: f64 = surface.v_resolution(precision::confusion())?;
    if iso == IsoType::IsoV {
        let (adjusted, _) =
            elclib::adjust_periodic(first, period_end, tolerance, parameter, parameter);
        return Ok((adjusted, range));
    }
    let adjusted = elclib::adjust_periodic(first, period_end, tolerance, range.0, range.1);
    Ok((parameter, adjusted))
}
