//! The checked setters of `Graphic3d_CLight` (each counts a revision only for an effective
//! change) and its `DumpJson`.
use super::light::{LightState, is_one_of};
use super::{Graphic3dCLight, Graphic3dTypeOfLightSource};
use crate::json_dump::{
    class_begin, field_dumped, field_integer, field_pointer, field_real, field_string,
    field_unsigned,
};
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::{not_implemented, out_of_range, program_error};
use rustycad_tkernel::standard::short_real::short_real_epsilon;
use rustycad_tkmath::gp::{Direction3, Point3, resolution};
use std::f64::consts::PI;
use std::io;

use Graphic3dTypeOfLightSource as T;

/// Whether two floats differ by more than `ShortRealEpsilon()`.
fn differs(a: f32, b: f32) -> bool {
    (a - b).abs() > short_real_epsilon()
}

/// `value < lower || value > upper`: a NaN is inside, as in the native checks.
#[allow(clippy::manual_range_contains)]
fn outside(value: f32, lower: f32, upper: f32) -> bool {
    value < lower || value > upper
}

/// `Standard_OutOfRange_Raise_if(bad, message)`.
fn refuse_if(bad: bool, message: &str) -> Result<(), Failure> {
    if bad {
        return Err(out_of_range(message));
    }
    Ok(())
}

impl Graphic3dCLight {
    /// `Standard_ProgramError_Raise_if` of a light type outside `kinds`.
    fn require_type(&self, kinds: &[T], method: &str) -> Result<(), Failure> {
        let kind = self.light_type();
        if is_one_of(kind, kinds) {
            return Ok(());
        }
        Err(program_error(&format!(
            "Graphic3d_CLight::{method}(), incorrect light type {kind:?}; expected one of {kinds:?}"
        )))
    }
    /// Applies `edit`, which answers whether it changed anything, and counts a revision then.
    fn update(&self, edit: impl FnOnce(&mut LightState) -> bool) {
        let mut state = self.state.write();
        if edit(&mut state) {
            state.revision = state.revision.wrapping_add(1);
        }
    }
    /// Sets the color; `light.set_color(red)`.
    pub fn set_color(&self, color: Color) {
        self.update(|s| {
            let changed = s.color.rgb().is_different(color);
            *s.color.change_rgb() = color;
            changed
        });
    }
    /// Turns the light on or off; `light.set_enabled(false)`.
    pub fn set_enabled(&self, is_on: bool) {
        self.update(|s| std::mem::replace(&mut s.is_enabled, is_on) != is_on);
    }
    /// Shadow casting of a directional or spot light; other types raise
    /// `Standard_NotImplemented`. Example: `light.set_cast_shadows(true)?`.
    pub fn set_cast_shadows(&self, to_cast: bool) -> Result<(), Failure> {
        let kind = self.light_type();
        if !is_one_of(kind, &[T::Directional, T::Spot]) {
            return Err(not_implemented(&format!(
                "Graphic3d_CLight::SetCastShadows() is not implemented for this light type ({kind:?})"
            )));
        }
        self.update(|s| std::mem::replace(&mut s.to_cast_shadows, to_cast) != to_cast);
        Ok(())
    }
    /// Follows the camera; an ambient light raises `Standard_ProgramError`.
    /// Example: `light.set_headlight(true)?`.
    pub fn set_headlight(&self, value: bool) -> Result<(), Failure> {
        if self.light_type() == T::Ambient {
            return Err(program_error(
                "Graphic3d_CLight::SetHeadlight() is not applicable to ambient light",
            ));
        }
        self.update(|s| std::mem::replace(&mut s.is_headlight, value) != value);
        Ok(())
    }
    /// Stores a position, counting a move beyond `gp::Resolution()`.
    fn store_position(&self, position: Point3) {
        self.update(|s| {
            let moved = !s.position.is_equal(&position, resolution());
            s.position = position;
            moved
        });
    }
    /// The position of a positional or spot light; `light.set_position(&p)?`.
    pub fn set_position(&self, position: &Point3) -> Result<(), Failure> {
        self.require_type(&[T::Spot, T::Positional], "SetPosition")?;
        self.store_position(*position);
        Ok(())
    }
    /// `SetPosition(x, y, z)`; `light.set_position_coords(1.0, 2.0, 3.0)?`.
    pub fn set_position_coords(&self, x: f64, y: f64, z: f64) -> Result<(), Failure> {
        self.set_position(&Point3::new(x, y, z))
    }
    /// The display position of a non-ambient light; `light.set_display_position(&p)?`.
    pub fn set_display_position(&self, position: &Point3) -> Result<(), Failure> {
        self.require_type(
            &[T::Directional, T::Positional, T::Spot],
            "SetDisplayPosition",
        )?;
        self.store_position(*position);
        Ok(())
    }
    /// The direction of a spot or directional light, stored as floats.
    /// Example: `light.set_direction(&direction)?`.
    pub fn set_direction(&self, direction: &Direction3) -> Result<(), Failure> {
        self.require_type(&[T::Spot, T::Directional], "SetDirection")?;
        let [x, y, z] = [direction.x(), direction.y(), direction.z()].map(|v| v as f32);
        self.update(|s| {
            let d = &mut s.direction;
            let changed = differs(d.x, x) || differs(d.y, y) || differs(d.z, z);
            (d.x, d.y, d.z) = (x, y, z);
            changed
        });
        Ok(())
    }
    /// `SetDirection(vx, vy, vz)`: a null vector raises `Standard_ConstructionError` first.
    /// Example: `light.set_direction_coords(0.0, 0.0, -1.0)?`.
    pub fn set_direction_coords(&self, vx: f64, vy: f64, vz: f64) -> Result<(), Failure> {
        self.set_direction(&Direction3::new(vx, vy, vz)?)
    }
    /// A positive intensity; `light.set_intensity(2.0)?`.
    pub fn set_intensity(&self, value: f32) -> Result<(), Failure> {
        refuse_if(
            value <= 0.0,
            &format!(
                "Graphic3d_CLight::SetIntensity(), Negative value for intensity {value}; expected > 0"
            ),
        )?;
        self.update(|s| differs(std::mem::replace(&mut s.intensity, value), value));
        Ok(())
    }
}

impl Graphic3dCLight {
    /// The spot cone angle in (0, PI); `light.set_angle(0.5)?`.
    pub fn set_angle(&self, angle: f32) -> Result<(), Failure> {
        self.require_type(&[T::Spot], "SetAngle")?;
        let wide = f64::from(angle);
        refuse_if(
            wide <= 0.0 || wide >= PI,
            &format!("Graphic3d_CLight::SetAngle(), bad angle {angle}; expected (0, PI)"),
        )?;
        self.update(|s| differs(std::mem::replace(&mut s.params.z, angle), angle));
        Ok(())
    }
    /// Non-negative attenuations, not both zero; `light.set_attenuation(1.0, 0.5)?`.
    pub fn set_attenuation(&self, constant: f32, linear: f32) -> Result<(), Failure> {
        self.require_type(&[T::Positional, T::Spot], "SetAttenuation")?;
        refuse_if(
            constant < 0.0 || linear < 0.0 || constant + linear == 0.0,
            &format!(
                "Graphic3d_CLight::SetAttenuation(), bad coefficient ({constant}, {linear}); expected >= 0 with a positive sum"
            ),
        )?;
        self.update(|s| {
            let changed = differs(s.params.x, constant) || differs(s.params.y, linear);
            (s.params.x, s.params.y) = (constant, linear);
            changed
        });
        Ok(())
    }
    /// The spot concentration in [0, 1]; `light.set_concentration(0.5)?`.
    pub fn set_concentration(&self, concentration: f32) -> Result<(), Failure> {
        self.require_type(&[T::Spot], "SetConcentration")?;
        refuse_if(
            outside(concentration, 0.0, 1.0),
            &format!(
                "Graphic3d_CLight::SetConcentration(), bad coefficient {concentration}; expected [0, 1]"
            ),
        )?;
        self.update(|s| {
            differs(
                std::mem::replace(&mut s.params.w, concentration),
                concentration,
            )
        });
        Ok(())
    }
    /// The non-negative smoothing radius of a positional or spot light.
    /// Example: `light.set_smooth_radius(0.1)?`.
    pub fn set_smooth_radius(&self, value: f32) -> Result<(), Failure> {
        self.require_type(&[T::Positional, T::Spot], "SetSmoothRadius")?;
        refuse_if(
            value < 0.0,
            &format!(
                "Graphic3d_CLight::SetSmoothRadius(), Bad value for smoothing radius {value}; expected >= 0"
            ),
        )?;
        self.store_smoothness(value);
        Ok(())
    }
    /// The smoothing angle of a directional light in [0, PI/2]; `light.set_smooth_angle(0.1)?`.
    pub fn set_smooth_angle(&self, value: f32) -> Result<(), Failure> {
        self.require_type(&[T::Directional], "SetSmoothAngle")?;
        refuse_if(
            outside(value, 0.0, (PI / 2.0) as f32),
            &format!(
                "Graphic3d_CLight::SetSmoothAngle(), Bad value for smoothing angle {value}; expected [0, PI/2]"
            ),
        )?;
        self.store_smoothness(value);
        Ok(())
    }
    fn store_smoothness(&self, value: f32) {
        self.update(|s| differs(std::mem::replace(&mut s.smoothness, value), value));
    }
    /// The non-negative falloff range of a positional or spot light (0 for none).
    /// Example: `light.set_range(100.0)?`.
    pub fn set_range(&self, value: f32) -> Result<(), Failure> {
        self.require_type(&[T::Positional, T::Spot], "SetRange")?;
        refuse_if(
            value < 0.0,
            &format!(
                "Graphic3d_CLight::SetRange(), Bad value for falloff range {value}; expected >= 0"
            ),
        )?;
        self.update(|s| differs(std::mem::replace(&mut s.direction.w, value), value));
        Ok(())
    }
    /// `DumpJson`; the direction of a spot or directional light without one raises the
    /// `Standard_ConstructionError` of `gp_Dir`, as natively. Example: `light.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let s = self.snapshot();
        class_begin(stream, "Graphic3d_CLight")?;
        field_pointer(stream, "this", std::ptr::from_ref(self) as usize)?;
        field_string(stream, "myId", s.id.as_bytes())?;
        field_string(stream, "myName", s.name.as_bytes())?;
        if is_one_of(s.kind, &[T::Spot, T::Positional]) {
            field_dumped(stream, depth, "&myPosition", &|o, _| {
                s.position.dump_json(o)
            })?;
        }
        field_dumped(stream, depth, "&myColor", &|o, _| s.color.dump_json(o))?;
        field_real(stream, "myIntensity", f64::from(s.intensity))?;
        if is_one_of(s.kind, &[T::Spot, T::Directional]) {
            let direction = self.direction().map_err(io::Error::other)?;
            field_dumped(stream, depth, "&aDirection", &|o, _| direction.dump_json(o))?;
        }
        dump_typed_params(&s, stream)?;
        field_real(stream, "mySmoothness", f64::from(s.smoothness))?;
        field_integer(stream, "myType", s.kind as i32)?;
        field_unsigned(stream, "myRevision", s.revision as u64)?;
        field_integer(stream, "myIsHeadlight", i32::from(s.is_headlight))?;
        field_integer(stream, "myIsEnabled", i32::from(s.is_enabled))
    }
}

/// The attenuation, spot and range fields of the dump.
fn dump_typed_params(s: &LightState, stream: &mut DumpStream) -> io::Result<()> {
    let positional = is_one_of(s.kind, &[T::Positional, T::Spot]);
    if positional {
        field_real(stream, "ConstAttenuation()", f64::from(s.params.x))?;
        field_real(stream, "LinearAttenuation()", f64::from(s.params.y))?;
    }
    if s.kind == T::Spot {
        field_real(stream, "Angle()", f64::from(s.params.z))?;
        field_real(stream, "Concentration()", f64::from(s.params.w))?;
    }
    if positional {
        field_real(stream, "Range()", f64::from(s.direction.w))?;
    }
    Ok(())
}
