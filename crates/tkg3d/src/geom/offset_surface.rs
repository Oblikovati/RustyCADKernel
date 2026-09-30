// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_OffsetSurface`: the surface at a constant distance along the normal of a
//! basis surface, with an equivalent elementary surface as evaluation
//! representation when there is one, and an osculating surface for B-spline and
//! Bezier bases.
mod basis;
mod equivalent;
mod iso;
mod traits;
use super::errors;
use super::offset_surface_utils::OsculatingQuery;
use super::osculating_surface::GeomOsculatingSurface;
use super::surface::GeomSurface;
use crate::geom::GeomBSplineSurface;
use crate::geom::geometry::GeomGeometry;
use crate::geom_eval::{RepSurfaceDesc, RepSurfaceDescFull, RepSurfaceKind, rep_utils};
use basis::{BasisResolution, resolve_basis};
pub use equivalent::equivalent_surface;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::Shape;

#[derive(Clone)]
struct OffsetSurfaceState {
    basis: Handle<dyn GeomSurface>,
    eval_rep: Handle<dyn RepSurfaceDesc>,
    offset_value: f64,
    osc: Option<GeomOsculatingSurface>,
    basis_continuity: Shape,
}

/// An offset surface; the basis is a private copy (unwrapped from trimmed and
/// offset surfaces); `Handle::new(GeomOffsetSurface::new(&plane.cast(), 3.0, false)?)`.
pub struct GeomOffsetSurface {
    parent: StandardTransient,
    state: SharedState<OffsetSurfaceState>,
}
impl_transient!(
    GeomOffsetSurface,
    "Geom_OffsetSurface",
    dyn GeomSurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface]
);

// makeFullSurfaceRep: a Full descriptor on `surface`, null for a null surface.
fn full_rep(surface: Handle<dyn GeomSurface>) -> Handle<dyn RepSurfaceDesc> {
    if surface.is_null() {
        return Handle::default();
    }
    Handle::new(RepSurfaceDescFull::new(surface)).cast()
}

impl GeomOffsetSurface {
    /// Offset of `surface` by `offset`; trimmed and offset inputs are unwrapped (the
    /// offsets add up), a C0 basis must be G1 through its iso curves unless
    /// `not_check_c0`, an equivalent elementary surface becomes the evaluation
    /// representation and a B-spline or Bezier basis gets an osculating surface
    /// (`Geom_OffsetSurface(S, Offset, isNotCheckC0)`);
    /// `GeomOffsetSurface::new(&surface, 3.0, false)?`.
    pub fn new(
        surface: &Handle<dyn GeomSurface>,
        offset: f64,
        not_check_c0: bool,
    ) -> Result<Self, Failure> {
        let this = Self {
            parent: StandardTransient::default(),
            state: SharedState::new(OffsetSurfaceState {
                basis: Handle::default(),
                eval_rep: Handle::default(),
                offset_value: offset,
                osc: None,
                basis_continuity: Shape::C0,
            }),
        };
        this.set_basis_surface(surface, not_check_c0)?;
        Ok(this)
    }
    /// The native copy constructor: basis, evaluation representation and osculating
    /// surface are deep-copied (`Geom_OffsetSurface(const Geom_OffsetSurface&)`);
    /// `GeomOffsetSurface::copy_of(&s)?`.
    pub fn copy_of(other: &GeomOffsetSurface) -> Result<Self, Failure> {
        let source = other.state.read();
        let state = OffsetSurfaceState {
            basis: source.basis.copy()?.cast(),
            eval_rep: rep_utils::clone_surface_desc(&source.eval_rep)?,
            offset_value: source.offset_value,
            osc: source.osc.clone(),
            basis_continuity: source.basis_continuity,
        };
        Ok(Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }
    /// Replace the basis, as the constructor does (`SetBasisSurface`). Like the native
    /// member, a rejected basis leaves the summed offset and the continuity stored and
    /// the evaluation representation cleared, and a basis that is neither B-spline nor
    /// Bezier keeps the previous osculating surface; `s.set_basis_surface(&basis, false)?`.
    pub fn set_basis_surface(
        &self,
        surface: &Handle<dyn GeomSurface>,
        not_check_c0: bool,
    ) -> Result<(), Failure> {
        self.clear_eval_representation();
        let mut resolved = BasisResolution {
            offset: self.offset(),
            continuity: None,
        };
        let result = resolve_basis(surface, not_check_c0, &mut resolved);
        {
            let mut state = self.state.write();
            state.offset_value = resolved.offset;
            if let Some(continuity) = resolved.continuity {
                state.basis_continuity = continuity;
            }
        }
        let (basis, checking) = result?;
        self.state.write().basis = basis;
        self.set_eval_representation(full_rep(self.surface()?))?;
        if is_spline_surface(&checking) {
            let osc =
                GeomOsculatingSurface::new(&checking, rustycad_tkernel::precision::confusion())?;
            self.state.write().osc = Some(osc);
        }
        Ok(())
    }
    /// Change the offset distance and rebuild the equivalent representation
    /// (`SetOffsetValue`); `s.set_offset_value(2.0)?`.
    pub fn set_offset_value(&self, offset: f64) -> Result<(), Failure> {
        {
            let mut state = self.state.write();
            state.eval_rep = Handle::default();
            state.offset_value = offset;
        }
        self.set_eval_representation(full_rep(self.surface()?))
    }
    rustycad_tkg2d::eval_representation_members!(
        dyn RepSurfaceDesc,
        rep_utils::validate_surface_desc,
        "s"
    );
    /// The offset distance (`Offset`); `s.offset()`.
    pub fn offset(&self) -> f64 {
        self.state.read().offset_value
    }
    /// The basis surface (`BasisSurface`); `s.basis_surface()`.
    pub fn basis_surface(&self) -> Handle<dyn GeomSurface> {
        self.state.read().basis.clone()
    }
    /// Continuity of the unwrapped basis, G1 for a C0 basis whose isos are G1
    /// (`GetBasisSurfContinuity`); `s.basis_surf_continuity()`.
    pub fn basis_surf_continuity(&self) -> Shape {
        self.state.read().basis_continuity
    }
    /// The equivalent surface: the basis itself for a null offset, a parallel plane or
    /// a quadric of changed radius, trimmed like the basis; null otherwise (`Surface`);
    /// `s.surface()?`.
    pub fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        let (basis, offset) = {
            let state = self.state.read();
            (state.basis.clone(), state.offset_value)
        };
        equivalent_surface(&basis, offset)
    }
    /// The osculating surface along U at `(u, v)`, if the basis degenerates there
    /// (`UOsculatingSurface`); `s.u_osculating_surface(u, v, &mut opposite, &mut l)?`.
    pub fn u_osculating_surface(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<GeomBSplineSurface>,
    ) -> Result<bool, Failure> {
        match self.state.read().osc.as_ref() {
            Some(osc) => osc.u_osculating(u, v, is_opposite, surface),
            None => Ok(false),
        }
    }
    /// The osculating surface along V at `(u, v)` (`VOsculatingSurface`);
    /// `s.v_osculating_surface(u, v, &mut opposite, &mut l)?`.
    pub fn v_osculating_surface(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<GeomBSplineSurface>,
    ) -> Result<bool, Failure> {
        match self.state.read().osc.as_ref() {
            Some(osc) => osc.v_osculating(u, v, is_opposite, surface),
            None => Ok(false),
        }
    }
    // directRepSurface: the representation of a Full descriptor, null otherwise.
    fn direct_rep_surface(&self) -> Handle<dyn GeomSurface> {
        let desc = self.eval_representation();
        match desc.get() {
            Some(desc) if desc.kind() == RepSurfaceKind::Full => desc.representation(),
            _ => Handle::default(),
        }
    }
}

// GeomAdaptor_Surface passes the offset surface itself as the osculating query of
// Geom_OffsetSurfaceUtils; its members forward to the osculating surface, if any.
impl OsculatingQuery for GeomOffsetSurface {
    fn u_osculating_surface(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<dyn GeomSurface>,
    ) -> Result<bool, Failure> {
        match self.state.read().osc.as_ref() {
            Some(osc) => osc.u_osculating_surface(u, v, is_opposite, surface),
            None => Ok(false),
        }
    }
    fn v_osculating_surface(
        &self,
        u: f64,
        v: f64,
        is_opposite: &mut bool,
        surface: &mut Handle<dyn GeomSurface>,
    ) -> Result<bool, Failure> {
        match self.state.read().osc.as_ref() {
            Some(osc) => osc.v_osculating_surface(u, v, is_opposite, surface),
            None => Ok(false),
        }
    }
}

// Whether the unwrapped basis is a B-spline or a Bezier surface (the kinds that get
// an osculating surface).
fn is_spline_surface(surface: &Handle<dyn GeomSurface>) -> bool {
    surface.cast::<GeomBSplineSurface>().is_some()
        || surface.cast::<crate::geom::GeomBezierSurface>().is_some()
}

fn undefined_value(member: &str) -> Failure {
    errors::undefined_value(&format!(
        "Geom_OffsetSurface::{member}: no normal could be found at the point or around it; expected a regular basis"
    ))
}

fn undefined_derivative(member: &str) -> Failure {
    errors::undefined_derivative(&format!(
        "Geom_OffsetSurface::{member}: no normal could be found at the point or around it; expected a regular basis"
    ))
}
