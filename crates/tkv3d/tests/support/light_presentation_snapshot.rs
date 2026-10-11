//! Capture semantic aspects, closed groups and emitted text alongside primitive callbacks.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*};
pub fn constructor(object: &AisLightSource) -> Vec<f64> {
    let mut values = light_constructor_modes(object);
    append_point_constructor(object, &mut values);
    append_arrow_constructor(object, &mut values);
    append_shading_constructor(object, &mut values);
    append_text_constructor(object, &mut values);
    values
}
fn append_text_constructor(object: &AisLightSource, values: &mut Vec<f64>) {
    let drawer = object.attributes();
    let highlight = object.dynamic_hilight_attributes();
    let text = drawer.text_aspect();
    values.extend(
        [
            text.horizontal_justification() as i32,
            text.vertical_justification() as i32,
            text.aspect().text_display_type() as i32,
        ]
        .map(f64::from),
    );
    values.extend(highlight.color().rgb().map(f64::from));
}
fn append_point_constructor(object: &AisLightSource, values: &mut Vec<f64>) {
    let aspect = object.attributes().point_aspect().aspect();
    values.extend(aspect.interior_color().rgb().map(f64::from));
    values.extend([
        aspect.marker_type() as i32 as f64,
        f64::from(aspect.marker_scale()),
    ]);
}
fn append_arrow_constructor(object: &AisLightSource, values: &mut Vec<f64>) {
    let aspect = object.attributes().arrow_aspect().aspect();
    values.extend(aspect.interior_color().rgb().map(f64::from));
    values.extend([
        aspect.shading_model() as i32 as f64,
        aspect.marker_type() as i32 as f64,
        f64::from(aspect.marker_scale()),
    ]);
}
fn append_shading_constructor(object: &AisLightSource, values: &mut Vec<f64>) {
    let aspect = object.attributes().shading_aspect().aspect();
    values.extend(aspect.interior_color().rgb().map(f64::from));
    values.extend([
        aspect.shading_model() as i32 as f64,
        f64::from(aspect.front_material().transparency()),
    ]);
}
pub fn append_groups(prs: &Handle<dyn Graphic3dStructureSource>, values: &mut Vec<f64>) {
    let groups = prs.groups();
    values.push(groups.len() as f64);
    for group in groups {
        let aspect = group.aspects();
        values.push(f64::from(group.is_closed()));
        values.extend(aspect.interior_color().rgb().map(f64::from));
        values.extend([
            aspect.shading_model() as i32 as f64,
            aspect.marker_type() as i32 as f64,
            f64::from(aspect.marker_scale()),
            f64::from(aspect.line_width()),
            f64::from(aspect.front_material().transparency()),
        ]);
    }
}
pub fn append_texts(texts: &[Handle<Graphic3dText>], values: &mut Vec<f64>) {
    values.push(texts.len() as f64);
    for text in texts {
        let value = text.text();
        let bytes = &value.to_c_string()[..value.size() as usize];
        values.extend([f64::from(value.length()), bytes.len() as f64]);
        values.extend(bytes.iter().copied().map(f64::from));
    }
}

fn light_constructor_modes(object: &AisLightSource) -> Vec<f64> {
    let drawer = object.attributes();
    let persistence = object.presentable_base().read().transform_persistence;
    let mut values = vec![
        f64::from(object.is_infinite()),
        object.kind_of_interactive() as i32 as f64,
        object.signature() as f64,
    ];
    values.extend(
        [
            persistence.get().map(|p| p.mode().0).unwrap_or_default(),
            drawer.z_layer(),
            object.dynamic_hilight_attributes().z_layer(),
            drawer.display_mode(),
            object.dynamic_hilight_attributes().display_mode(),
        ]
        .map(f64::from),
    );
    values
}
