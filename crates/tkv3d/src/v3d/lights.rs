//! The V3d light classes over `Graphic3d_CLight` (`V3d_Light` is a typedef of it):
//! `V3d_AmbientLight`, `V3d_PositionLight`, `V3d_DirectionalLight`, `V3d_PositionalLight` and
//! `V3d_SpotLight`. Each embeds its parent and derefs to it; the members the native classes
//! re-declare as private (`using Graphic3d_CLight::SetAngle` for an ambient light...) stay
//! reachable through the parent, as they do natively through a `Graphic3d_CLight` handle.
use super::{V3d, V3dTypeOfOrientation};
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::{Graphic3dCLight, Graphic3dTypeOfLightSource};

/// Implements `Deref` to the embedded parent light.
macro_rules! light_parent {
    ($ty:ident, $parent:ty, $field:ident) => {
        impl std::ops::Deref for $ty {
            type Target = $parent;
            fn deref(&self) -> &$parent {
                &self.$field
            }
        }
    };
}

/// An ambient light; `V3dAmbientLight::new(white)`.
pub struct V3dAmbientLight {
    light: Graphic3dCLight,
}
rustycad_tkernel::impl_transient!(V3dAmbientLight, "V3d_AmbientLight", Graphic3dCLight, light);
light_parent!(V3dAmbientLight, Graphic3dCLight, light);

impl V3dAmbientLight {
    /// An ambient light of a color; `V3dAmbientLight::new(white)`.
    pub fn new(color: Color) -> Self {
        let light = Graphic3dCLight::new(Graphic3dTypeOfLightSource::Ambient);
        light.set_color(color);
        Self { light }
    }
}

/// The base of the lights with a position or a direction (`V3d_PositionLight`).
pub struct V3dPositionLight {
    light: Graphic3dCLight,
}
rustycad_tkernel::impl_transient!(
    V3dPositionLight,
    "V3d_PositionLight",
    Graphic3dCLight,
    light
);
light_parent!(V3dPositionLight, Graphic3dCLight, light);

impl V3dPositionLight {
    /// The protected constructor: a light of a source type; `V3dPositionLight::new(kind)`.
    pub fn new(kind: Graphic3dTypeOfLightSource) -> Self {
        Self {
            light: Graphic3dCLight::new(kind),
        }
    }
}

/// A directional light; `V3dDirectionalLight::new(V3dTypeOfOrientation::Zneg, white, true)?`.
pub struct V3dDirectionalLight {
    base: V3dPositionLight,
}
rustycad_tkernel::impl_transient!(
    V3dDirectionalLight,
    "V3d_DirectionalLight",
    V3dPositionLight,
    base
);
light_parent!(V3dDirectionalLight, V3dPositionLight, base);

impl V3dDirectionalLight {
    /// A directional light along a predefined orientation; setting a headlight on a
    /// non-directional light raises natively, here the light is directional.
    /// Example: `V3dDirectionalLight::new(V3dTypeOfOrientation::XposYposZpos, white, false)?`.
    pub fn new(
        direction: V3dTypeOfOrientation,
        color: Color,
        is_headlight: bool,
    ) -> Result<Self, Failure> {
        Self::with_direction(&V3d::get_proj_axis(direction), color, is_headlight)
    }
    /// A directional light along a direction; `V3dDirectionalLight::with_direction(&dir, white, false)?`.
    pub fn with_direction(
        direction: &Direction3,
        color: Color,
        is_headlight: bool,
    ) -> Result<Self, Failure> {
        let base = V3dPositionLight::new(Graphic3dTypeOfLightSource::Directional);
        base.set_color(color);
        base.set_headlight(is_headlight)?;
        base.set_direction(direction)?;
        Ok(Self { base })
    }
    /// `SetDirection(V3d_TypeOfOrientation)`; `light.set_direction_orientation(V3dTypeOfOrientation::Zneg)?`.
    pub fn set_direction_orientation(
        &self,
        direction: V3dTypeOfOrientation,
    ) -> Result<(), Failure> {
        self.set_direction(&V3d::get_proj_axis(direction))
    }
}

/// A positional (point) light; `V3dPositionalLight::new(&p, white)?`.
pub struct V3dPositionalLight {
    base: V3dPositionLight,
}
rustycad_tkernel::impl_transient!(
    V3dPositionalLight,
    "V3d_PositionalLight",
    V3dPositionLight,
    base
);
light_parent!(V3dPositionalLight, V3dPositionLight, base);

impl V3dPositionalLight {
    /// A point light at a position; `V3dPositionalLight::new(&p, white)?`.
    pub fn new(position: &Point3, color: Color) -> Result<Self, Failure> {
        let base = V3dPositionLight::new(Graphic3dTypeOfLightSource::Positional);
        base.set_color(color);
        base.set_position(position)?;
        Ok(Self { base })
    }
}

/// A spot light; `V3dSpotLight::new(&p, V3dTypeOfOrientation::XnegYnegZpos, white)?`.
pub struct V3dSpotLight {
    base: V3dPositionLight,
}
rustycad_tkernel::impl_transient!(V3dSpotLight, "V3d_SpotLight", V3dPositionLight, base);
light_parent!(V3dSpotLight, V3dPositionLight, base);

impl V3dSpotLight {
    /// A spot light at a position along a predefined orientation.
    /// Example: `V3dSpotLight::new(&p, V3dTypeOfOrientation::Zneg, white)?`.
    pub fn new(
        position: &Point3,
        direction: V3dTypeOfOrientation,
        color: Color,
    ) -> Result<Self, Failure> {
        Self::with_direction(position, &V3d::get_proj_axis(direction), color)
    }
    /// A spot light at a position along a direction; `V3dSpotLight::with_direction(&p, &dir, white)?`.
    pub fn with_direction(
        position: &Point3,
        direction: &Direction3,
        color: Color,
    ) -> Result<Self, Failure> {
        let base = V3dPositionLight::new(Graphic3dTypeOfLightSource::Spot);
        base.set_color(color);
        base.set_position(position)?;
        base.set_direction(direction)?;
        Ok(Self { base })
    }
    /// `SetDirection(V3d_TypeOfOrientation)`; `light.set_direction_orientation(V3dTypeOfOrientation::Yneg)?`.
    pub fn set_direction_orientation(
        &self,
        direction: V3dTypeOfOrientation,
    ) -> Result<(), Failure> {
        self.set_direction(&V3d::get_proj_axis(direction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::quantity::NamedColor;
    use rustycad_tkernel::standard::{Handle, RuntimeTyped};

    #[test]
    fn lights_keep_type_color_and_geometry() {
        let white = Color::named(NamedColor::WHITE);
        let ambient = V3dAmbientLight::new(white);
        assert_eq!(ambient.light_type(), Graphic3dTypeOfLightSource::Ambient);
        let dir = V3dDirectionalLight::new(V3dTypeOfOrientation::Zneg, white, true).unwrap();
        assert!(dir.is_headlight());
        assert_eq!(dir.direction().unwrap().z(), -1.0);
        let spot = V3dSpotLight::new(
            &Point3::new(1.0, 2.0, 3.0),
            V3dTypeOfOrientation::Xpos,
            white,
        )
        .unwrap();
        assert_eq!(spot.position().x(), 1.0);
        spot.set_direction_orientation(V3dTypeOfOrientation::Yneg)
            .unwrap();
        assert_eq!(spot.direction().unwrap().y(), -1.0);
        let point =
            Handle::new(V3dPositionalLight::new(&Point3::new(0.0, 0.0, 5.0), white).unwrap());
        let base: Handle<Graphic3dCLight> = point.clone().cast();
        assert!(!base.is_null());
        assert_eq!(
            point.dynamic_type().unwrap().name(),
            V3dPositionalLight::TYPE_NAME
        );
    }
}
