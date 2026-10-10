//! Light packing follows the native world-space shader interface before GPU upload.
use super::*;
use crate::identity_clip;
use Graphic3dTypeOfLightSource as Kind;
use rustycad_tkernel::quantity::{Color, ColorSpace};
fn source(kind: Kind) -> Handle<Graphic3dCLight> {
    Handle::new(Graphic3dCLight::new(kind))
}
fn set(sources: &[Handle<Graphic3dCLight>]) -> Handle<Graphic3dLightSet> {
    let lights = Handle::new(Graphic3dLightSet::default());
    for light in sources {
        assert!(lights.add(light).unwrap());
    }
    lights
}
#[test]
fn null_and_empty_sets_preserve_native_ambient_alpha_difference() {
    let null = RenderLightSet::snapshot(&Handle::default(), identity_clip()).unwrap();
    let empty = RenderLightSet::snapshot(&set(&[]), identity_clip()).unwrap();
    assert_eq!(null.ambient_color(), [0.; 4]);
    assert_eq!(empty.ambient_color(), [0., 0., 0., 1.]);
    assert_eq!(empty.source_count(), 0);
}
#[test]
fn ambient_revision_refresh_filters_disabled_lights_and_snapshots_are_immutable() {
    let ambient = source(Kind::Ambient);
    ambient.set_color(Color::new(0.25, 0.5, 0.75, ColorSpace::Rgb).unwrap());
    ambient.set_intensity(2.).unwrap();
    let disabled = source(Kind::Directional);
    disabled.set_enabled(false);
    let lights = set(&[ambient.clone(), disabled.clone()]);
    let before = RenderLightSet::snapshot(&lights, identity_clip()).unwrap();
    assert_eq!(before.ambient_color(), [0.5, 1., 1.5, 1.]);
    assert_eq!(before.source_count(), 0);
    ambient.set_enabled(false);
    disabled.set_enabled(true);
    let after = RenderLightSet::snapshot(&lights, identity_clip()).unwrap();
    assert_eq!(after.ambient_color(), [0., 0., 0., 1.]);
    assert_eq!(after.source_count(), 1);
    assert_eq!(before.ambient_color(), [0.5, 1., 1.5, 1.]);
}
#[test]
fn directional_sources_negate_direction_and_keep_intensity_separate_from_rgb() {
    let light = source(Kind::Directional);
    light.set_direction_coords(0., 0., -1.).unwrap();
    light.set_intensity(3.).unwrap();
    let packed = RenderLightSet::snapshot(&set(&[light]), identity_clip()).unwrap();
    assert_eq!(packed.sources[0].position, [-0., -0., 1., 0.]);
    assert_eq!(packed.sources[0].color, [1., 1., 1., 3.]);
}
#[test]
fn spot_records_keep_cone_attenuation_range_and_world_position() {
    let light = source(Kind::Spot);
    light.set_direction_coords(0., 1., 0.).unwrap();
    light.set_position_coords(2., 3., 4.).unwrap();
    light.set_attenuation(0.5, 0.25).unwrap();
    light.set_angle(0.75).unwrap();
    light.set_concentration(0.125).unwrap();
    light.set_range(12.).unwrap();
    let packed = RenderLightSet::snapshot(&set(&[light]), identity_clip()).unwrap();
    assert_eq!(packed.sources[0].position, [2., 3., 4., 0.]);
    assert_eq!(packed.sources[0].direction, [0., 1., 0., 12.]);
    assert_eq!(packed.sources[0].params, [0.5, 0.25, 0.75, 0.125]);
}
#[test]
fn headlights_translate_positions_but_only_rotate_and_normalize_directions() {
    let inverse = [
        0., 2., 0., 0., -2., 0., 0., 0., 0., 0., 2., 0., 4., 5., 6., 1.,
    ];
    let spot = source(Kind::Spot);
    spot.set_headlight(true).unwrap();
    spot.set_position_coords(1., 2., 3.).unwrap();
    spot.set_direction_coords(1., 0., 0.).unwrap();
    let directional = source(Kind::Directional);
    directional.set_headlight(true).unwrap();
    directional.set_direction_coords(1., 0., 0.).unwrap();
    let point = source(Kind::Positional);
    point.set_headlight(true).unwrap();
    point.set_position_coords(1., 2., 3.).unwrap();
    let packed = RenderLightSet::snapshot(&set(&[spot, directional, point]), inverse).unwrap();
    assert_eq!(packed.sources[0].position, [0., 7., 12., 1.]);
    assert_eq!(packed.sources[0].direction, [0., 1., 0., 0.]);
    assert_eq!(packed.sources[1].position, [0., -1., 0., 0.]);
    assert_eq!(packed.sources[2].position, [0., 7., 12., 1.]);
}
#[test]
fn snapshot_rejects_nonfinite_inverse_view_with_offending_value() {
    let mut inverse = identity_clip();
    inverse[7] = f32::INFINITY;
    let error = RenderLightSet::snapshot(&Handle::default(), inverse).unwrap_err();
    assert!(error.to_string().contains("inf"));
    assert!(
        error
            .to_string()
            .contains("finite inverse world-view matrix")
    );
}
