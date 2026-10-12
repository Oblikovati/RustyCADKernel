//! Native copy/fresh texture/all-group/repeat semantics replay through real retained state.
#[path = "support/textured_allocation_factory.rs"]
mod allocation_factory;
#[allow(dead_code)] // Shared named fixture also serves the presentation-append regression.
#[path = "support/textured_appending_group.rs"]
mod appending_group;
#[allow(dead_code)] // Shared named fixture also serves source-state-only replay modules.
#[path = "support/textured_shape_state_fixture.rs"]
mod state_fixture;
use allocation_factory::TexturedAllocationFactory;
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use rustycad_tkservice::{
    graphic3d::*,
    image::{ImageFormat, ImagePixMap},
    null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::{ais::*, prs_mgr::PrsMgrPresentableObjectMembers, v3d::V3dViewer};
use state_fixture::TexturedStateFixture;
use std::sync::Arc;
const NATIVE: &str = include_str!("fixtures/ais_textured_shape/native-aspects.txt");

#[test]
fn native_update_attributes_assigns_fresh_copies_to_every_existing_group() {
    let object = TexturedStateFixture::with_factory(Arc::new(TexturedAllocationFactory::default()));
    object.attributes().setup_own_defaults().unwrap();
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    prs.new_group().unwrap();
    prs.new_group().unwrap();
    for index in 0..5 {
        configure_aspect_case(&object, index);
        replay_aspect_case(&object, &prs.cast(), index);
    }
}
fn configure_aspect_case(object: &TexturedStateFixture, index: i32) {
    match index {
        0 => object.set_texture_map_off(),
        1 => configure_loaded_aspect_case(object),
        2 => {
            object.set_texture_map_off();
            object.set_show_triangles(true);
        }
        3 => {
            object.set_texture_pix_map(&Handle::new(ImagePixMap::default()));
            object.set_texture_map_on();
            object.disable_texture_modulate();
        }
        4 => object
            .set_texture_file_name(&AsciiString::default())
            .unwrap(),
        _ => panic!("aspect case {index}; expected 0..5"),
    }
}
fn configure_loaded_aspect_case(object: &TexturedStateFixture) {
    let pixels = Handle::new(ImagePixMap::default());
    assert!(pixels.init_zero(ImageFormat::Rgba, 2, 2, 0, 0));
    pixels.set_top_down(true);
    object.set_texture_pix_map(&pixels);
    object.set_texture_map_on();
    object.set_texture_repeat(false, 2., 3.);
}
fn replay_aspect_case(
    object: &TexturedStateFixture,
    prs: &Handle<dyn Graphic3dStructureSource>,
    index: i32,
) {
    let old = object.base.fields.snapshot();
    ais_textured_shape_update_attributes(object, prs).unwrap();
    let fields = object.base.fields.snapshot();
    let mut actual = aspect_case_flags(prs, &old, &fields);
    append_aspect_sampler_flags(&mut actual, &fields.texture);
    let prefix = format!("MARTENSITE_ASPECT {index} ");
    let expected: Vec<i32> = NATIVE
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
        .split_whitespace()
        .map(|word| word.parse().unwrap())
        .collect();
    assert_eq!(actual, expected, "native update case {index}");
}
fn aspect_case_flags(
    prs: &Handle<dyn Graphic3dStructureSource>,
    old: &AisTexturedShapeFields,
    fields: &AisTexturedShapeFields,
) -> Vec<i32> {
    let aspect = &fields.aspect;
    vec![
        i32::from(aspect != &old.aspect),
        i32::from(fields.texture.is_null()),
        i32::from(fields.texture == old.texture),
        i32::from(aspect.to_map_texture()),
        i32::from(aspect.to_draw_edges()),
        i32::from(aspect.texture_map() == fields.texture.cast::<dyn Graphic3dTextureMap>()),
        i32::from(prs.groups()[0].aspects() == aspect.cast::<dyn Graphic3dAspectsSource>()),
        i32::from(prs.groups()[1].aspects() == aspect.cast::<dyn Graphic3dAspectsSource>()),
    ]
}
fn append_aspect_sampler_flags(
    flags: &mut Vec<i32>,
    texture: &Handle<dyn Graphic3dTexture2DSource>,
) {
    let Some(texture) = texture.get() else {
        flags.extend([-1, -1]);
        return;
    };
    flags.extend([
        i32::from(texture.is_repeat()),
        i32::from(texture.is_modulate()),
    ]);
}
#[test]
fn native_virtual_aspect_callback_append_is_included_in_the_same_update() {
    let object = TexturedStateFixture::new();
    object.attributes().setup_own_defaults().unwrap();
    object.set_texture_map_off();
    let prs = appending_textured_presentation();
    ais_textured_shape_update_attributes(&*object, &prs.cast()).unwrap();
    let actual = appended_textured_aspect_flags(&object, &prs);
    assert_eq!(actual.as_slice(), native_appended_group_flags());
}
fn appending_textured_presentation() -> Handle<Graphic3dStructure> {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let first = appending_group::TexturedAppendingGroup::new(&prs.cast());
    prs.cstructure()
        .cstructure_base()
        .fields
        .write()
        .groups
        .push(first.cast());
    prs
}
fn appended_textured_aspect_flags(
    object: &TexturedStateFixture,
    prs: &Graphic3dStructure,
) -> [i32; 3] {
    let aspect = object
        .base
        .fields
        .read()
        .aspect
        .cast::<dyn Graphic3dAspectsSource>();
    let groups = prs.groups();
    [
        groups.len() as i32,
        i32::from(groups[0].aspects() == aspect),
        i32::from(groups[1].aspects() == aspect),
    ]
}
fn native_appended_group_flags() -> Vec<i32> {
    include_str!("fixtures/ais_textured_shape/native-group-append.txt")
        .trim()
        .strip_prefix("MARTENSITE_APPENDED_GROUP ")
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect()
}
