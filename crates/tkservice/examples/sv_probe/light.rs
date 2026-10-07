//! Graphic3d_CLight and Graphic3d_LightSet, the twin of tools/oracle/tkservice_probe_light.cxx.
use super::common::{Reader, dir, end, flag, int, pnt, real, text};
use super::pbr::vec4 as vec4f;
use super::print::show;
use super::tried;
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::{
    Graphic3dCLight, Graphic3dLightSet, Graphic3dLightSetIterationFilter,
    Graphic3dLightSetIterator, Graphic3dTypeOfLightSource,
};

fn light_type(value: i32) -> Graphic3dTypeOfLightSource {
    use Graphic3dTypeOfLightSource as T;
    [T::Ambient, T::Directional, T::Positional, T::Spot][value as usize]
}

fn rgb(r: &mut Reader<'_>) -> Color {
    let [red, green, blue] = [0; 3].map(|_| r.real());
    Color::new(red, green, blue, ColorSpace::Rgb).unwrap_or_default()
}

fn light_state(l: &Graphic3dCLight) {
    vec4f(l.packed_params());
    vec4f(l.packed_color());
    vec4f(l.packed_direction_range());
    pnt(l.position());
    real(f64::from(l.smoothness()));
    real(f64::from(l.intensity()));
    int(l.revision() as i32);
    flag(l.is_enabled());
    flag(l.to_cast_shadows());
    flag(l.is_headlight());
    flag(l.has_range());
    show(l.direction(), dir);
}

/// The source light of operation 15: a light of `kind` with fixed settings.
fn copy_source(kind: Graphic3dTypeOfLightSource) -> Handle<Graphic3dCLight> {
    let source = Graphic3dCLight::new(kind);
    source.set_name(&AsciiString::from("source"));
    tried(source.set_intensity(3.0));
    source.set_color(Color::new(0.25, 0.5, 0.75, ColorSpace::Rgb).unwrap_or_default());
    tried(source.set_direction_coords(0.0, 0.6, -0.8));
    tried(source.set_position_coords(1.0, 2.0, 3.0));
    tried(source.set_headlight(true));
    tried(source.set_cast_shadows(true));
    tried(source.set_angle(0.25));
    tried(source.set_smooth_radius(0.5));
    tried(source.set_smooth_angle(0.125));
    Handle::new(source)
}

fn point_op(r: &mut Reader<'_>, l: &Graphic3dCLight, op: i32) {
    let [x, y, z] = [0; 3].map(|_| r.real());
    tried(match op {
        5 => l.set_position_coords(x, y, z),
        6 => l.set_display_position(&Point3::new(x, y, z)),
        _ => l.set_direction_coords(x, y, z),
    });
}

fn scalar_op(r: &mut Reader<'_>, l: &Graphic3dCLight, op: i32) {
    let value = r.real() as f32;
    tried(match op {
        8 => l.set_intensity(value),
        9 => l.set_angle(value),
        11 => l.set_concentration(value),
        12 => l.set_smooth_radius(value),
        13 => l.set_smooth_angle(value),
        _ => l.set_range(value),
    });
}

fn light_op(r: &mut Reader<'_>, l: &Graphic3dCLight) {
    match r.integer() {
        1 => l.set_color(rgb(r)),
        2 => l.set_enabled(r.flag()),
        3 => tried(l.set_cast_shadows(r.flag())),
        4 => tried(l.set_headlight(r.flag())),
        op @ 5..=7 => point_op(r, l, op),
        10 => {
            let (c, li) = (r.real() as f32, r.real() as f32);
            tried(l.set_attenuation(c, li));
        }
        15 => {
            tried(l.copy_from(&copy_source(light_type(r.integer()))));
            flag(l.name().as_bytes() == b"source");
        }
        op => scalar_op(r, l, op),
    }
}

/// `sv_light`: a light, its state after construction and after each operation.
pub fn light_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let light = Graphic3dCLight::new(light_type(r.integer()));
    light_state(&light);
    for _ in 0..r.integer() {
        light_op(&mut r, &light);
        light_state(&light);
    }
    end();
}

fn set_state(s: &Graphic3dLightSet) {
    use Graphic3dLightSetIterationFilter as F;
    int(s.update_revision() as i32);
    int(s.extent());
    for kind in 0..4 {
        int(s.nb_lights_of_type(light_type(kind)));
        int(s.nb_enabled_lights_of_type(light_type(kind)));
    }
    int(s.nb_enabled());
    int(s.nb_cast_shadows());
    vec4f(s.ambient_color());
    text(s.key_enabled_long().as_bytes());
    text(s.key_enabled_short().as_bytes());
    let filters = [
        F::None,
        F::ExcludeAmbient,
        F::ExcludeDisabled,
        F::ExcludeNoShadow,
        F::ExcludeDisabledAndAmbient,
        F::ActiveShadowCasters,
    ];
    for filter in filters {
        let mut it = Graphic3dLightSetIterator::new(s, filter);
        let mut count = 0;
        while it.more() {
            int(it.value().light_type() as i32);
            count += 1;
            it.next();
        }
        int(count);
    }
}

/// Reads one light of the set request and adds it twice.
fn read_set_light(r: &mut Reader<'_>, s: &Graphic3dLightSet) -> Handle<Graphic3dCLight> {
    let light = Handle::new(Graphic3dCLight::new(light_type(r.integer())));
    light.set_enabled(r.flag());
    if r.flag() {
        tried(light.set_cast_shadows(true));
    }
    tried(light.set_intensity(r.real() as f32));
    light.set_color(rgb(r));
    show(s.add(&light), flag);
    show(s.add(&light), flag);
    light
}

/// `sv_light_set`: lights added to a set, its state, then a removal and a toggle.
pub fn light_set_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let set = Graphic3dLightSet::default();
    let lights: Vec<_> = (0..r.integer())
        .map(|_| read_set_light(&mut r, &set))
        .collect();
    show(set.add(&Handle::default()), |_| ());
    int(set.revision() as i32);
    set_state(&set);
    int(set.update_revision() as i32);
    let (removed, toggled) = (r.integer() as usize, r.integer() as usize);
    if removed > 0 {
        flag(set.remove(&lights[removed - 1]));
        flag(set.remove(&lights[removed - 1]));
        flag(set.contains(&lights[removed - 1]));
    }
    for index in 1..=set.extent() {
        show(set.value(index), |light| int(light.light_type() as i32));
    }
    show(set.value(0), |_| ());
    if toggled > 0 {
        let light = &lights[toggled - 1];
        light.set_enabled(!light.is_enabled());
    }
    set_state(&set);
    end();
}
