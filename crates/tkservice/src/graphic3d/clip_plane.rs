//! `Graphic3d_ClipPlane`: a clipping plane, optionally chained with further planes into a
//! union ("clip the space outside every plane of the chain"), with its capping attributes.
//!
//! The previous plane of a chain is a raw back pointer natively; it is a weak handle here, so
//! `ChainPreviousPlane` of a destroyed predecessor is null instead of a dangling pointer.
use super::{
    Graphic3dAspectFillArea3d, Graphic3dAspectsEdgeMembers, Graphic3dAspectsMembers,
    Graphic3dAspectsRenderMembers, Graphic3dAspectsSource, Graphic3dMaterialAspect,
    Graphic3dNameOfMaterial, Graphic3dTypeOfBackfacingModel,
};
use crate::aspect::{AspectHatchStyle, AspectInteriorStyle};
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Plane3;
use std::sync::atomic::{AtomicI32, Ordering};

/// `Graphic3d_ClipPlane::Equation`: the plane equation `A*x + B*y + C*z + D = 0`.
pub type Graphic3dClipPlaneEquation = Vec4<f64>;

/// The clipping state of a point or a box (native `Graphic3d_ClipState`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Graphic3dClipState {
    /// `Graphic3d_ClipState_Out`: fully outside (clipped), to be discarded.
    Out = 0,
    /// `Graphic3d_ClipState_In`: fully inside (not clipped).
    In = 1,
    /// `Graphic3d_ClipState_On`: on the plane, or partially clipped.
    On = 2,
}

/// The process-wide counter of the plane identifiers (`THE_CLIP_PLANE_COUNTER`).
static THE_CLIP_PLANE_COUNTER: AtomicI32 = AtomicI32::new(0);

/// The fields of a clip plane, shared by every handle to it.
#[derive(Clone)]
pub(super) struct ClipPlaneState {
    /// The capping aspect; never null (`set_capping_aspect` refuses null).
    pub(super) aspect: Handle<Graphic3dAspectFillArea3d>,
    pub(super) next: Handle<Graphic3dClipPlane>,
    pub(super) prev: WeakHandle<Graphic3dClipPlane>,
    pub(super) id: AsciiString,
    pub(super) plane: Plane3,
    pub(super) equation: Vec4<f64>,
    pub(super) equation_rev: Vec4<f64>,
    pub(super) chain_len_fwd: i32,
    pub(super) flags: i32,
    pub(super) equation_mod: u32,
    pub(super) aspect_mod: u32,
    pub(super) is_on: bool,
    pub(super) is_capping: bool,
}

/// A clipping plane; `Graphic3dClipPlane::from_equation(Vec4::new(0.0, 0.0, 1.0, 0.0))?`.
pub struct Graphic3dClipPlane {
    transient: StandardTransient,
    pub(super) state: SharedState<ClipPlaneState>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dClipPlane,
    "Graphic3d_ClipPlane",
    StandardTransient,
    transient
);

/// The capping aspect of a new plane (`defaultAspect`): solid gray, horizontal hatch.
fn default_aspect() -> Handle<Graphic3dAspectFillArea3d> {
    let aspect = Graphic3dAspectFillArea3d::default();
    aspect.set_distinguish(false);
    aspect.set_front_material(Graphic3dMaterialAspect::new(
        Graphic3dNameOfMaterial::Default,
    ));
    aspect.set_hatch_style_type(AspectHatchStyle::Horizontal);
    aspect.set_interior_style(AspectInteriorStyle::Solid);
    aspect.set_interior_color(Color::named(NamedColor::GRAY20));
    aspect.set_face_culling(Graphic3dTypeOfBackfacingModel::DoubleSided);
    Handle::new(aspect)
}

/// `makeId`: `Graphic3d_ClipPlane_<n>` with the next value of the process-wide counter.
fn make_id() -> AsciiString {
    let number = THE_CLIP_PLANE_COUNTER
        .fetch_add(1, Ordering::SeqCst)
        .wrapping_add(1);
    AsciiString::from(format!("Graphic3d_ClipPlane_{number}").as_str())
}

/// The equation of `plane` with its axis reversed (`updateInversedPlane`).
fn reversed_equation(plane: &Plane3) -> Result<Vec4<f64>, Failure> {
    let mut reversed = *plane;
    reversed.set_axis(plane.axis().reversed())?;
    let [a, b, c, d] = reversed.coefficients()?;
    Ok(Vec4::new(a, b, c, d))
}

/// `gp_Pln(A, B, C, D)` of an equation; a null normal raises `Standard_ConstructionError`.
fn plane_of(equation: &Vec4<f64>) -> Result<Plane3, Failure> {
    Plane3::from_equation(equation.x, equation.y, equation.z, equation.w)
}

impl ClipPlaneState {
    /// The initial fields of the constructors: the given plane and equations, a fresh id.
    fn new(plane: Plane3, equation: Vec4<f64>, equation_rev: Vec4<f64>) -> Self {
        Self {
            aspect: default_aspect(),
            next: Handle::default(),
            prev: WeakHandle::default(),
            id: make_id(),
            plane,
            equation,
            equation_rev,
            chain_len_fwd: 1,
            flags: 0,
            equation_mod: 0,
            aspect_mod: 0,
            is_on: true,
            is_capping: false,
        }
    }
}

impl Default for Graphic3dClipPlane {
    /// The plane `z = 0` facing +Z, built as `gp_Pln(0, 0, 1, 0)` (origin `(0, 0, -0)`); its
    /// reversed equation is the literal `(0, 0, -1, 0)`.
    fn default() -> Self {
        let equation = Vec4::new(0.0, 0.0, 1.0, 0.0);
        // A unit normal cannot raise, so the fallback is never taken.
        let plane = plane_of(&equation).unwrap_or_default();
        Self::with_state(ClipPlaneState::new(
            plane,
            equation,
            Vec4::new(0.0, 0.0, -1.0, 0.0),
        ))
    }
}

impl Graphic3dClipPlane {
    fn with_state(state: ClipPlaneState) -> Self {
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// A copy of the fields; `plane.snapshot().equation`.
    pub(super) fn snapshot(&self) -> ClipPlaneState {
        self.state.read().clone()
    }
    /// The plane of an equation; a null normal raises `Standard_ConstructionError`.
    /// Example: `Graphic3dClipPlane::from_equation(Vec4::new(1.0, 0.0, 0.0, -2.0))?`.
    pub fn from_equation(equation: Graphic3dClipPlaneEquation) -> Result<Self, Failure> {
        let plane = plane_of(&equation)?;
        let equation_rev = reversed_equation(&plane)?;
        Ok(Self::with_state(ClipPlaneState::new(
            plane,
            equation,
            equation_rev,
        )))
    }
    /// The plane of a `gp_Pln`; `Graphic3dClipPlane::from_plane(&plane)?`.
    pub fn from_plane(plane: &Plane3) -> Result<Self, Failure> {
        let [a, b, c, d] = plane.coefficients()?;
        let equation_rev = reversed_equation(plane)?;
        Ok(Self::with_state(ClipPlaneState::new(
            *plane,
            Vec4::new(a, b, c, d),
            equation_rev,
        )))
    }
    /// The copy constructor: equations, flags and on/capping states with a fresh id, a copy of
    /// the capping aspect, no chain and zero counters. Example: `Graphic3dClipPlane::copy_of(&plane)`.
    pub fn copy_of(other: &Self) -> Self {
        let source = other.snapshot();
        let mut state = ClipPlaneState::new(source.plane, source.equation, source.equation_rev);
        (state.flags, state.is_on) = (source.flags, source.is_on);
        state.is_capping = source.is_capping;
        let fields = source.aspect.aspects_base().fields.read().clone();
        *state.aspect.aspects_base().fields.write() = fields;
        Self::with_state(state)
    }
    /// `Clone`: a new handle to a copy; `plane.clone_plane()`.
    pub fn clone_plane(&self) -> Handle<Self> {
        Handle::new(Self::copy_of(self))
    }
    /// Sets the plane from an equation; a null normal raises and leaves the plane.
    /// Example: `plane.set_equation(Vec4::new(0.0, 1.0, 0.0, 0.0))?`.
    pub fn set_equation(&self, equation: Graphic3dClipPlaneEquation) -> Result<(), Failure> {
        let plane = plane_of(&equation)?;
        self.store_equation(plane, equation)
    }
    /// Sets the plane from a `gp_Pln`; `plane.set_plane(&plane)?`.
    pub fn set_plane(&self, plane: &Plane3) -> Result<(), Failure> {
        let [a, b, c, d] = plane.coefficients()?;
        self.store_equation(*plane, Vec4::new(a, b, c, d))
    }
    fn store_equation(&self, plane: Plane3, equation: Vec4<f64>) -> Result<(), Failure> {
        let equation_rev = reversed_equation(&plane)?;
        let mut state = self.state.write();
        (state.plane, state.equation, state.equation_rev) = (plane, equation, equation_rev);
        state.equation_mod = state.equation_mod.wrapping_add(1);
        Ok(())
    }
    /// The plane equation; `plane.get_equation()`.
    pub fn get_equation(&self) -> Graphic3dClipPlaneEquation {
        self.state.read().equation
    }
    /// The equation of the reversed plane; `plane.reversed_equation()`.
    pub fn reversed_equation(&self) -> Graphic3dClipPlaneEquation {
        self.state.read().equation_rev
    }
    /// The plane as a `gp_Pln`; `plane.to_plane()`.
    pub fn to_plane(&self) -> Plane3 {
        self.state.read().plane
    }
    /// Whether the plane clips; `plane.is_on()`.
    pub fn is_on(&self) -> bool {
        self.state.read().is_on
    }
    /// Turns the clipping on or off; a plane inside a chain raises `Standard_ProgramError`.
    /// Example: `plane.set_on(false)?`.
    pub fn set_on(&self, is_on: bool) -> Result<(), Failure> {
        let mut state = self.state.write();
        if !state.prev.is_null() {
            return Err(program_error(&format!(
                "Graphic3d_ClipPlane::SetOn() - undefined operation for a plane in Union \
                 (plane {} has a previous plane; expected a chain head)",
                String::from_utf8_lossy(state.id.as_bytes())
            )));
        }
        state.is_on = is_on;
        Ok(())
    }
    /// Whether the capping is drawn; `plane.is_capping()`.
    pub fn is_capping(&self) -> bool {
        self.state.read().is_capping
    }
    /// Turns the capping on or off; `plane.set_capping(true)`.
    pub fn set_capping(&self, is_on: bool) {
        self.state.write().is_capping = is_on;
    }
    /// The unique identifier `Graphic3d_ClipPlane_<n>`; `plane.get_id()`.
    pub fn get_id(&self) -> AsciiString {
        self.state.read().id.clone()
    }
    /// The equation modification counter; `plane.m_count_equation()`.
    pub fn m_count_equation(&self) -> u32 {
        self.state.read().equation_mod
    }
    /// The aspect modification counter; `plane.m_count_aspect()`.
    pub fn m_count_aspect(&self) -> u32 {
        self.state.read().aspect_mod
    }
}
