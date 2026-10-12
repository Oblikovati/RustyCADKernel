//! Native FFP light uploads use eye space, eight slots and legacy attenuation.
use crate::{RenderError, RenderLightSet, error::rejected, light_snapshot::PackedRenderLight};
use rustycad_tkernel::{
    ncollection::{Mat4, Vec4},
    standard::Handle,
};
use rustycad_tkservice::graphic3d::*;

pub(crate) fn snapshot_fixed_lights(
    lights: &Handle<Graphic3dLightSet>,
    view: [f32; 16],
) -> Result<RenderLightSet, RenderError> {
    validate_fixed_light_view(view)?;
    let Some(set) = lights.get() else {
        return Ok(RenderLightSet::default());
    };
    set.update_revision();
    Ok(RenderLightSet {
        ambient: set.ambient_color().data(),
        sources: fixed_enabled_sources(set, view),
    })
}
fn validate_fixed_light_view(view: [f32; 16]) -> Result<(), RenderError> {
    if view.iter().any(|v| !v.is_finite()) {
        return Err(rejected(view, "a finite fixed lighting world-view matrix"));
    }
    Mat4::from_data(view)
        .inverted()
        .map_err(|error| rejected((view, error), "an invertible fixed lighting world-view"))?;
    Ok(())
}
fn fixed_enabled_sources(set: &Graphic3dLightSet, view: [f32; 16]) -> Vec<PackedRenderLight> {
    set.lights()
        .iter()
        .filter(|light| {
            light.is_enabled() && light.light_type() != Graphic3dTypeOfLightSource::Ambient
        })
        .take(8)
        .map(|light| fixed_eye_light(light, Mat4::from_data(view)))
        .collect()
}

fn fixed_eye_light(light: &Graphic3dCLight, view: Mat4<f32>) -> PackedRenderLight {
    let matrix = if light.is_headlight() {
        Mat4::identity()
    } else {
        view
    };
    let position = fixed_light_position(light);
    let eye = (matrix * Vec4::from_array(position)).data();
    let direction = fixed_spot_direction(light, matrix);
    PackedRenderLight {
        kind: light.light_type() as u32,
        color: light.packed_color().data(),
        position: eye,
        direction,
        params: light.packed_params().data(),
    }
}
fn fixed_light_position(light: &Graphic3dCLight) -> [f32; 4] {
    if light.light_type() == Graphic3dTypeOfLightSource::Directional {
        return light.packed_direction_range().data().map(|v| -v);
    }
    let [x, y, z] = light.position_coords().map(|v| v as f32);
    [x, y, z, 1.]
}

fn fixed_spot_direction(light: &Graphic3dCLight, matrix: Mat4<f32>) -> [f32; 4] {
    if light.light_type() != Graphic3dTypeOfLightSource::Spot {
        return [0., 0., -1., 0.];
    }
    // GL transforms the spot direction as a normal at the upload boundary.
    let inverse = matrix
        .inverted()
        .expect("camera world-view is invertible")
        .transposed();
    let [x, y, z] = light.packed_direction().data();
    (inverse * Vec4::new(x, y, z, 0.)).data()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_ambient_headlight_and_eight_slot_rules_are_separate() {
        let lights = Handle::new(Graphic3dLightSet::default());
        let ambient = Handle::new(Graphic3dCLight::new(Graphic3dTypeOfLightSource::Ambient));
        lights.add(&ambient).unwrap();
        for _ in 0..10 {
            lights
                .add(&Handle::new(Graphic3dCLight::new(
                    Graphic3dTypeOfLightSource::Positional,
                )))
                .unwrap();
        }
        let mut view = crate::identity_clip();
        view[14] = -20.;
        let snapshot = snapshot_fixed_lights(&lights, view).unwrap();
        assert_eq!(snapshot.source_count(), 8);
        assert_eq!(snapshot.sources[0].position[2], -20.);
        let head = Handle::new(Graphic3dCLight::new(Graphic3dTypeOfLightSource::Spot));
        head.set_headlight(true).unwrap();
        assert_eq!(
            fixed_eye_light(&head, Mat4::from_data(view)).position[2],
            0.
        );
        assert_eq!(fixed_spot_direction(&head, Mat4::identity())[3], 0.);
        assert!(snapshot_fixed_lights(&lights, [f32::NAN; 16]).is_err());
    }
}
