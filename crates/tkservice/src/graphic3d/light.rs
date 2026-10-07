//! `Graphic3d_CLight`: a light source (ambient, directional, positional or spot) with its
//! packed shader parameters and a revision counter of the effective changes.
use super::Graphic3dTypeOfLightSource;
use rustycad_tkernel::ncollection::{Vec3, Vec4};
use rustycad_tkernel::quantity::{Color, ColorRgba, NamedColor};
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Direction3, Point3};
use std::sync::atomic::{AtomicI32, Ordering};

/// The process-wide counter of the light identifiers (`THE_LIGHT_COUNTER`).
static THE_LIGHT_COUNTER: AtomicI32 = AtomicI32::new(0);

/// The fields of a light, shared by every handle to it.
#[derive(Clone)]
pub(super) struct LightState {
    pub(super) id: AsciiString,
    pub(super) name: AsciiString,
    pub(super) position: Point3,
    pub(super) color: ColorRgba,
    /// The direction (xyz) and the range (w).
    pub(super) direction: Vec4<f32>,
    /// Constant and linear attenuation, spot angle and concentration.
    pub(super) params: Vec4<f32>,
    pub(super) smoothness: f32,
    pub(super) intensity: f32,
    pub(super) kind: Graphic3dTypeOfLightSource,
    pub(super) revision: usize,
    pub(super) is_headlight: bool,
    pub(super) is_enabled: bool,
    pub(super) to_cast_shadows: bool,
}

/// A light source; `Graphic3dCLight::new(Graphic3dTypeOfLightSource::Spot)`.
pub struct Graphic3dCLight {
    transient: StandardTransient,
    pub(super) state: SharedState<LightState>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dCLight,
    "Graphic3d_CLight",
    StandardTransient,
    transient
);

/// `makeId`: `Graphic3d_CLight_<amb|dir|pos|spot><n>` with the next counter value.
fn make_id(kind: Graphic3dTypeOfLightSource) -> AsciiString {
    let suffix = match kind {
        Graphic3dTypeOfLightSource::Ambient => "amb",
        Graphic3dTypeOfLightSource::Directional => "dir",
        Graphic3dTypeOfLightSource::Positional => "pos",
        Graphic3dTypeOfLightSource::Spot => "spot",
    };
    let number = THE_LIGHT_COUNTER
        .fetch_add(1, Ordering::SeqCst)
        .wrapping_add(1);
    AsciiString::from(format!("Graphic3d_CLight_{suffix}{number}").as_str())
}

/// The native literal spot angle `0.523599f` (not `FRAC_PI_6` rounded to float).
#[allow(clippy::approx_constant)]
const SPOT_DEFAULT_ANGLE: f32 = 0.523599;

/// The per-type defaults of the constructor: (params, smoothness, intensity).
fn type_defaults(kind: Graphic3dTypeOfLightSource) -> (Vec4<f32>, f32, f32) {
    match kind {
        Graphic3dTypeOfLightSource::Ambient => (Vec4::new(0.0, 0.0, 0.0, 0.0), 0.0, 1.0),
        Graphic3dTypeOfLightSource::Directional => (Vec4::new(0.0, 0.0, 0.0, 0.0), 0.2, 20.0),
        Graphic3dTypeOfLightSource::Positional => (Vec4::new(1.0, 0.0, 0.0, 0.0), 0.0, 1.0),
        Graphic3dTypeOfLightSource::Spot => {
            (Vec4::new(1.0, 0.0, SPOT_DEFAULT_ANGLE, 1.0), 0.0, 1.0)
        }
    }
}

/// Whether the light type is one of `kinds`.
pub(super) fn is_one_of(
    kind: Graphic3dTypeOfLightSource,
    kinds: &[Graphic3dTypeOfLightSource],
) -> bool {
    kinds.contains(&kind)
}

impl Graphic3dCLight {
    /// A light of `kind` with the native per-type defaults and a fresh id.
    /// Example: `Graphic3dCLight::new(Graphic3dTypeOfLightSource::Directional).intensity() == 20.0`.
    pub fn new(kind: Graphic3dTypeOfLightSource) -> Self {
        let (params, smoothness, intensity) = type_defaults(kind);
        let state = LightState {
            id: make_id(kind),
            name: AsciiString::default(),
            position: Point3::new(0.0, 0.0, 0.0),
            color: ColorRgba::from_rgb(Color::named(NamedColor::WHITE), 1.0),
            direction: Vec4::new(0.0, 0.0, 0.0, 0.0),
            params,
            smoothness,
            intensity,
            kind,
            revision: 0,
            is_headlight: false,
            is_enabled: true,
            to_cast_shadows: false,
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// A copy of the fields; `light.snapshot().revision`.
    pub(super) fn snapshot(&self) -> LightState {
        self.state.read().clone()
    }
    /// `CopyFrom`: the name, state, intensity and color, plus the members that the two
    /// light types share; a null source raises `Standard_ProgramError` (natively a null
    /// dereference). Example: `light.copy_from(&other)?`.
    pub fn copy_from(&self, source: &Handle<Graphic3dCLight>) -> Result<(), Failure> {
        let Some(source) = source.get() else {
            return Err(program_error(
                "Graphic3d_CLight::CopyFrom() - null light; expected a light to copy",
            ));
        };
        let from = source.snapshot();
        let mut to = self.state.write();
        (to.name, to.is_enabled) = (from.name.clone(), from.is_enabled);
        (to.intensity, to.color) = (from.intensity, from.color);
        copy_typed_members(&mut to, &from);
        Ok(())
    }
}

/// The type-dependent part of `CopyFrom`.
fn copy_typed_members(to: &mut LightState, from: &LightState) {
    use Graphic3dTypeOfLightSource as T;
    if to.kind != T::Ambient {
        (to.position, to.is_headlight) = (from.position, from.is_headlight);
    }
    let directed = [T::Spot, T::Directional];
    if is_one_of(to.kind, &directed) && is_one_of(from.kind, &directed) {
        to.direction = from.direction;
    }
    if to.kind == T::Directional {
        to.to_cast_shadows = from.to_cast_shadows;
    }
    if to.kind == from.kind {
        (to.params, to.smoothness) = (from.params, from.smoothness);
    }
}

impl Graphic3dCLight {
    /// The light type; `light.light_type()`.
    pub fn light_type(&self) -> Graphic3dTypeOfLightSource {
        self.state.read().kind
    }
    /// The user name; `light.name()`.
    pub fn name(&self) -> AsciiString {
        self.state.read().name.clone()
    }
    /// Sets the user name (not counted as a revision); `light.set_name(&"key".into())`.
    pub fn set_name(&self, name: &AsciiString) {
        self.state.write().name = name.clone();
    }
    /// The color; `light.color()`.
    pub fn color(&self) -> Color {
        *self.state.read().color.rgb()
    }
    /// Whether the light is on; `light.is_enabled()`.
    pub fn is_enabled(&self) -> bool {
        self.state.read().is_enabled
    }
    /// Whether the light casts shadows; `light.to_cast_shadows()`.
    pub fn to_cast_shadows(&self) -> bool {
        self.state.read().to_cast_shadows
    }
    /// Whether the light follows the camera; `light.is_headlight()`.
    pub fn is_headlight(&self) -> bool {
        self.state.read().is_headlight
    }
    /// Alias of [`Self::is_headlight`]; `light.headlight()`.
    pub fn headlight(&self) -> bool {
        self.is_headlight()
    }
    /// The position (also `DisplayPosition`); `light.position()`.
    pub fn position(&self) -> Point3 {
        self.state.read().position
    }
    /// The position coordinates (`Position(double&, double&, double&)`); `light.position_coords()`.
    pub fn position_coords(&self) -> [f64; 3] {
        let p = self.position();
        [p.x(), p.y(), p.z()]
    }
    /// The constant attenuation; `light.const_attenuation()`.
    pub fn const_attenuation(&self) -> f32 {
        self.state.read().params.x
    }
    /// The linear attenuation; `light.linear_attenuation()`.
    pub fn linear_attenuation(&self) -> f32 {
        self.state.read().params.y
    }
    /// Both attenuations as doubles; `light.attenuation()`.
    pub fn attenuation(&self) -> (f64, f64) {
        (
            f64::from(self.const_attenuation()),
            f64::from(self.linear_attenuation()),
        )
    }
    /// The direction; a zero direction (of a light without one) raises
    /// `Standard_ConstructionError` from `gp_Dir`. Example: `light.direction()?`.
    pub fn direction(&self) -> Result<Direction3, Failure> {
        let d = self.state.read().direction;
        Direction3::new(f64::from(d.x), f64::from(d.y), f64::from(d.z))
    }
    /// The direction components as doubles; `light.direction_coords()`.
    pub fn direction_coords(&self) -> [f64; 3] {
        let d = self.state.read().direction;
        [f64::from(d.x), f64::from(d.y), f64::from(d.z)]
    }
    /// The display position of a directional light; `light.display_position()`.
    pub fn display_position(&self) -> Point3 {
        self.position()
    }
    /// The spot cone angle; `light.angle()`.
    pub fn angle(&self) -> f32 {
        self.state.read().params.z
    }
    /// The spot concentration; `light.concentration()`.
    pub fn concentration(&self) -> f32 {
        self.state.read().params.w
    }
    /// The intensity; `light.intensity()`.
    pub fn intensity(&self) -> f32 {
        self.state.read().intensity
    }
    /// The smoothing radius or angle; `light.smoothness()`.
    pub fn smoothness(&self) -> f32 {
        self.state.read().smoothness
    }
    /// Whether a falloff range is set; `light.has_range()`.
    pub fn has_range(&self) -> bool {
        self.state.read().direction.w != 0.0
    }
    /// The falloff range (0 for none); `light.range()`.
    pub fn range(&self) -> f32 {
        self.state.read().direction.w
    }
    /// The unique id `Graphic3d_CLight_<type><n>`; `light.get_id()`.
    pub fn get_id(&self) -> AsciiString {
        self.state.read().id.clone()
    }
    /// The packed attenuations, angle and concentration; `light.packed_params()`.
    pub fn packed_params(&self) -> Vec4<f32> {
        self.state.read().params
    }
    /// The packed RGBA color; `light.packed_color()`.
    pub fn packed_color(&self) -> Vec4<f32> {
        let color = self.state.read().color;
        let [r, g, b] = color.rgb().rgb();
        Vec4::new(r, g, b, color.alpha())
    }
    /// The packed direction and range; `light.packed_direction_range()`.
    pub fn packed_direction_range(&self) -> Vec4<f32> {
        self.state.read().direction
    }
    /// The packed direction; `light.packed_direction()`.
    pub fn packed_direction(&self) -> Vec3<f32> {
        let d = self.state.read().direction;
        Vec3::new(d.x, d.y, d.z)
    }
    /// The number of effective changes; `light.revision()`.
    pub fn revision(&self) -> usize {
        self.state.read().revision
    }
}
