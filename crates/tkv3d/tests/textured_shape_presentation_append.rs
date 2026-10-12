//! Native appearance overrides must observe a valid presentation appended during a group callback.
#[allow(dead_code)] // Shared named fixture also supports group-only callbacks in another replay module.
#[path = "support/textured_appending_group.rs"]
mod appending_group;
#[allow(dead_code)]
#[path = "support/textured_shape_state_fixture.rs"]
mod state_fixture;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dDrawer, v3d::V3dViewer};
use state_fixture::TexturedStateFixture;

#[test]
fn textured_appearance_traverses_presentations_appended_by_native_group_callbacks() {
    for operation in 0..3 {
        let object = prepared_presentation_append_object(operation);
        let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
        let owner = PrsMgrPresentationManager::new(&viewer.structure_manager());
        let (first, next) = appendable_textured_presentations(&object, &owner);
        change_textured_append_appearance(&object, operation);
        compare_appended_presentations(&object, &first, &next, operation);
    }
}
fn prepared_presentation_append_object(operation: i32) -> Handle<TexturedStateFixture> {
    let object = TexturedStateFixture::new();
    object.attributes().setup_own_defaults().unwrap();
    object.set_texture_map_off();
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    object.attributes().link_to(&linked);
    if operation == 2 {
        ais_shape_set_material(
            &*object,
            &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold),
        )
        .unwrap();
    }
    object
}
fn appendable_textured_presentations(
    object: &Handle<TexturedStateFixture>,
    owner: &Handle<PrsMgrPresentationManager>,
) -> (Handle<PrsMgrPresentation>, Handle<PrsMgrPresentation>) {
    let first = PrsMgrPresentation::new(owner, &object.cast(), 3).unwrap();
    let next = PrsMgrPresentation::new(owner, &object.cast(), 3).unwrap();
    install_presentation_appending_group(&first, object, &next);
    next.new_group().unwrap();
    object
        .presentable_base()
        .fields
        .write()
        .presentations
        .push(first.clone());
    (first, next)
}
fn install_presentation_appending_group(
    first: &Handle<PrsMgrPresentation>,
    object: &Handle<TexturedStateFixture>,
    next: &Handle<PrsMgrPresentation>,
) {
    let group = appending_group::TexturedAppendingGroup::with_presentation(
        &first.cast(),
        &object.cast(),
        next,
    );
    first
        .cstructure()
        .cstructure_base()
        .fields
        .write()
        .groups
        .push(group.cast());
}
fn change_textured_append_appearance(object: &TexturedStateFixture, operation: i32) {
    let gold = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold);
    match operation {
        0 => ais_textured_shape_set_color(object, Color::named(NamedColor::RED)).unwrap(),
        1 => ais_textured_shape_set_material(object, &gold).unwrap(),
        _ => ais_textured_shape_unset_material(object).unwrap(),
    }
}
fn compare_appended_presentations(
    object: &TexturedStateFixture,
    first: &PrsMgrPresentation,
    next: &PrsMgrPresentation,
    operation: i32,
) {
    let a = first.groups()[0].aspects();
    let b = next.groups()[0].aspects();
    let final_aspect: Handle<dyn Graphic3dAspectsSource> = object.base.fields.read().aspect.cast();
    let actual = format!(
        "MARTENSITE_APPENDED_PRESENTATION {operation} {} {} {}",
        object.presentations().len(),
        i32::from(a == b),
        i32::from(b == final_aspect)
    );
    assert!(
        include_str!("fixtures/ais_textured_shape/native-presentation-append.txt")
            .contains(&actual)
    );
}
